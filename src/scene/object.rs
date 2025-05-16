use wgpu::{hal::Device, util::DeviceExt, wgc::device};

use crate::res::{material::Material, mesh::Mesh, model::Model, Handle};

use super::transform::Transform;

pub enum SceneEntityKind {
    Object {
        vertex_buffer: wgpu::Buffer,
        index_buffer: wgpu::Buffer,
    },
    Camera {
        fov: f32,
        near: f32,
        far: f32,
    },
    Light,
    Empty,
}

pub struct SceneEntity {
    pub kind: SceneEntityKind,
    pub transform: Transform,
    pub visible: bool,
}

impl SceneEntity {
    pub fn new_object_from_model<T: Device>(
        model: Model,
        file_name: String,
        device: &wgpu::Device,) -> Self {
            // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            //     label: Some(&format!("{:?} Vertex Buffer", file_name)),
            //     contents: bytemuck::cast_slice(&vertices),
            //     usage: wgpu::BufferUsages::VERTEX,
            // });
            // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            //     label: Some(&format!("{:?} Index Buffer", file_name)),
            //     contents: bytemuck::cast_slice(&indices),
            //     usage: wgpu::BufferUsages::INDEX,
            // });

        Self {
            kind: SceneEntityKind::Object {vertex_buffer: todo!(), index_buffer: todo!() },
            transform: Transform::default(),
            visible: true,
        }
    }

     pub fn new_object_from_mesh<T: Device>(
        mesh: Mesh,
        file_name: String,
        device: &wgpu::Device,) -> Self {
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        Self {
            kind: SceneEntityKind::Object {
                vertex_buffer: vertex_buffer, 
                index_buffer: index_buffer 
            },
            transform: Transform::default(),
            visible: true,
        }
    }

    pub fn new_camera(fov: f32, near: f32, far: f32) -> Self {
        Self {
            kind: SceneEntityKind::Camera { fov, near, far },
            transform: Transform::default(),
            visible: false,
        }
    }
}

pub trait Renderable {    
    fn render() -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

