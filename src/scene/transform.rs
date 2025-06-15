use glam::{Mat4, Quat, Vec3};
use wgpu::{util::DeviceExt, Buffer, BufferUsages};


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

    pub fn view_matrix(&self) -> Mat4 {
        let rotation_matrix = Mat4::from_quat(self.rotation);
        let translation_matrix = Mat4::from_translation(-self.position);
        rotation_matrix * translation_matrix
    }

    pub fn calculate_view_projection(&self, aspect: f32) -> Mat4 {
        let projection = Mat4::perspective_rh(
            45.0f32.to_radians(),
            aspect,
            0.1,
            100.0,
        );
        let view = self.view_matrix();
        projection * view
    }

    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn rotate_camera(&mut self, delta_pitch: f32, delta_yaw: f32) {
        let pitch = Quat::from_rotation_x(delta_pitch);
        let yaw = Quat::from_rotation_y(delta_yaw);
        self.rotation = self.rotation * yaw * pitch;
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
