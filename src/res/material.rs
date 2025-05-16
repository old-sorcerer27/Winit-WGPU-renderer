use std::{error::Error, fmt};

use wgpu::{Device, BindGroupLayout, BindGroupDescriptor, BindGroupEntry, BindingResource};
use super::{storage::Storage, texture::{GpuTexture, GpuTextureHandle}, Handle};

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
        layout: &BindGroupLayout,
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

        if !entries.is_empty() {
            self.bind_group = Some(device.create_bind_group(&BindGroupDescriptor {
                label: Some(&format!("{}_bind_group", self.name)),
                layout,
                entries: &entries,
            }));
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


// #[cfg(test)]
// mod tests {
//     use super::*;
//     use gltf::Material as GltfMaterial;
//     use wgpu::{
//         DeviceDescriptor, Features, Limits, Instance, PowerPreference, Queue, RequestAdapterOptions,
//     };

//     // Вспомогательная функция для создания тестового устройства
//     async fn create_test_device() -> (Device, Queue) {
//         let instance = Instance::new(wgpu::Backends::all());
//         let adapter = instance
//             .request_adapter(&RequestAdapterOptions {
//                 power_preference: PowerPreference::LowPower,
//                 compatible_surface: None,
//                 force_fallback_adapter: false,
//             })
//             .await
//             .unwrap();
        
//         adapter
//             .request_device(
//                 &DeviceDescriptor {
//                     features: Features::empty(),
//                     limits: Limits::default(),
//                     label: None,
//                     required_features: todo!(),
//                     required_limits: todo!(),
//                     memory_hints: todo!(),
//                     trace: wgpu::Trace::Off,
//                 },
//                 None,
//             )
//             .await
//             .unwrap()
//     }

//     #[test]
//     fn test_material_creation() {
//         // Тестовый GLTF материал
//         let gltf_material = GltfMaterial {
//             name: Some("Test Material"),
//             pbr_metallic_roughness: gltf::material::PbrMetallicRoughness {
//                 base_color_factor: [1.0, 0.5, 0.5, 1.0],
//                 metallic_factor: 0.3,
//                 roughness_factor: 0.7,
//                 ..Default::default()
//             },
//             ..Default::default()
//         };

//         // Тест создания материала с текстурой
//         let texture_handle = Handle::<GpuTexture>::default();
//         let material = Material::from_gltf_texture(&gltf_material, texture_handle);
        
//         assert_eq!(material.name, "Test Material");
//         assert_eq!(material.base_color, [1.0, 0.5, 0.5, 1.0]);
//         assert!(material.base_color_texture.is_some());
//         assert_eq!(material.metallic, 0.3);
//         assert_eq!(material.roughness, 0.7);

//         // Тест создания материала из списка текстур
//         let textures = vec![Handle::<GpuTexture>::default(); 2];
//         let material = Material::from_gltf_material(&gltf_material, &textures);
//         assert!(material.base_color_texture.is_none()); // В нашем тесте нет текстуры
//     }

//     #[tokio::test]
//     async fn test_bind_group_creation() {
//         let (device, _) = create_test_device().await;
        
//         let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             label: None,
//             entries: &[],
//         });

//         let mut material = Material {
//             name: "Test".to_string(),
//             base_color: [1.0; 4],
//             base_color_texture: None,
//             metallic: 0.0,
//             roughness: 0.0,
//             bind_group: None,
//         };

//         // Создаем bind group без текстур
//         material.create_bind_group(&device, &layout, None, None);
//         assert!(material.bind_group.is_none());

//         // TODO: Тест с реальными текстурой и сэмплером потребует больше setup
//     }
// }