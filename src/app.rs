use std::{cell::RefCell, rc::Rc};

use crate::{
    construction::{ObjectArena, ObjectId},
    geom::{self, Point2, Vec2},
    render::{Renderer, texture_dimensions},
    slint_gen::{
        self, HoverInfo, MainWindow, ObjectDataResponse, ObjectDataUpdate, TaggedObjectId, ToolData,
    },
    tool::{self, ToolResponse},
};

use slint::{
    ComponentHandle, SetRenderingNotifierError, Weak,
    language::{KeyboardModifiers, PointerEventKind},
    platform::PointerEventButton,
    wgpu_29::wgpu,
};

use vello::kurbo::{self, Affine};

pub struct App {
    window: Weak<MainWindow>,
    window_dims: (f32, f32),
    arena: ObjectArena,
    tool: Option<Box<dyn tool::Tool>>,
    selected: Option<ObjectId>,
    renderer: Option<Renderer>,
    view: View,
    middle_drag_enter: Option<(Point2, Vec2)>,
    flags: Flags,
}

impl App {
    pub fn new(window: slint::Weak<MainWindow>) -> Self {
        let tools_model = slint::ModelRc::new(slint::VecModel::from(vec![
            ToolData {
                name: "move".into(),
            },
            ToolData {
                name: "free point".into(),
            },
            ToolData {
                name: "point dist/angle".into(),
            },
            ToolData {
                name: "point on line".into(),
            },
            ToolData {
                name: "point on curve".into(),
            },
            ToolData {
                name: "line".into(),
            },
            ToolData {
                name: "curve".into(),
            },
        ]));
        window.upgrade().unwrap().set_tools(tools_model.clone());

        Self {
            window,
            window_dims: (0., 0.),
            arena: ObjectArena::default(),
            renderer: None,
            view: View::default(),
            tool: None,
            selected: None,
            middle_drag_enter: None,
            flags: Flags::default(),
        }
    }

    pub fn arena_mut(&mut self) -> &mut ObjectArena {
        &mut self.arena
    }

    pub fn init_callbacks(app: Rc<RefCell<App>>) -> Result<(), SetRenderingNotifierError> {
        let window = app.borrow().window.unwrap();

        window.window().set_rendering_notifier({
            let main_window = window.as_weak();
            let app = app.clone();

            move |state, graphics_api| match state {
                slint::RenderingState::RenderingSetup => {
                    let slint::GraphicsAPI::WGPU29 { device, queue, .. } = graphics_api else {
                        panic!("unexpected graphics API");
                    };
                    app.borrow_mut()
                        .init_renderer(device.clone(), queue.clone())
                        .unwrap();
                }
                slint::RenderingState::BeforeRendering => {
                    // WARN: if nothing else causes a redraw (slint), then a redraw needs to be
                    // requested manually for this to even run

                    if let Some(main_window) = main_window.upgrade()
                        && let Some(img) = app.borrow_mut().next_frame()
                    {
                        main_window.set_canvas(img);
                    }
                }
                _ => {}
            }
        })?;

        window.on_pointer_event({
            let app = app.clone();
            move |k, b, x, y, m| {
                let mut app = app.borrow_mut();
                app.handle_pointer_event(k, b, x, y, m);
                app.handle_flags();
            }
        });

        window.on_tool_choice({
            let app = app.clone();
            move |i| {
                let mut app = app.borrow_mut();
                app.handle_tool_choice(i);
                app.handle_flags();
            }
        });

        window.on_canvas_size_changed({
            let app = app.clone();
            move |w, h| {
                let mut app = app.borrow_mut();
                app.handle_canvas_size_changed(w, h);
                app.handle_flags();
            }
        });

        window.on_update_object({
            let app = app.clone();
            move |upd| {
                let mut app = app.borrow_mut();
                app.handle_update_object(upd);
                app.handle_flags();
            }
        });

        window.on_delete_object({
            let app = app.clone();
            move |id| {
                let mut app = app.borrow_mut();
                app.handle_delete_object(id);
                app.handle_flags();
            }
        });

        Ok(())
    }

    fn handle_update_object(&mut self, upd: ObjectDataUpdate) {
        if let Err(e) = self.arena.try_apply_update(upd) {
            // TODO: need a way to surface error to UI
            eprintln!("failed to apply update: {}", e)
        };
        self.flags.set_arena_dirty();
    }

    fn handle_delete_object(&mut self, id: TaggedObjectId) {
        let id = ObjectId::from(id);
        if self.arena.can_delete(id) {
            self.arena
                .try_delete(id)
                .expect("surely the check would not lie");
            self.flags.set_arena_dirty();
        }
    }

