use crate::res::texture::Texture;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub center: glam::Vec4,  // 0 byte offset
    pub radius: f32,        // 16 byte offset
    pub material_idx: u32,  // 20 byte offset
    pub _padding: [u32; 2], // 24 byte offset, 8 bytes size
}

impl Sphere {
    pub fn new(center: glam::Vec3, radius: f32, material_idx: u32) -> Self {
        Self {
            center: center.extend(1.0),
            radius,
            material_idx,
            _padding: [0_u32; 2],
        }
    }
}
