use glam::{Mat4, Quat, Vec3};
use wgpu::{util::DeviceExt, Buffer, BufferUsages};

use crate::scene::camera::Camera;


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub position: Vec3,  
    _padding1: f32,           
    pub rotation: Quat,  
    pub scale: Vec3,     
    _padding2: f32,           
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            _padding1: 0.0,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            _padding2: 0.0,
        }
    }
}

impl Transform {

    pub fn new(
        position: glam::Vec3, 
        rotation: glam::Quat, 
        scale: glam::Vec3,
    ) -> Self {
        Self {
            position,
            _padding1: 0.0,
            rotation,
            scale,
            _padding2: 0.0,
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * -Vec3::Z
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn rotate(&mut self, delta: glam::Vec3) {
        let rotation = glam::Quat::from_euler(
            glam::EulerRot::YXZ,
            delta.y,
            delta.x,
            delta.z,
        );
        self.rotation = (rotation * self.rotation).normalize();
    }

    // pub fn axis_translate(&mut self, translation: glam::Vec3) {
    //     self.position += translation;
    // }

    // pub fn direct_translate(&mut self, translation: glam::Vec3) {
    //     self.position += self.rotation * translation;
    // }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(
            self.position,
            self.forward(),
            self.up(),
        )
    }

    pub fn calculate_view_projection(&self, camera: &Camera) -> Mat4 {
        let view = self.view_matrix();
        
        let proj = Mat4::perspective_rh(
            camera.fov.to_radians(),
            camera.aspect,
            camera.near,
            camera.far,
        );
        
        proj * view
    }


    pub fn create_buffer(
        device: &wgpu::Device, 
        transform: Transform
    )-> Buffer{
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("transform_buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    &transform as *const _ as *const u8,
                    std::mem::size_of::<Transform>()
                )
            },
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

}
