pub mod angle;
pub mod sphere;

pub fn unit_quad_projection_matrix() -> nalgebra_glm::Mat4 {
    let sw = 0.5_f32;
    let sh = 0.5_f32;

    // Our ortho camera is just centered at (0, 0)

    let left = -sw;
    let right = sw;
    let bottom = -sh;
    let top = sh;

    // DirectX, Metal, wgpu share the same left-handed coordinate system
    // for their normalized device coordinates:
    // https://github.com/gfx-rs/gfx/tree/master/src/backend/dx12
    // Mat4::orthographic_lh(left, right, bottom, top, -1_f32, 1_f32)
    nalgebra_glm::ortho_lh_zo(left, right, bottom, top, -1_f32, 1_f32)
}