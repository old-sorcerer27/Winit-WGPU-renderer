use glam::{Mat4, Vec3, Vec4};
use gltf::buffer;
use wgpu::{util::DeviceExt, wgc::device::queue, BindGroupLayout, Buffer, Device, Queue};

use crate::scene::transform::{self, Transform};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, view_proj: Mat4) {
        self.view_proj = view_proj.to_cols_array_2d();
        println!("\n {:?}",  view_proj);
    }

    pub fn create_buffer(
        device: &wgpu::Device, 
        uniform: CameraUniform
    )-> Buffer{
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn update_buffer(self, queue: &Queue, buffer: &Buffer){
        queue.write_buffer(
            &buffer,
            0,
            bytemuck::cast_slice(&[self]),
        );
    }
}

// #[repr(C)]
// #[derive(Debug, Copy, Clone)]
// pub struct CameraUniform {
//     pub view_proj: Mat4,
//     pub view_position: Vec4,
// }

// impl CameraUniform {
//     pub fn new() -> Self {
//         Self {
//             view_position: [0.0; 4].into(),
//             view_proj: Mat4::IDENTITY,
//         }
//     }

//     pub fn update_view_proj(&mut self, camera: &Camera, camera_transform: &mut Transform) {
//         self.view_proj = camera.projection_matrix() * camera_transform.view_matrix();
//         self.view_position = camera_transform.position.extend(1.);      
//     }


//     pub fn create_buffer(
//         device: &wgpu::Device, 
//         uniform: CameraUniform
//     )-> Buffer{
//         device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("camera_uniform_buffer"),
//             contents: unsafe {
//                 std::slice::from_raw_parts(
//                     &uniform as *const _ as *const u8,
//                     std::mem::size_of::<CameraUniform>()
//                 )
//             },
//             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//         })
//     }
// }

pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl Camera {
    pub fn create_bind_group(
        &mut self,
        device: &Device,
        layout: &BindGroupLayout,
        buffer: Buffer
    ) {
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        }));
        
    }

    pub fn new(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            fov,
            near,
            far,
            aspect,
            bind_group: None,
        }
    }
    
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    
}

pub fn get_camera_bind_group_layout(
    device: &wgpu::Device,
) -> BindGroupLayout {
    return 
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("camera_bind_group_layout"),
    });
}
