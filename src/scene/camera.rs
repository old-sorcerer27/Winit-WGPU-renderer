use glam::{Mat4, Vec3};

use super::transform::Transform;

pub struct Camera {
    pub transform: Transform,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
}

impl Camera {
    pub fn new(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            transform: Transform::default(),
            fov,
            near,
            far,
            aspect,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(
            self.transform.position,
            self.transform.position + self.transform.rotation * Vec3::NEG_Z,
            self.transform.rotation * Vec3::Y,
        )
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.near, self.far)
    }
}