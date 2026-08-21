use crate::construction::Entry;
use crate::geom::{self, CubicBezier, Point2};
use crate::wgpu;
use vello::{
    kurbo,
    peniko::{self, color::palette},
};
use wgpu::util::TextureBlitter;

use std::f64::consts::PI;

const VELLO_USAGE: wgpu::TextureUsages =
    wgpu::TextureUsages::STORAGE_BINDING.union(wgpu::TextureUsages::TEXTURE_BINDING);

const SLINT_USAGE: wgpu::TextureUsages =
    wgpu::TextureUsages::TEXTURE_BINDING.union(wgpu::TextureUsages::RENDER_ATTACHMENT);

#[derive(Debug, Clone, Copy)]
pub enum PathPrimitive {
    Line(Point2, Point2),
    Point(Point2),
    Curve(CubicBezier),
    Circle(Point2, f64),
}

pub struct Renderer {
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
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        texture_width: u32,
        texture_height: u32,
    ) -> Result<Self, vello::Error> {
        let v = vello::Renderer::new(
            &device,
            vello::RendererOptions {
                antialiasing_support: vello::AaSupport::area_only(),
                ..Default::default()
            },
        )?;

        let tex_dims = (texture_width, texture_height);

        let blitter = TextureBlitter::new(&device, wgpu::TextureFormat::Rgba8Unorm);

        let vello_tex = make_texture(&device, "vello_target", VELLO_USAGE, tex_dims.0, tex_dims.1);

        let vello_view = vello_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let slint_tex = make_texture(&device, "slint_target", SLINT_USAGE, tex_dims.0, tex_dims.1);

        let slint_view = slint_tex.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
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
        })
    }

    pub fn render(
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

    pub fn build_tool_scene(&mut self, tool_stuff: &[PathPrimitive]) {
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

                    let first_half = c.split_at(0.5).0;
                    // not actually the mid-point by arc length
                    // but probably close enough
                    let mid = first_half.at(1.);
                    let local_ang = first_half.at(0.95).angle(mid);

                    let arrow_arm_len = 10.;

                    let left_ang = local_ang - 0.75 * PI;
                    let right_ang = local_ang + 0.75 * PI;

                    let left = mid + geom::polar(arrow_arm_len, left_ang);
                    let right = mid + geom::polar(arrow_arm_len, right_ang);

                    self.tool_scene.stroke(
                        &kurbo::Stroke::new(2.),
                        kurbo::Affine::IDENTITY,
                        peniko::Color::from_rgba8(190, 30, 30, 180),
                        None,
                        &kurbo::Line::new(mid, left),
                    );
                    self.tool_scene.stroke(
                        &kurbo::Stroke::new(2.),
                        kurbo::Affine::IDENTITY,
                        peniko::Color::from_rgba8(190, 30, 30, 180),
                        None,
                        &kurbo::Line::new(mid, right),
                    );
                }
                PathPrimitive::Circle(center, radius) => {
                    self.tool_scene.stroke(
                        &kurbo::Stroke::new(2.),
                        kurbo::Affine::IDENTITY,
                        peniko::Color::from_rgba8(190, 30, 30, 180),
                        None,
                        &kurbo::Circle::new(*center, *radius),
                    );
                }
            }
        }
    }

    pub fn build_main_scene<'a>(&mut self, geometry: impl Iterator<Item = &'a Entry>) {
        // TODO: extract magic constants
        // consider screen scale factor
        self.main_scene.reset();

        let default_col = peniko::Color::from_rgb8(30, 30, 30);
        let default_stroke_width = 2.;
        let node_radius = 6f64;

        for g in geometry {
            match g {
                Entry::PointFree(_, _, p) => {
                    self.main_scene.fill(
                        vello::peniko::Fill::NonZero,
                        kurbo::Affine::IDENTITY,
                        default_col,
                        None,
                        &kurbo::Circle::new(p.pos, node_radius),
                    );
                }
                Entry::PointOnCurve(_, _, p) => {
                    self.main_scene.fill(
                        vello::peniko::Fill::NonZero,
                        kurbo::Affine::IDENTITY,
                        default_col,
                        None,
                        &kurbo::Circle::new(p.pos, node_radius),
                    );
                }
                Entry::PointOnLine(_, _, p) => {
                    self.main_scene.fill(
                        vello::peniko::Fill::NonZero,
                        kurbo::Affine::IDENTITY,
                        default_col,
                        None,
                        &kurbo::Circle::new(p.pos, node_radius),
                    );
                    self.main_scene.stroke(
                        &kurbo::Stroke::new(default_stroke_width).with_dashes(0., [6., 6.]),
                        kurbo::Affine::IDENTITY,
                        default_col,
                        None,
                        &kurbo::Line::new(p.from, p.to),
                    );
                }
                Entry::PointDistAngle(_, _, p) => {
                    self.main_scene.fill(
                        vello::peniko::Fill::NonZero,
                        kurbo::Affine::IDENTITY,
                        default_col,
                        None,
                        &kurbo::Circle::new(p.pos, node_radius),
                    );
                    self.main_scene.stroke(
                        &kurbo::Stroke::new(default_stroke_width).with_dashes(0., [6., 6.]),
                        kurbo::Affine::IDENTITY,
                        default_col,
                        None,
                        &kurbo::Line::new(p.parent, p.pos),
                    );
                }
                Entry::Line(_, _, l) => self.main_scene.stroke(
                    &kurbo::Stroke::new(default_stroke_width),
                    kurbo::Affine::IDENTITY,
                    default_col,
                    None,
                    &kurbo::Line::new(l.from, l.to),
                ),
                Entry::Curve(_, _, c) => {
                    self.main_scene.stroke(
                        &kurbo::Stroke::new(default_stroke_width),
                        kurbo::Affine::IDENTITY,
                        default_col,
                        None,
                        &kurbo::CubicBez::new(c.curve.p_0, c.curve.p_1, c.curve.p_2, c.curve.p_3),
                    );

                    let first_half = c.curve.split_at(0.5).0;
                    // not actually the mid-point by arc length
                    // but probably close enough
                    let mid = first_half.at(1.);
                    let local_ang = first_half.at(0.95).angle(mid);

                    let arrow_arm_len = 10.;

                    let left_ang = local_ang - 0.75 * PI;
                    let right_ang = local_ang + 0.75 * PI;

                    let left = mid + geom::polar(arrow_arm_len, left_ang);
                    let right = mid + geom::polar(arrow_arm_len, right_ang);

                    self.main_scene.stroke(
                        &kurbo::Stroke::new(default_stroke_width),
                        kurbo::Affine::IDENTITY,
                        default_col,
                        None,
                        &kurbo::Line::new(mid, left),
                    );
                    self.main_scene.stroke(
                        &kurbo::Stroke::new(default_stroke_width),
                        kurbo::Affine::IDENTITY,
                        default_col,
                        None,
                        &kurbo::Line::new(mid, right),
                    );
                }
                Entry::CurveControl(_, _, c) => {
                    self.main_scene.stroke(
                        &kurbo::Stroke::new(default_stroke_width)
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
                        peniko::Color::from_rgba8(220, 50, 40, 180),
                        None,
                        &kurbo::Rect::from_center_size(c.pos, (2. * node_radius, 2. * node_radius))
                            .to_rounded_rect(2.),
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

pub fn texture_dimensions(
    logical_width: f32,
    logical_height: f32,
    scale_factor: f32,
) -> (u32, u32) {
    let actual_w = (logical_width as f64 * scale_factor as f64)
        .round()
        .max(1.0) as u32;
    let actual_h = (logical_height as f64 * scale_factor as f64)
        .round()
        .max(1.0) as u32;
    (actual_w, actual_h)
}
