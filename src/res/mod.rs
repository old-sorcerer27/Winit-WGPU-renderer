//! Система управления игровыми ресурсами (ассетами).
//!
//! Предоставляет систему типобезопасных handle'ов для управления такими ресурсами как:
//! - Меши (Mesh)
//! - Текстуры (Texture)
//! - Материалы (Material)
//! - Шейдеры (Shader)
//! - Модели (Model)
//! - Сцены (Scene)
//! - Анимации (Animation)

pub mod storage;
pub mod asset_manager;
pub mod vertex;
pub mod mesh;
pub mod scin;
pub mod texture;
pub mod material;
pub mod animation;
pub mod model;
pub mod buffer;
pub mod image;
pub mod scene;
pub mod camera;

#[cfg(test)]
pub mod test;  // Модуль тестов (только для тестирования)

use std::{marker::PhantomData, path::Path};
use buffer::{load_gltf_buffers, BufferData};
use gltf::Gltf;
use image::{load_gltf_images, ImageData};
use slotmap::new_key_type;

/// Уникальные типы ключей для разных ресурсов.
///
/// Используются вместе с slotmap для типобезопасных handle'ов.
new_key_type! {
    /// Ключ для мешей
    pub struct MeshKey;
    /// Ключ для текстур
    pub struct TextureKey;
    /// Ключ для материалов
    pub struct MaterialKey;
    /// Ключ для шейдеров
    pub struct ShaderKey;
    /// Ключ для моделей
    pub struct ModelKey;
    /// Ключ для сцен
    pub struct SceneKey;
    /// Ключ для анимаций
    pub struct AnimationKey;
    /// Ключ для камеры
    pub struct CameraKey;
    /// Ключ для скелета
    pub struct ScinKey;
}

/// Трейт для типов, которые могут быть загружены и управляться как ресурсы.
///
/// # Пример
/// ```
/// use your_crate::Resource;
///
/// struct MyResource;
///
/// impl Resource for MyResource {
///     type Key = MeshKey;
///     type LoadParams = String;
///     
///     fn load(params: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>> {
///         println!("Загружаем ресурс из: {}", params);
///         Ok(MyResource)
///     }
/// }
/// ```
pub trait Resource {
    /// Тип ключа для этого ресурса
    type Key: slotmap::Key;
    
    /// Параметры, необходимые для загрузки ресурса
    type LoadParams;
    
    /// Загружает ресурс из указанных параметров
    ///
    /// # Ошибки
    /// Возвращает ошибку, если не удалось загрузить ресурс
    fn load(params: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

/// Handle (дескриптор) ресурса, обеспечивающий типобезопасный доступ.
///
/// # Пример
/// ```
/// use your_crate::{Handle, Mesh, MeshKey};
///
/// let mesh_handle = Handle::<Mesh>::new(MeshKey::default());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle<T: Resource> {
    key: T::Key,
    _phantom: PhantomData<T>,  // Для типобезопасности
}

impl<T: Resource> Handle<T> {
    /// Создает новый handle из ключа
    pub fn new(key: T::Key) -> Self {
        Self {
            key,
            _phantom: PhantomData,
        }
    }
    
    /// Возвращает внутренний ключ
    pub fn key(&self) -> T::Key {
        self.key
    }
    
    /// Преобразует handle в ключ
    pub fn into_key(self) -> T::Key {
        self.key
    }
}


pub fn load_gltf_file_data(
    gltf: &Gltf,
    base_path: Option<&Path>,
) -> Result<(Vec<BufferData>, Vec<ImageData>), Box<dyn std::error::Error>> {
    let buffers =  match load_gltf_buffers(gltf, base_path){
        Ok(buffers) => buffers,
        Err(_) => return Err("Error loading file data (buffers)".into()),
    };
    match load_gltf_images(gltf, base_path, &buffers) {
        Ok(images) => return Ok((buffers, images)),
        Err(_) => return Err("Error loading file data (images)".into()),
    } 
}


pub async fn load_binary(file_name: &str) -> Result<Vec<u8>,Box<dyn std::error::Error>> {
    let path = std::path::Path::new("assets").join(file_name);
    let data = std::fs::read(path)?;
    Ok(data)
}