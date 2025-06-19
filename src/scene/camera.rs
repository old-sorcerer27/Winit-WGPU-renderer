use glam::{Mat4, Vec3, Vec4};
use gltf::buffer;
use wgpu::{util::DeviceExt, wgc::device::queue, BindGroupLayout, Buffer, Device, Queue};

use crate::{math::angle::Angle, scene::transform::{self, Transform}};

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
        println!("{:?}", view_proj);
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

#[derive(Clone, PartialEq)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
    pub rccp: Option<RayCastCameraParams>,
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

    pub fn new(fov: f32, aspect: f32, near: f32, far: f32, rccp: Option<RayCastCameraParams>) -> Self {
        Self {
            fov,
            near,
            far,
            aspect,
            rccp,
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

#[derive(Clone, Copy, PartialEq)]
pub struct RayCastCameraParams{
    pub vfov: Angle,
    pub aperture: f32,
    pub focus_distance: f32,
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GpuCamera {
    eye: glam::Vec3,
    _padding1: f32,
    horizontal: glam::Vec3,
    _padding2: f32,
    vertical: glam::Vec3,
    _padding3: f32,
    u: glam::Vec3,
    _padding4: f32,
    v: glam::Vec3,
    lens_radius: f32,
    lower_left_corner: glam::Vec3,
    _padding5: f32,
}

impl GpuCamera {
    pub fn new(camera: &Camera, camera_transform: &Transform) -> Self {
        let (lens_radius, focus_distance) = match &camera.rccp {
            Some(rccp) => (0.5 * rccp.aperture, rccp.focus_distance),
            None => (0.0, 1.0), 
        };

        let theta = camera.fov.to_radians();
        let half_height = focus_distance * (0.5 * theta).tan();
        let half_width = camera.aspect * half_height;

        let w = camera_transform.forward().normalize();
        let v = camera_transform.up().normalize();
        let u = w.cross(v);

        let lower_left_corner = camera_transform.position 
            + focus_distance * w 
            - half_width * u 
            - half_height * v;
        
        let horizontal = 2.0 * half_width * u;
        let vertical = 2.0 * half_height * v;

        Self {
            eye: camera_transform.position,
            _padding1: 0.0,
            horizontal,
            _padding2: 0.0,
            vertical,
            _padding3: 0.0,
            u,
            _padding4: 0.0,
            v,
            lens_radius,
            lower_left_corner,
            _padding5: 0.0,
        }
    }

    pub fn create_buffer(
        device: &wgpu::Device, 
        gpu_camera: GpuCamera
    )-> Buffer{
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gpu_camera_buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    &gpu_camera as *const _ as *const u8,
                    std::mem::size_of::<Transform>()
                )
            },
            usage: wgpu::BufferUsages::VERTEX,
        })
    }
}

