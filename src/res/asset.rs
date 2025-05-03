use std::{collections::HashMap, path::Path};



use gltf::{buffer::Data, iter::{Buffers, Images}, Buffer, Gltf};

use super::{material::Material, mesh::Mesh, storage::Storage, texture::Texture, Handle, Resource};


pub struct AssetManager {
    pub meshes: Storage<Mesh>,
    pub textures: Storage<Texture>,
    pub materials: Storage<Material>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            meshes: Storage::new(),
            textures: Storage::new(),
            materials: Storage::new(),
        }
    }
    

        pub fn load_gltf(
        &mut self,
        path: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<Handle<Mesh>>, Box<dyn std::error::Error>> {
        let (gltf, buffers, images) = Self::load_gltf_file(path)?;
        let base_path = Path::new(path).parent().unwrap_or(Path::new(""));
        
        // Сначала загружаем все текстуры
        let mut texture_handles = Vec::new();
        for image in images {
            let texture = Texture::from_gltf_image(image, device, queue)?;
            texture_handles.push(self.textures.load(texture));
        }

        // Затем загружаем материалы
        let mut material_handles = Vec::new();
        for material in gltf.materials() {
            let mat = Material::from_gltf(&material, &texture_handles);
            material_handles.push(self.materials.load(mat));
        }

        // Наконец загружаем меши
        let mut mesh_handles = Vec::new();
        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let mesh = Mesh::from_gltf_primitive(
                    &primitive,
                    &buffers
                )?;
                mesh_handles.push(self.meshes.load(mesh));
            }
        }

        Ok(mesh_handles)
    }

    fn load_gltf_file(path: &str) -> Result<(Gltf, Vec<Buffer>, Vec<Data>), Box<dyn std::error::Error>> {
        let gltf = Gltf::open(path)?;
        let buffers = gltf.load_buffers()?;
        let images = gltf.load_images()?;
        Ok((gltf, buffers, images))
    }

    // fn load_gltf_file(path: &str) -> Result<(Gltf, Buffers, Images), Box<dyn std::error::Error>> {
    //     let gltf = Gltf::open(path)?;
    //     let buffers = gltf.buffers();
    //     let images = gltf.images();
    //     Ok((gltf, buffers, images))
    // }

    // fn cached_load<T: Resource>(
    //     &mut self,
    //     cache: &mut HashMap<String, Handle<T>>,
    //     path: &str,
    //     params: T::LoadParams,
    // ) -> Result<Handle<T>, Box<dyn std::error::Error>> {
    //     if let Some(handle) = cache.get(path) {
    //         return Ok(*handle);
    //     }
    //     let handle = T::load(params)?;
    //     cache.insert(path.to_string(), handle);
    //     Ok(handle)
    // }
}