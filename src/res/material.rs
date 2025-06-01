use std::{error::Error, fmt};

use wgpu::{Device, BindGroupLayout, BindGroupDescriptor, BindGroupEntry, BindingResource};
use super::{storage::Storage, texture::{get_texture_bind_group_layout, GpuTexture, GpuTextureHandle}, Handle};

/// Материал для рендеринга, содержащий параметры PBR и текстуры
#[derive(Debug, Clone)]
pub struct Material {
    /// Название материала
    pub name: String,
    /// Базовый цвет (RGBA)
    pub base_color: [f32; 4],
    /// Текстура базового цвета
    pub base_color_texture: Option<GpuTextureHandle>,
    /// Металличность (0.0 - 1.0)
    pub metallic: f32,
    /// Шероховатость (0.0 - 1.0)
    pub roughness: f32,
    /// Bind group для шейдера
    pub bind_group: Option<wgpu::BindGroup>,
}

impl super::Resource for Material {
    type Key = super::MaterialKey;
    type LoadParams = Material;
    
    /// Загружает материал из параметров
    fn load(params: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(params) // Просто возвращаем готовый материал
    }
}

/// Типы для удобной работы с хранилищем материалов
pub type MaterialStorage = Storage<Material>;
pub type MaterialHandle = Handle<Material>;

impl Material {
    /// Создаёт bind group для материала
    ///
    /// # Аргументы
    /// * `device` - GPU устройство
    /// * `layout` - Layout bind group
    /// * `texture_view` - Вью текстуры (если есть)
    /// * `sampler` - Сэмплер (если есть)
    pub fn create_bind_group(
        &mut self,
        device: &Device,
        layout: Option<&BindGroupLayout>,
        texture_view: Option<&wgpu::TextureView>,
        sampler: Option<&wgpu::Sampler>,
    ) {
        let mut entries = Vec::with_capacity(2); 

        if let (Some(view), Some(sampler)) = (texture_view, sampler) {
            entries.extend([
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                }
            ]);
        }

        match layout {
            Some(layout) => {if !entries.is_empty() {
                self.bind_group = Some(device.create_bind_group(&BindGroupDescriptor {
                    label: Some(&format!("{}_bind_group", self.name)),
                    layout,
                    entries: &entries,
                }));
            }},
            None => {
                let layout = get_texture_bind_group_layout(device);
                self.bind_group = Some(device.create_bind_group(&BindGroupDescriptor {
                    label: Some(&format!("{}_bind_group", self.name)),
                    layout: &layout,
                    entries: &entries,
                }));
            },
        }
    }

    /// Создаёт материал из GLTF материала с указанной текстурой
    pub fn from_gltf_texture(
        material: &gltf::Material,
        texture: Handle<GpuTexture>
    ) -> Self {
        let pbr = material.pbr_metallic_roughness();
        Self {
            name: material.name().unwrap_or_default().to_string(),
            base_color: pbr.base_color_factor(),
            base_color_texture: Some(texture),
            metallic: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            bind_group: None,
        }
    }
 
    /// Создаёт материал из GLTF материала, выбирая текстуру из списка
    pub fn from_gltf_material(
        material: &gltf::Material,
        textures: &[Handle<GpuTexture>],
    ) -> Self {
        let pbr = material.pbr_metallic_roughness();
        Self {
            name: material.name().unwrap_or_default().to_string(),
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


#[derive(Debug)]
pub struct LoadMaterialError {
    message: String,
}

impl fmt::Display for LoadMaterialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Material loading error: {}", self.message)  // Исправлено сообщение
    }
}

impl Error for LoadMaterialError {}

impl LoadMaterialError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}


