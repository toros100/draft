use draft::{
    construction::{ObjectArena, ObjectId},
    expression, geom,
    render::{Renderer, texture_dimensions},
    slint_gen::{self, DataCallbacks, HoverInfo, ToolData},
    tool::{self, ToolResponse},
};
use slint::{
    ComponentHandle,
    language::PointerEventKind,
    platform::PointerEventButton,
    wgpu_29::{WGPUSettings, wgpu},
};
use std::{
    cell::{Cell, RefCell},
    f64::consts::{FRAC_PI_2, FRAC_PI_8},
    rc::Rc,
};

use vello::kurbo::{self, Affine};

#[derive(Debug)]
struct Flags {
    view_dirty: Cell<bool>,
    arena_dirty: Cell<bool>,
    tool_overlay_dirty: Cell<bool>,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            view_dirty: Cell::new(true),
            arena_dirty: Cell::new(true),
            tool_overlay_dirty: Cell::new(true),
        }
    }
}

impl Flags {
    fn set_view_dirty(&self) {
        self.view_dirty.set(true)
    }
    fn set_arena_dirty(&self) {
        self.arena_dirty.set(true)
    }
    fn set_tool_overlay_dirty(&self) {
        self.tool_overlay_dirty.set(true)
    }

    fn view_dirty(&self) -> bool {
        self.view_dirty.get()
    }
    fn arena_dirty(&self) -> bool {
        self.arena_dirty.get()
    }
    fn tool_overlay_dirty(&self) -> bool {
        self.tool_overlay_dirty.get()
    }

    fn needs_redraw(&self) -> bool {
        self.view_dirty() || self.arena_dirty() || self.tool_overlay_dirty()
    }

