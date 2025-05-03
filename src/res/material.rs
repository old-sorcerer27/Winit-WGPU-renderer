use wgpu::{Device, BindGroupLayout, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource};

use super::{storage::Storage, texture::{Texture, TextureHandle}, Handle};


#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub base_color: [f32; 4],
    pub base_color_texture: Option<TextureHandle>,
    pub metallic: f32,
    pub roughness: f32,
    pub bind_group: Option<wgpu::BindGroup>, // GPU-представление
}


impl super::Resource for Material {
    type Key = super::MaterialKey;
    type LoadParams = Material;
    
    fn load(params: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized {
        todo!()
    }
}

pub type MaterialStorage = Storage<Material>;
pub type MaterialHandle = Handle<Material>;


impl Material {
    pub fn default() -> Self {
        Self {
            name: "Default".to_string(),
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_color_texture: None,
            metallic: 0.0,
            roughness: 1.0,
            bind_group: todo!(),
        }
    }
    
    pub fn new(
        name: &str,
        base_color: [f32; 4],
        base_color_texture: Option<TextureHandle>,
        metallic: f32,
        roughness: f32,
    ) -> Self {
        Self {
            name: name.to_string(),
            base_color,
            base_color_texture,
            metallic,
            roughness,
            bind_group: None,
        }
    }

    pub fn create_bind_group(
        &mut self,
        device: &Device,
        layout: &BindGroupLayout,
        texture_view: Option<&wgpu::TextureView>,
        sampler: Option<&wgpu::Sampler>,
    ) {
        let mut entries = vec![];

        // Добавляем текстуру и сэмплер, если есть
        if let (Some(view), Some(sampler)) = (texture_view, sampler) {
            entries.push(BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(view),
            });
            entries.push(BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(sampler),
            });
        }

        if !entries.is_empty() {
            self.bind_group = Some(device.create_bind_group(&BindGroupDescriptor {
                label: Some(&format!("{}_bind_group", self.name)),
                layout,
                entries: &entries,
            }));
        }
    }

 
    pub fn from_gltf(
        material: &gltf::Material,
        textures: &[Handle<Texture>],
    ) -> Self {
        let pbr = material.pbr_metallic_roughness();
        
        Self {
            name: material.name().unwrap_or("Unnamed").to_string(),
            base_color: pbr.base_color_factor(),
            base_color_texture: pbr.base_color_texture()
                .and_then(|tex| textures.get(tex.texture().index()))
                .cloned(),
            metallic: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            bind_group: None,
        }
    }
    
}