    fn handle_canvas_size_changed(&mut self, w: f32, h: f32) {
        if (w, h) != self.window_dims {
            self.window_dims = (w, h);
            self.flags.set_dims_dirty();
        }
    }

    fn handle_tool_choice(&mut self, i: i32) {
        // HACK:
        let tool = &mut self.tool;
        match i {
            0 => _ = tool.insert(tool::default_boxed::<tool::Move>()),
            1 => _ = tool.insert(tool::default_boxed::<tool::AddPointFree>()),
            2 => _ = tool.insert(tool::default_boxed::<tool::AddPointDistAngle>()),
            3 => _ = tool.insert(tool::default_boxed::<tool::AddPointOnLine>()),
            4 => _ = tool.insert(tool::default_boxed::<tool::AddPointOnCurve>()),
            5 => _ = tool.insert(tool::default_boxed::<tool::AddLine>()),
            6 => _ = tool.insert(tool::default_boxed::<tool::AddCurve>()),
            _ => _ = tool.take(),
        }
        self.flags.set_tool_overlay_dirty();
        if let Some(m) = self.window.upgrade() {
            m.set_selected_tool(i);
        }
    }

    fn handle_pointer_event(
        &mut self,
        k: PointerEventKind,
        b: PointerEventButton,
        x: f32,
        y: f32,
        m: KeyboardModifiers,
    ) {
        let Some(main_window) = self.window.upgrade() else {
            return;
        };

        // not guaranteed to get a redraw between each two pointer events, so evaluating to make
        // sure the changes produces by a possible previous pointer event are applied
        _ = self.arena.evaluate_all();

        // HACK: find better way to keep selected data updated
        if let Some(id) = self.selected
            && let Ok(data) = self.arena.get_data_for(id)
        {
            main_window.set_selected_data(ObjectDataResponse {
                id: id.into(),
                data,
                ok: true,
                ..Default::default()
            });
        }

        let screen_target = geom::point2(x as f64, y as f64);
        let world_target = self.view.affine().inverse() * kurbo::Point::from(screen_target);

        // deliberately not implementing panning as a tool, so that it can be used concurrently
        // with a tool
        match k {
            PointerEventKind::Down if b == PointerEventButton::Middle => {
                self.middle_drag_enter = Some((screen_target, self.view.translation()));
                main_window.set_panning(true);
            }
            PointerEventKind::Up | PointerEventKind::Cancel if b == PointerEventButton::Middle => {
                self.middle_drag_enter = None;
                main_window.set_panning(false);
            }
            PointerEventKind::Move if let Some((p, q)) = self.middle_drag_enter => {
                let disp = screen_target - p;
                let disp_world = disp * (1. / self.view.scale());
                self.view = self.view.with_translation(disp_world + q);
                self.flags.set_view_dirty();
            }
            _ => {}
        }

        if !main_window.get_panning() {
            let hit = self.arena.hit_scan(world_target, 10.);

            let can_interact = self.tool.as_ref().is_some_and(|t| t.can_interact(hit));

            let info = match hit {
                None => HoverInfo {
                    can_interact,
                    ..Default::default()
                },
                Some(g) => {
                    let id = g.id();
                    HoverInfo {
                        can_delete: self.arena.can_delete(id),
                        can_interact,
                        id: id.into(),
                        is_some: true,
                    }
                }
            };
            main_window.set_hover_info(info);
            main_window.set_world_pos(geom::Point2::from(world_target).into());

            if k == PointerEventKind::Down && b == PointerEventButton::Left {
                if let Some(hit) = hit {
                    let resp = match self.arena.get_data_for(hit.id()) {
                        Ok(data) => slint_gen::ObjectDataResponse {
                            id: hit.id().into(),
                            ok: true,
                            data,
                            ..Default::default()
                        },

                        Err(e) => slint_gen::ObjectDataResponse {
                            id: hit.id().into(),
                            err: e.to_string().into(),
                            ok: true,
                            ..Default::default()
                        },
                    };
                    self.selected = Some(hit.id());
                    main_window.set_selected_data(resp);
                } else {
                    self.selected = None;
                    main_window.set_selected_data(slint_gen::ObjectDataResponse::default());
                }
            }

            if let Some(tool) = self.tool.as_mut() {
                let tool_state = match k {
                    PointerEventKind::Down if b == PointerEventButton::Left => Some(tool.submit(
                        tool::ToolInput {
                            cursor: world_target.into(),
                            modifiers: m,
                            mouse: tool::Mouse::Press,
                            hover: hit,
                        },
                        &self.arena,
                    )),
                    PointerEventKind::Down if b == PointerEventButton::Right => {
                        tool.reset();
                        self.flags.set_tool_overlay_dirty();
                        None
                    }
                    PointerEventKind::Up if b == PointerEventButton::Left => Some(tool.submit(
                        tool::ToolInput {
                            cursor: world_target.into(),
                            modifiers: m,
                            mouse: tool::Mouse::Release,
                            hover: hit,
                        },
                        &self.arena,
                    )),
                    PointerEventKind::Move => Some(tool.submit(
                        tool::ToolInput {
                            cursor: world_target.into(),
                            modifiers: m,
                            mouse: tool::Mouse::Move,
                            hover: hit,
                        },
                        &self.arena,
                    )),
                    PointerEventKind::Cancel if b == PointerEventButton::Left => {
                        tool.reset();
                        None
                    }
                    _ => None,
                };

                if let Some(ToolResponse {
                    action,
                    done,
                    overlay_changed,
                }) = tool_state
                {
                    if let Some(action) = action {
                        // TODO:
                        if let Err(e) = self.arena.apply_action(action) {
                            eprintln!("failed to apply action: {e:?}");
                        };
                        self.flags.set_arena_dirty();
                    }

                    if done {
                        tool.reset();
                    }

                    if done || overlay_changed {
                        self.flags.set_tool_overlay_dirty();
                    }
                }
            }
        }
    }

