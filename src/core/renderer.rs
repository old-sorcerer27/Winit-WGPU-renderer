use std::collections::HashMap;

use gltf::{Buffer, Scene};

use crate::res::{asset::AssetManager, mesh::Mesh, Handle};

use super::PipelineType;

pub struct Renderer<'a> {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'a>,
    pub vertex_buffer: wgpu::Buffer,
    assets: AssetManager,
    pipelines: HashMap<PipelineType, wgpu::RenderPipeline>,
}

impl<'a> Renderer<'a> {
    pub fn render_scene(&mut self, scene: &Scene) {
        // Проход рендеринга
    }

    pub fn render_mesh(&mut self, mesh_handle: Handle<Mesh>) {
        if let Some(mesh) = self.assets.meshes.get(mesh_handle) {
            // Рендерим меш
            self.queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&mesh.vertices),
            );
        }
    }
}