use std::{error::Error, fmt};

use wgpu::{Device, BindGroupLayout, BindGroupDescriptor, BindGroupEntry, BindingResource};
use crate::res::texture::{gpu_texture::{get_texture_bind_group_layout, GpuTexture, GpuTextureHandle}, Texture, TextureDescriptor};

use super::{storage::Storage, Handle};

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


#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuMaterial {
    id: u32,
    desc1: TextureDescriptor,
    desc2: TextureDescriptor,
    x: f32,
}

impl GpuMaterial {
    pub fn lambertian(albedo: &Texture, global_texture_data: &mut Vec<[f32; 3]>) -> Self {
        Self {
            id: 0_u32,
            desc1: Self::append_to_global_texture_data(albedo, global_texture_data),
            desc2: TextureDescriptor::empty(),
            x: 0_f32,
        }
    }

    pub fn metal(albedo: &Texture, fuzz: f32, global_texture_data: &mut Vec<[f32; 3]>) -> Self {
        Self {
            id: 1_u32,
            desc1: Self::append_to_global_texture_data(albedo, global_texture_data),
            desc2: TextureDescriptor::empty(),
            x: fuzz,
        }
    }

    pub fn dielectric(refraction_index: f32) -> Self {
        Self {
            id: 2_u32,
            desc1: TextureDescriptor::empty(),
            desc2: TextureDescriptor::empty(),
            x: refraction_index,
        }
    }

    pub fn checkerboard(
        even: &Texture,
        odd: &Texture,
        global_texture_data: &mut Vec<[f32; 3]>,
    ) -> Self {
        Self {
            id: 3_u32,
            desc1: Self::append_to_global_texture_data(even, global_texture_data),
            desc2: Self::append_to_global_texture_data(odd, global_texture_data),
            x: 0_f32,
        }
    }

    pub fn emissive(emit: &Texture, global_texture_data: &mut Vec<[f32; 3]>) -> Self {
        Self {
            id: 4_u32,
            desc1: Self::append_to_global_texture_data(emit, global_texture_data),
            desc2: TextureDescriptor::empty(),
            x: 0_f32,
        }
    }

    fn append_to_global_texture_data(
        texture: &Texture,
        global_texture_data: &mut Vec<[f32; 3]>,
    ) -> TextureDescriptor {
        let dimensions = texture.dimensions();
        let offset = global_texture_data.len() as u32;
        global_texture_data.extend_from_slice(texture.as_slice());
        TextureDescriptor {
            width: dimensions.0,
            height: dimensions.1,
            offset,
        }
    }
}


pub enum RayCastMaterial {
    Lambertian { albedo: Texture },
    Metal { albedo: Texture, fuzz: f32 },
    Dielectric { refraction_index: f32 },
    Checkerboard { even: Texture, odd: Texture },
    Emissive { emit: Texture },
}