    fn init_renderer(
        &mut self,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Result<(), vello::Error> {
        let window = self.window.unwrap();

        let (w, h) = texture_dimensions(1., 1., window.window().scale_factor());

        self.renderer = Some(Renderer::new(device, queue, w, h)?);
        Ok(())
    }

    fn next_frame(&mut self) -> Option<slint::Image> {
        let r = self.renderer.as_mut()?;

        let window = self.window.upgrade()?;

        if !self.flags.needs_redraw() {
            return None;
        };

        if self.flags.arena_dirty() {
            _ = self.arena.evaluate_all();
            r.build_main_scene(self.arena.iter_evaluated());
        }

        if self.flags.tool_overlay_dirty() {
            let ov = self.tool.as_ref().map_or([].as_slice(), |t| t.overlay());
            r.build_tool_scene(ov);
        }

        let (w, h) = texture_dimensions(
            self.window_dims.0,
            self.window_dims.1,
            window.window().scale_factor(),
        );

        self.flags.clear();
        Some(r.render(self.view.affine(), w, h))
    }

    fn handle_flags(&mut self) {
        let Some(window) = self.window.upgrade() else {
            return;
        };

        if self.flags.needs_redraw() {
            window.window().request_redraw();
        }
    }
}

#[derive(Debug)]
struct Flags {
    view_dirty: bool,
    arena_dirty: bool,
    tool_overlay_dirty: bool,
    dims_dirty: bool,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            view_dirty: true,
            arena_dirty: true,
            tool_overlay_dirty: true,
            dims_dirty: true,
        }
    }
}

impl Flags {
    fn set_view_dirty(&mut self) {
        self.view_dirty = true
    }

    fn set_arena_dirty(&mut self) {
        self.arena_dirty = true
    }

    fn set_tool_overlay_dirty(&mut self) {
        self.tool_overlay_dirty = true
    }

    fn set_dims_dirty(&mut self) {
        self.dims_dirty = true
    }

    fn arena_dirty(&self) -> bool {
        self.arena_dirty
    }

    fn tool_overlay_dirty(&self) -> bool {
        self.tool_overlay_dirty
    }

    fn needs_redraw(&self) -> bool {
        self.view_dirty || self.arena_dirty || self.tool_overlay_dirty || self.dims_dirty
    }

    fn clear(&mut self) {
        self.view_dirty = false;
        self.arena_dirty = false;
        self.tool_overlay_dirty = false;
        self.dims_dirty = false;
    }
}

#[derive(Clone, Copy, Debug)]
struct View {
    scale: f64,
    translation: geom::Vec2, // in world space
}

impl Default for View {
    fn default() -> Self {
        Self {
            scale: 1.0,
            translation: geom::Vec2::default(),
        }
    }
}

#[allow(unused)]
impl View {
    fn affine(&self) -> Affine {
        Affine::translate(self.translation).then_scale(self.scale)
    }
    fn scale(&self) -> f64 {
        self.scale
    }

    fn screen_to_world(&self, p: geom::Point2) -> geom::Point2 {
        let p = kurbo::Point::from(p);
        // no need to use the affine here, but i guess it is better to use the exact same pathway
        // the renderer uses
        (self.affine().inverse() * p).into()
    }

    fn translation(&self) -> geom::Vec2 {
        self.translation
    }
    fn with_scale(self, scale: f64) -> Self {
        Self {
            translation: self.translation,
            scale,
        }
    }
    fn with_translation(self, translation: geom::Vec2) -> Self {
        Self {
            translation,
            scale: self.scale,
        }
    }
}
