use glam::{Mat4, Vec3, Vec4};
use wgpu::{util::DeviceExt, BindGroupDescriptor, BindGroupLayout, Buffer, Device};


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LightUniform {
    view_position: Vec4,
    view_proj: Mat4,
}

impl LightUniform {
    pub fn create_buffer(
        device: &wgpu::Device, 
        uniform: LightUniform
    )-> Buffer{
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_uniform_buffer"),
            contents: unsafe {
                std::slice::from_raw_parts(
                    &uniform as *const _ as *const u8,
                    std::mem::size_of::<LightUniform>()
                )
            },
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}


pub enum LightType {
    Directional,
    Point {
        range: f32,
    },
    Spot {
        angle: f32,
        range: f32,
    },
}

pub struct Light {
    pub light_type: LightType,
    pub color: Vec3,
    pub shadows_enabled: bool,
    pub intensity: f32,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl Light {
     pub fn new(light_type: LightType, color: Vec3, shadows_enabled: bool, intensity: f32) -> Self {
        Self {
            light_type,
            color,
            shadows_enabled,
            intensity,
            bind_group: None,
        }
    }

    pub fn create_bind_group(
        &mut self,
        device: &Device,
        layout: Option<&BindGroupLayout>,
        buffer: Buffer
    ) {
        match layout {
            Some(layout) => {
                self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                    label: Some("light_bind_group"),
                }));
            },
            None => {
                let layout = get_light_bind_group_layout(device);
                self.bind_group = Some(device.create_bind_group(&BindGroupDescriptor {
                    layout: &layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                    label: Some("light_bind_group"),
                }));
            },
        }  
    }
    
}

pub fn get_light_bind_group_layout(
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
