pub mod entity;
pub mod transform;
pub mod camera;
pub mod light;

use std::collections::HashMap;
use entity::SceneEntity;
use crate::res::{material::Material, mesh::Mesh, model::Model, texture::gpu_texture::GpuTexture, Handle};


#[derive(Default)]
pub struct AppScene {
    pub entities: HashMap<String, SceneEntity>,
    pub active_camera: String,
    pub lights: Vec<winit::window::Theme>,
    pub ambient_light: [f32; 3],
    pub skybox: Option<Handle<GpuTexture>>,
}


impl AppScene {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            active_camera: "main_camera".to_string(),
            lights: Vec::new(),
            ambient_light: [0.1, 0.1, 0.1],
            skybox: None,
        }
    }
      
    /// Добавляет объект в сцену
    pub fn add_entity(&mut self, name: &str, object: SceneEntity) {
        self.entities.insert(name.to_string(), object);
    }

    /// Устанавливает активную камеру
    pub fn set_active_camera(&mut self, name: &str) {
        self.active_camera = name.to_string();
    }

    /// Возвращает трансформ камеры
    // pub fn get_camera_transform(&self) -> Option<&Transform> {
    //     self.entities.get(&self.active_camera)
    //         .and_then(|obj| match &obj.kind {
    //             // SceneObjectKind::Camera => Some(&obj.transform),
    //             SceneEntityKind::Camera { fov: _, near: _, far: _ } => Some(&obj.transform),
    //             _ => None,
    //         })
    // }

    pub fn update(&mut self, delta_time: f32) {

    }
}




// pub fn get_depth_stencil_attachment (
//     depth_texture: Option<&GpuTexture>
// )-> RenderPassDepthStencilAttachment<'static> {
//     match depth_texture {
//         Some(tex) => {
//             let dsa: RenderPassDepthStencilAttachment<'_> = wgpu::RenderPassDepthStencilAttachment {
//             view: &tex.view,
//             depth_ops: Some(wgpu::Operations {
//                 load: wgpu::LoadOp::Clear(1.0),
//                 store: wgpu::StoreOp::Discard
//             }),
//                stencil_ops: None,
//             };
//             return dsa;
//         },
//         None => todo!(),
//     }
   
// }


pub trait Draw<'a> {
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
        instances: std::ops::Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_light_mesh(
        &mut self,
        mesh: &'a Mesh,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: std::ops::Range<u32>,
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
        instances: std::ops::Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

// impl<'a, 'b> Draw<'b> for wgpu::RenderPass<'a>
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
//         self.draw_indexed(0..mesh.indices.len() as u32, 0, instances);
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
//             let material = &mesh.material;
//             self.draw_mesh_instanced(
//                 mesh,
//                 material,
//                 instances.clone(),
//                 camera_bind_group,
//                 light_bind_group,
//             );
//         }
//     }

//     fn draw_light_mesh(
//         &mut self,
//         mesh: &'b Mesh,
//         camera_bind_group: &'b wgpu::BindGroup,
//         light_bind_group: &'b wgpu::BindGroup,
//     ) {
//         self.draw_light_mesh_instanced(mesh, 0..1, camera_bind_group, light_bind_group);
//     }
    
//     fn draw_light_mesh_instanced(
//         &mut self,
//         mesh: &'b Mesh,
//         instances: Range<u32>,
//         camera_bind_group: &'b wgpu::BindGroup,
//         light_bind_group: &'b wgpu::BindGroup,
//     ) {
//         self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
//         self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
//         self.set_bind_group(0, camera_bind_group, &[]);
//         self.set_bind_group(1, light_bind_group, &[]);
//         self.draw_indexed(0..mesh.indices.len() as u32, 0, instances);
//     } 
// }

