// should already fail to compile elsewhere
static_assertions::assert_type_eq_all!(slint::wgpu_29::wgpu::Device, vello::wgpu::Device);
pub use slint::wgpu_29::wgpu;

pub mod construction;
pub mod expression;
pub mod geom;
pub mod render;
pub mod slint_conv;
pub mod slint_gen;
pub mod tool;
