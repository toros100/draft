use crate::construction::variants::expression;
use draft::{
    DataCallbacks, ToolData,
    arena::Arena,
    construction::{self, Object, ObjectId, PathSementVal, PointObj},
    geom::{self},
    tool::{self, PathPrimitive, ToolResponse},
};
use slint::{
    ComponentHandle,
    language::PointerEventKind,
    platform::PointerEventButton,
    wgpu_29::{WGPUSettings, wgpu},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use vello::kurbo::{self, Affine, PathEl};
use vello::peniko::{self, color::palette};
use wgpu::util::TextureBlitter;

// should already fail to compile elsewhere
static_assertions::assert_type_eq_all!(slint::wgpu_29::wgpu::Device, vello::wgpu::Device);

const VELLO_USAGE: wgpu::TextureUsages =
    wgpu::TextureUsages::STORAGE_BINDING.union(wgpu::TextureUsages::TEXTURE_BINDING);

const SLINT_USAGE: wgpu::TextureUsages =
    wgpu::TextureUsages::TEXTURE_BINDING.union(wgpu::TextureUsages::RENDER_ATTACHMENT);

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

fn get_hover_info(arena: &Arena<Object>, id: Option<ObjectId>) -> draft::HoverInfo {
    let Some(id) = id else {
        return draft::HoverInfo::default();
    };

    let Some(tid) = arena.get_tagged_id(id) else {
        return draft::HoverInfo::default();
    };

    draft::HoverInfo {
        id: tid,
        is_some: true,
        can_delete: arena.can_delete(id),
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

    let main_window = draft::MainWindow::new()?;

    let renderer: Rc<RefCell<Option<Renderer>>> = Rc::default();
    let view = Rc::new(Cell::new(View::default()));
    let flags = Rc::new(Flags::default());

    let arena = Rc::new(RefCell::new(Arena::<Object>::default()));

    {
        let mut arena = arena.borrow_mut();

        let p = arena.add_root(geom::point2(300., 300.));
        arena.add_point_relative(p, expression::length(100.), expression::angle(1.5));
        arena.evaluate_all();
    }

    let tools_model = slint::ModelRc::new(slint::VecModel::from(vec![
        ToolData {
            name: "move".into(),
        },
        ToolData {
            name: "point dist/angle".into(),
        },
        ToolData {
            name: "line".into(),
        },
        ToolData {
            name: "free point".into(),
        },
        ToolData {
            name: "point on line".into(),
        },
        ToolData {
            name: "curve".into(),
        },
        ToolData {
            name: "point on curve".into(),
        },
        ToolData {
            name: "none".into(),
        },
    ]));

    let tool = Rc::new(RefCell::new(Some(tool::default_boxed::<tool::Drag>())));

    main_window.set_tools(tools_model.clone());

    main_window.on_tool_choice({
        // HACK:
        let tool = tool.clone();
        let flags = flags.clone();

        move |i| {
            let mut tool = tool.borrow_mut();
            match i {
                0 => _ = tool.insert(tool::default_boxed::<tool::Drag>()),
                1 => _ = tool.insert(tool::default_boxed::<tool::AddPointDistAngle>()),
                2 => _ = tool.insert(tool::default_boxed::<tool::Line>()),
                3 => _ = tool.insert(tool::default_boxed::<tool::Free>()),
                4 => _ = tool.insert(tool::default_boxed::<tool::OnLine>()),
                5 => _ = tool.insert(tool::default_boxed::<tool::Curve>()),
                6 => _ = tool.insert(tool::default_boxed::<tool::OnCurve>()),
                7 => _ = tool.take(),
                _ => {}
            }
            flags.set_tool_overlay_dirty();
        }
    });

    let data_callbacks = main_window.global::<DataCallbacks>();

    data_callbacks.on_submit_data({
        let arena = arena.clone();
        let flags = flags.clone();
        let main_window = main_window.as_weak();

        move |upd| {
            arena.borrow_mut().apply_object_data(upd);
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
            arena.borrow_mut().delete(id.into());
            flags.set_arena_dirty();

            if let Some(m) = main_window.upgrade() {
                m.window().request_redraw();
            }
        }
    });

    main_window.on_pointer_event({
        let mut middle_drag_enter = None;
        let main_window = main_window.as_weak();
        let v = view.clone();
        let arena = arena.clone();
        let view = view.clone();
        let flags = flags.clone();

        let tool = tool.clone();

        move |k, b, x, y, m| {
            let Some(main_window) = main_window.upgrade() else {
                return;
            };

            let mut arena = arena.borrow_mut();
            let mut tool = tool.borrow_mut();

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
                _ if middle_drag_enter.is_none() => {
                    let t = arena.hit_scan(world_target.into(), 10.);
                    main_window.set_hover_info(get_hover_info(&arena, t));
                    main_window.set_world_pos(geom::Point2::from(world_target).into());
                }
                _ => {}
            }

            if !main_window.get_panning()
                && tool.is_none()
                && k == PointerEventKind::Down
                && b == PointerEventButton::Left
            {
                let hit = arena.hit_scan(world_target.into(), 10.);
                let info = get_hover_info(&arena, hit);
                main_window.set_selected_info(info);

                if let Some(hit) = hit {
                    let data = arena.get_object_data(hit);
                    main_window.set_selected_data(data);
                } else {
                    main_window.set_selected_data(draft::ObjectDataResponse::default());
                }
            }

            // not updating tool state when panning
            if !main_window.get_panning()
                && let Some(tool) = tool.as_mut()
            {
                let tool_state = match k {
                    PointerEventKind::Down if b == PointerEventButton::Left => {
                        let hit = arena.hit_scan(world_target.into(), 10.);
                        Some(tool.submit(
                            tool::ToolInput::Press {
                                obj: hit,
                                pos: world_target.into(),
                            },
                            m,
                            &arena,
                        ))
                    }
                    PointerEventKind::Down if b == PointerEventButton::Right => {
                        tool.reset();
                        flags.set_tool_overlay_dirty();
                        None
                    }
                    PointerEventKind::Up if b == PointerEventButton::Left => {
                        let hit = arena.hit_scan(world_target.into(), 10.);
                        Some(tool.submit(
                            tool::ToolInput::Release {
                                obj: hit,
                                pos: world_target.into(),
                            },
                            m,
                            &arena,
                        ))
                    }
                    PointerEventKind::Move => {
                        let hit = arena.hit_scan(world_target.into(), 10.);
                        Some(tool.submit(
                            tool::ToolInput::Move {
                                obj: hit,
                                pos: world_target.into(),
                            },
                            m,
                            &arena,
                        ))
                    }
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
                        arena.apply_action(action);
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

        move || {
            if let (Some(main_window), Some(renderer)) =
                (main_window.upgrade(), renderer.borrow_mut().as_mut())
            {
                let (w, h) = {
                    let w = main_window.get_canvas_width();
                    let h = main_window.get_canvas_height();
                    let s = main_window.window().scale_factor();
                    texture_dimensions(w, h, s)
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

                if flags.arena_dirty() {
                    // how smart is it to do this in this callback? probably not that smart?
                    arena.borrow_mut().evaluate_all();
                    renderer.build_main_scene(&arena.borrow());
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
                    let r = Renderer::new(device.clone(), queue.clone(), w, h);
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

struct Renderer {
    tex_dims: (u32, u32),

    device: wgpu::Device,
    queue: wgpu::Queue,
    vello_renderer: vello::Renderer,
    blitter: wgpu::util::TextureBlitter,

    tool_scene: vello::Scene,
    main_scene: vello::Scene,
    combined_scene: vello::Scene,

    vello_tex: wgpu::Texture,
    vello_view: wgpu::TextureView,

    slint_tex: wgpu::Texture,
    slint_view: wgpu::TextureView,
}

impl Renderer {
    fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        texture_width: u32,
        texture_height: u32,
    ) -> Self {
        let v = vello::Renderer::new(
            &device,
            vello::RendererOptions {
                antialiasing_support: vello::AaSupport::area_only(),
                ..Default::default()
            },
        )
        .unwrap();

        let tex_dims = (texture_width, texture_height);

        let blitter = TextureBlitter::new(&device, wgpu::TextureFormat::Rgba8Unorm);

        let vello_tex = make_texture(&device, "vello_target", VELLO_USAGE, tex_dims.0, tex_dims.1);

        let vello_view = vello_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let slint_tex = make_texture(&device, "slint_target", SLINT_USAGE, tex_dims.0, tex_dims.1);

        let slint_view = slint_tex.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            tex_dims,
            device,
            queue,
            vello_renderer: v,
            blitter,
            main_scene: vello::Scene::new(),
            combined_scene: vello::Scene::new(),
            tool_scene: vello::Scene::new(),
            vello_tex,
            vello_view,
            slint_tex,
            slint_view,
        }
    }

    fn render(
        &mut self,
        transform: kurbo::Affine,
        texture_width: u32,
        texture_height: u32,
    ) -> slint::Image {
        // "reusing" self.content with a (possibly different) transform for pan/zoom without
        // having to actually rebuild the scene
        self.combined_scene.reset();
        self.combined_scene
            .append(&self.main_scene, Some(transform));
        self.combined_scene
            .append(&self.tool_scene, Some(transform));

        let tex_dims = (texture_width, texture_height);

        if tex_dims != self.tex_dims {
            // re-allocating a texture for the new size
            // if the window size is changed and the old texture of the wrong size is reused, the
            // image element in slint will scale its contents and appear blurry (not cool)

            // this does allocate a texture on each "tick" of a window resizing, should be fine on
            // any hardware from this century though i guess
            // (if this was an issue, we should debounce to limit how often this can happen and just
            // deal with the blurriness during active resizing, but we would have to make sure that
            // the texture is properly sized when the window size has settled)

            let vello_tex = make_texture(
                &self.device,
                "vello_target",
                VELLO_USAGE,
                tex_dims.0,
                tex_dims.1,
            );
            let vello_view = vello_tex.create_view(&wgpu::TextureViewDescriptor::default());

            let slint_tex = make_texture(
                &self.device,
                "slint_target",
                SLINT_USAGE,
                tex_dims.0,
                tex_dims.1,
            );
            let slint_view = slint_tex.create_view(&wgpu::TextureViewDescriptor::default());

            // should be fine to destroy? probably does not matter either way
            self.vello_tex.destroy();

            self.vello_tex = vello_tex;
            self.vello_view = vello_view;

            // i don't think i can destroy slint_tex, because slint probably still references it
            self.slint_tex = slint_tex;
            self.slint_view = slint_view;

            self.tex_dims = tex_dims;
        }

        self.vello_renderer
            .render_to_texture(
                &self.device,
                &self.queue,
                &self.combined_scene,
                &self.vello_view,
                &vello::RenderParams {
                    base_color: palette::css::FLORAL_WHITE,
                    width: tex_dims.0,
                    height: tex_dims.1,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .unwrap();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("blit"),
            });

        self.blitter.copy(
            &self.device,
            &mut encoder,
            &self.vello_view,
            &self.slint_view,
        );

        self.queue.submit(Some(encoder.finish()));
        slint::Image::try_from(self.slint_tex.clone()).unwrap()
    }

    fn build_tool_scene(&mut self, tool_stuff: &[PathPrimitive]) {
        self.tool_scene.reset();

        for p in tool_stuff {
            match p {
                PathPrimitive::Point(pos) => {
                    self.tool_scene.fill(
                        vello::peniko::Fill::NonZero,
                        kurbo::Affine::IDENTITY,
                        peniko::Color::from_rgba8(190, 30, 30, 180),
                        None,
                        &kurbo::Circle::new(*pos, 6.),
                    );
                }
                PathPrimitive::Line(p, q) => {
                    self.tool_scene.stroke(
                        &kurbo::Stroke::new(2.),
                        kurbo::Affine::IDENTITY,
                        peniko::Color::from_rgba8(190, 30, 30, 180),
                        None,
                        &kurbo::Line::new(*p, *q),
                    );
                }
                PathPrimitive::Curve(c) => {
                    self.tool_scene.stroke(
                        &kurbo::Stroke::new(2.),
                        kurbo::Affine::IDENTITY,
                        peniko::Color::from_rgba8(190, 30, 30, 180),
                        None,
                        &kurbo::CubicBez::new(c.p_0, c.p_1, c.p_2, c.p_3),
                    );
                }
            }
        }
    }

    fn build_main_scene(&mut self, arena: &draft::arena::Arena<Object>) {
        // TODO: extract magic constants
        // consider screen scale factor
        self.main_scene.reset();
        for (_, o, v) in arena.iter_triples() {
            let Some(v) = v else {
                continue;
            };
            match v {
                construction::Value::Point(p) => {
                    self.main_scene.fill(
                        vello::peniko::Fill::NonZero,
                        kurbo::Affine::IDENTITY,
                        peniko::Color::from_rgb8(30, 30, 30),
                        None,
                        &kurbo::Circle::new(p.pos, 6.),
                    );

                    match o {
                        Object::Point(PointObj::DistAngle { parent, .. }) => {
                            if let Some(parent_val) = arena.get_value_for::<PointObj>(*parent) {
                                self.main_scene.stroke(
                                    &kurbo::Stroke::new(1.).with_dashes(0., [6., 6.]),
                                    kurbo::Affine::IDENTITY,
                                    peniko::Color::from_rgba8(0, 0, 0, 200),
                                    None,
                                    &kurbo::Line::new(parent_val.pos, p.pos),
                                );
                            }
                        }
                        Object::Point(PointObj::OnLine { from, to, .. }) => {
                            let Some(&from_val) = arena.get_value_for::<PointObj>(*from) else {
                                continue;
                            };
                            let Some(&to_val) = arena.get_value_for::<PointObj>(*to) else {
                                continue;
                            };

                            // (avoiding overlapping dashed lines if the point is not on the line
                            // segment between its parent points)
                            // TODO: very weird usage of this function, needlessly confusing
                            let (_, t) =
                                geom::closest_point_on_beam(from_val.pos, to_val.pos, p.pos);

                            let (u, v) = if t < 0. {
                                (p.pos, to_val.pos)
                            } else if t > 1. {
                                (from_val.pos, p.pos)
                            } else {
                                (from_val.pos, to_val.pos)
                            };

                            self.main_scene.stroke(
                                &kurbo::Stroke::new(1.).with_dashes(0., [6., 6.]),
                                kurbo::Affine::IDENTITY,
                                peniko::Color::from_rgba8(0, 0, 0, 200),
                                None,
                                &kurbo::Line::new(u, v),
                            );
                        }
                        _ => {}
                    }
                }

                construction::Value::Line(l) => self.main_scene.stroke(
                    &kurbo::Stroke::new(2.),
                    kurbo::Affine::IDENTITY,
                    peniko::Color::from_rgba8(0, 0, 0, 200),
                    None,
                    &kurbo::Line::new(l.from, l.to),
                ),
                construction::Value::Curve(c) => self.main_scene.stroke(
                    &kurbo::Stroke::new(2.),
                    kurbo::Affine::IDENTITY,
                    peniko::Color::from_rgba8(0, 0, 0, 200),
                    None,
                    &kurbo::CubicBez::new(c.curve.p_0, c.curve.p_1, c.curve.p_2, c.curve.p_3),
                ),
                construction::Value::CurveControl(c) => {
                    self.main_scene.stroke(
                        &kurbo::Stroke::new(2.)
                            .with_dashes(0.0, [1., 6.])
                            .with_caps(kurbo::Cap::Round),
                        kurbo::Affine::IDENTITY,
                        peniko::Color::from_rgb8(100, 100, 100),
                        None,
                        &kurbo::Line::new(c.parent, c.pos),
                    );
                    self.main_scene.fill(
                        vello::peniko::Fill::NonZero,
                        kurbo::Affine::IDENTITY,
                        peniko::Color::from_rgb8(220, 50, 40),
                        None,
                        &kurbo::Rect::from_center_size(c.pos, (12., 12.)).to_rounded_rect(2.),
                    );
                }
                construction::Value::Path(p) => {
                    let path = kurbo::BezPath::from_iter(p.points.iter().enumerate().map(
                        |(i, e)| match e {
                            PathSementVal::Point(p) => {
                                if i == 0 {
                                    PathEl::MoveTo((*p).into())
                                } else {
                                    PathEl::LineTo((*p).into())
                                }
                            }
                            _ => panic!(),
                        },
                    ));

                    self.main_scene.fill(
                        vello::peniko::Fill::NonZero,
                        kurbo::Affine::IDENTITY,
                        peniko::Color::from_rgba8(200, 0, 0, 100),
                        None,
                        &path,
                    );
                }
                _ => {}
            }
        }
    }
}

fn make_texture(
    device: &wgpu::Device,
    label: &str,
    usage: wgpu::TextureUsages,
    w: u32,
    h: u32,
) -> wgpu::Texture {
    debug_assert_ne!(w, 0);
    debug_assert_ne!(h, 0);

    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage,
        view_formats: &[],
    })
}

fn texture_dimensions(logical_width: f32, logical_height: f32, scale_factor: f32) -> (u32, u32) {
    let actual_w = (logical_width as f64 * scale_factor as f64)
        .round()
        .max(1.0) as u32;
    let actual_h = (logical_height as f64 * scale_factor as f64)
        .round()
        .max(1.0) as u32;
    (actual_w, actual_h)
}
