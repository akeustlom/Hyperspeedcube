//! Structs shared between the CPU and GPU (vertices, uniforms, etc.).

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct GfxProjectionParams {
    pub facet_scale: f32,
    pub sticker_scale: f32,
    pub w_factor_4d: f32,
    pub w_factor_3d: f32,
    pub fov_signum: f32,
}