    fn clear(&self) {
        self.view_dirty.set(false);
        self.arena_dirty.set(false);
        self.tool_overlay_dirty.set(false);
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

fn main() -> Result<(), slint::PlatformError> {
    let mut settings = WGPUSettings::default();
    settings.device_required_limits = wgpu::Limits::defaults();

    slint::BackendSelector::new()
        .require_wgpu_29(slint::wgpu_29::WGPUConfiguration::Automatic(settings))
        .select()?;

    let main_window = slint_gen::MainWindow::new()?;

    let renderer: Rc<RefCell<Option<Renderer>>> = Rc::default();
    let view = Rc::new(Cell::new(View::default()));
    let flags = Rc::new(Flags::default());

    let selected = Rc::new(Cell::new(Option::<ObjectId>::None));

    let arena = Rc::new(RefCell::new(ObjectArena::default()));

    {
        let mut arena = arena.borrow_mut();

        let p = arena.add_point_free(geom::point2(200., 300.)).unwrap();

        let q = arena
            .add_point_dist_angle(p, expression::length(200.), expression::angle(0.))
            .unwrap();

        let r = arena
            .add_point_dist_angle(p, expression::length(200.), expression::angle(FRAC_PI_2))
            .unwrap();

        let c = arena
            .add_curve(
                p,
                q,
                geom::polar(150., -FRAC_PI_2),
                geom::polar(100., -FRAC_PI_8),
            )
            .unwrap();
        arena
            .add_point_on_line(p, r, expression::length(50.))
            .unwrap();

        arena
            .add_point_on_curve(c, expression::length(90.))
            .unwrap();

        arena.evaluate_all().unwrap();
    }

    // TODO: actual toolbar, this is just a quick hack
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

    let tool = Rc::new(RefCell::new(None));
    main_window.set_tools(tools_model.clone());
    main_window.on_tool_choice({
        // HACK:
        let tool = tool.clone();
        let flags = flags.clone();
        let main_window = main_window.as_weak();

        move |i| {
            let mut tool = tool.borrow_mut();
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
            flags.set_tool_overlay_dirty();
            if let Some(m) = main_window.upgrade() {
                m.set_selected_tool(i);
            }
        }
    });

    let data_callbacks = main_window.global::<DataCallbacks>();

    data_callbacks.on_submit_data({
        let arena = arena.clone();
        let flags = flags.clone();
        let main_window = main_window.as_weak();

        move |upd| {
            if let Err(e) = arena.borrow_mut().try_apply_update(upd) {
                // TODO: need a way to surface error to UI
                eprintln!("failed to apply update: {}", e)
            };
            flags.set_arena_dirty();
            if let Some(w) = main_window.upgrade() {
                w.window().request_redraw()
            }
        }
    });

    main_window.on_delete_object({
        let arena = arena.clone();
        let flags = flags.clone();
        let main_window = main_window.as_weak();

        move |id| {
            let mut arena = arena.borrow_mut();
            let id = id.into();
            if arena.can_delete(id) {
                arena
                    .try_delete(id)
                    .expect("surely the check would not lie");
            }
            flags.set_arena_dirty();
            if let Some(m) = main_window.upgrade() {
                m.window().request_redraw();
            }
        }
    });

    main_window.on_pointer_event({
        // TODO: need to come up with a more robust way of handling which thing is hovered/selected
        let selected = selected.clone();
        let mut middle_drag_enter = None;
        let main_window = main_window.as_weak();
        let v = view.clone();
        let arena = arena.clone();
        let view = view.clone();
        let flags = flags.clone();

        let tool_rc = tool.clone();

        move |k, b, x, y, m| {
            let Some(main_window) = main_window.upgrade() else {
                return;
            };

            let mut arena = arena.borrow_mut();

            let mut tool = tool_rc.borrow_mut();

            let screen_target = geom::point2(x as f64, y as f64);
            let world_target = view.get().affine().inverse() * kurbo::Point::from(screen_target);

            // deliberately not implementing panning as a tool, so that it can be used concurrently
            // with a tool
            match k {
                PointerEventKind::Down if b == PointerEventButton::Middle => {
                    middle_drag_enter = Some((screen_target, v.get().translation()));
                    main_window.set_panning(true);
                }
                PointerEventKind::Up | PointerEventKind::Cancel
                    if b == PointerEventButton::Middle =>
                {
                    middle_drag_enter = None;
                    main_window.set_panning(false);
                }
                PointerEventKind::Move if let Some((p, q)) = middle_drag_enter => {
                    let disp = screen_target - p;
                    let disp_world = disp * (1. / v.get().scale());
                    v.set(v.get().with_translation(disp_world + q));
                    flags.set_view_dirty();
                }
                _ => {}
            }

            if !main_window.get_panning() {
                let hit = arena.hit_scan(world_target, 10.);

                let can_interact = tool.as_ref().is_some_and(|t| t.can_interact(hit));

                let info = match hit {
                    None => HoverInfo {
                        can_interact,
                        ..Default::default()
                    },
                    Some(g) => {
                        let id = g.id();
                        HoverInfo {
                            can_delete: arena.can_delete(id),
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
                        let resp = match arena.get_data_for(hit.id()) {
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
                        selected.set(Some(hit.id()));
                        main_window.set_selected_data(resp);
                    } else {
                        selected.set(None);
                        main_window.set_selected_data(slint_gen::ObjectDataResponse::default());
                    }
                }

                if let Some(tool) = tool.as_mut() {
                    let tool_state = match k {
                        PointerEventKind::Down if b == PointerEventButton::Left => {
                            Some(tool.submit(
                                tool::ToolInput {
                                    cursor: world_target.into(),
                                    modifiers: m,
                                    mouse: tool::Mouse::Press,
                                    hover: hit,
                                },
                                &arena,
                            ))
                        }
                        PointerEventKind::Down if b == PointerEventButton::Right => {
                            tool.reset();
                            flags.set_tool_overlay_dirty();
                            None
                        }
                        PointerEventKind::Up if b == PointerEventButton::Left => Some(tool.submit(
                            tool::ToolInput {
                                cursor: world_target.into(),
                                modifiers: m,
                                mouse: tool::Mouse::Release,
                                hover: hit,
                            },
                            &arena,
                        )),
                        PointerEventKind::Move => Some(tool.submit(
                            tool::ToolInput {
                                cursor: world_target.into(),
                                modifiers: m,
                                mouse: tool::Mouse::Move,
                                hover: hit,
                            },
                            &arena,
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
                            if let Err(e) = arena.apply_action(action) {
                                eprintln!("failed to apply action: {e:?}");
                            };
                            flags.set_arena_dirty();
                        }

                        if done {
                            tool.reset();
                        }

                        if done || overlay_changed {
                            flags.set_tool_overlay_dirty();
                        }
                    }
                }
            }
            if flags.needs_redraw() {
                main_window.window().request_redraw();
            }
        }
    });

    let mut before_rendering = {
        let mut dims = (0, 0);
        let main_window = main_window.as_weak();
        let renderer = renderer.clone();
        let arena = arena.clone();
        let view = view.clone();
        let tool = tool.clone();
        let flags = flags.clone();
        let selected = selected.clone();

        move || {
            if let (Some(main_window), Some(renderer)) =
                (main_window.upgrade(), renderer.borrow_mut().as_mut())
            {
                let (w, h) = {
                    let w = main_window.get_canvas_width();
                    let h = main_window.get_canvas_height();
                    let s = main_window.window().scale_factor();
                    draft::render::texture_dimensions(w, h, s)
                };

                if (w, h) != dims {
                    // (always sets flag on first call, the above can not produce (0, 0))
                    flags.set_view_dirty();
                    dims = (w, h)
                }

                if flags.tool_overlay_dirty() {
                    let t = tool.borrow();
                    let overlay = if let Some(tool) = t.as_ref() {
                        tool.overlay()
                    } else {
                        &[]
                    };
                    renderer.build_tool_scene(overlay);
                }

                let mut arena = arena.borrow_mut();

                if flags.arena_dirty() {
                    if let Err(e) = arena.evaluate_all() {
                        // right now, this should never actually error. but in the future,
                        // when i properly implement formulas, it could error in ways that
                        // not be prevented by static analysis of the formula itself
                        // (e.g. dividing by zero)
                        // will be especially problematic with my magic move tool
                        eprintln!("failed to evaluate: {e}")
                    };
                    if let Some(id) = selected.get() {
                        match arena.get_data_for(id) {
                            Ok(data) => {
                                main_window.set_selected_data(slint_gen::ObjectDataResponse {
                                    data,
                                    id: id.into(),
                                    ok: true,
                                    ..Default::default()
                                });
                            }
                            Err(e) => {
                                if !matches!(e, draft::construction::ArenaError::Unimplemented) {
                                    eprintln!("failed to get data for {id:?}: {e}")
                                }
                            }
                        }
                    }
                    renderer.build_main_scene(arena.iter_evaluated());
                }

                if flags.needs_redraw() {
                    main_window.set_canvas(renderer.render(view.get().affine(), w, h));
                }
                flags.clear();
            }
        }
    };

    main_window
        .window()
        .set_rendering_notifier({
            let main_window = main_window.as_weak();

            move |state, graphics_api| match state {
                slint::RenderingState::RenderingSetup => {
                    let slint::GraphicsAPI::WGPU29 { device, queue, .. } = graphics_api else {
                        panic!("unexpected graphics API");
                    };

                    let (w, h, s) = if let Some(m) = main_window.upgrade() {
                        (
                            m.get_canvas_width(),
                            m.get_canvas_height(),
                            m.window().scale_factor(),
                        )
                    } else {
                        return;
                    };

                    let (w, h) = texture_dimensions(w, h, s);

                    // TODO: use scale factor in renderer
                    let r = Renderer::new(device.clone(), queue.clone(), w, h).unwrap();
                    _ = renderer.borrow_mut().insert(r);
                }
                slint::RenderingState::BeforeRendering => {
                    // WARN: if nothing else causes a redraw (slint), then a redraw needs to be
                    // requested manually for this to even run
                    before_rendering()
                }
                slint::RenderingState::RenderingTeardown => {
                    renderer.borrow_mut().take();
                }
                _ => {}
            }
        })
        .unwrap();

    main_window.run()
}
