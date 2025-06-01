use std::ops::Range;

use wgpu::{hal::Device, util::DeviceExt, wgc::device};

use crate::res::{material::Material, mesh::Mesh, model::Model, Handle};

use super::transform::Transform;

pub enum SceneEntityKind {
    Object {
        model: Model
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
    // pub fn new_object_from_model<T: Device>(
    //     model: Model,
    //     file_name: String,
    //     device: &wgpu::Device,) -> Self {
    //         let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             label: Some(&format!("{:?} Vertex Buffer", file_name)),
    //             contents: bytemuck::cast_slice(&vertices),
    //             usage: wgpu::BufferUsages::VERTEX,
    //         });
    //         let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             label: Some(&format!("{:?} Index Buffer", file_name)),
    //             contents: bytemuck::cast_slice(&indices),
    //             usage: wgpu::BufferUsages::INDEX,
    //         });
    //     Self {
    //         kind: SceneEntityKind::Object {vertex_buffer: todo!(), index_buffer: todo!() },
    //         transform: Transform::default(),
    //         visible: true,
    //     }
    // }

    pub fn new_camera(fov: f32, near: f32, far: f32) -> Self {
        Self {
            kind: SceneEntityKind::Camera { fov, near, far },
            transform: Transform::default(),
            visible: false,
        }
    }
}

pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

// impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
// where
//     'b: 'a,
// {
//     fn draw_mesh(
//         &mut self,
//         mesh: &'b Mesh,
//         material: &'b Material,
//         camera_bind_group: &'b wgpu::BindGroup,
//         light_bind_group: &'b wgpu::BindGroup,
//     ) {
//         self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group, light_bind_group);
//     }

//     fn draw_mesh_instanced(
//         &mut self,
//         mesh: &'b Mesh,
//         material: &'b Material,
//         instances: Range<u32>,
//         camera_bind_group: &'b wgpu::BindGroup,
//         light_bind_group: &'b wgpu::BindGroup,
//     ) {
//         self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
//         self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
//         self.set_bind_group(0, &material.bind_group, &[]);
//         self.set_bind_group(1, camera_bind_group, &[]);
//         self.set_bind_group(2, light_bind_group, &[]);
//         self.draw_indexed(0..mesh.num_elements, 0, instances);
//     }

//     fn draw_model(
//         &mut self,
//         model: &'b Model,
//         camera_bind_group: &'b wgpu::BindGroup,
//         light_bind_group: &'a wgpu::BindGroup,
//     ) {
//         self.draw_model_instanced(model, 0..1, camera_bind_group, light_bind_group);
//     }

//     fn draw_model_instanced(
//         &mut self,
//         model: &'b Model,
//         instances: Range<u32>,
//         camera_bind_group: &'b wgpu::BindGroup,
//         light_bind_group: &'b wgpu::BindGroup,
//     ) {
//         for mesh in &model.meshes {
//             let material = &model.materials[mesh.material];
//             self.draw_mesh_instanced(
//                 mesh,
//                 material,
//                 instances.clone(),
//                 camera_bind_group,
//                 light_bind_group,
//             );
//         }
//     }
// }