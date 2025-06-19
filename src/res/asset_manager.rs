use core::fmt;
use std::{error::Error, path::Path};

use gltf::{Gltf};

use crate::res::texture::{gpu_texture::GpuTexture, load_gltf_texture_source_data};

use super::{buffer::{load_gltf_buffers, to_vec}, camera::Camera, material::Material, mesh::Mesh, model::Model, scene::Scene, storage::Storage,  Handle};


#[derive(Debug, Clone)] 
pub struct AssetManager {
    pub scenes: Storage<Scene>,
    pub models: Storage<Model>,
    pub meshes: Storage<Mesh>,
    pub textures: Storage<GpuTexture>,
    pub materials: Storage<Material>,
    pub cameras: Storage<Camera>,
}

#[derive(Debug)]
pub struct LoadAssetError {
    message: String,
}

impl fmt::Display for LoadAssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Asset loading error: {}", self.message)  // Исправлено сообщение
    }
}

impl Error for LoadAssetError {}

impl LoadAssetError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            scenes: Storage::new(),
            models: Storage::new(),
            meshes: Storage::new(),
            textures: Storage::new(),
            materials: Storage::new(),
            cameras: Storage::new(),
        }
    }
    
    pub fn load_from_directory(
        &mut self,
        base_path: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        
    }

    pub fn load_gltf_meshes<'a>(
        &mut self,
        gltf: &'a Gltf,
        base_path: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        )-> Result<Vec<Handle<Mesh>>,  Box<dyn std::error::Error>> {
        let base_path: Option<&Path> = Some(Path::new(&base_path));
        let buffers = match load_gltf_buffers(gltf, base_path) {
            Ok(buffers) => buffers,
            Err(_) => return Err(Box::new(LoadAssetError::new("Load buffers error"))),
        };


        let mut material_handles: Vec<Handle<Material>> = Vec::new();
        for material in gltf.materials() { 
            let pbr = material.pbr_metallic_roughness();
            
            let image_data = match load_gltf_texture_source_data(base_path, pbr, &buffers){
                Ok(data) => {data},
                Err(_) => {return Err(Box::new(LoadAssetError::new("Load texture source error")))},
            };

            match  GpuTexture::from_bytes(device, queue, &image_data, "lable") {
                Ok(texture) => {
                    match self.textures.load(texture){
                        Ok(t_h) => {
                            match self.materials.load(Material::from_gltf_texture(&material, t_h)){
                                Ok(h_m) => {material_handles.push(h_m)},
                                Err(_) => {return Err(Box::new(LoadAssetError::new("Load texture source error")))},
                            }
                        },
                        Err(_) => {return Err(Box::new(LoadAssetError::new("Load texture source error")))},
                    };
                },
                Err(_) => {return Err(Box::new(LoadAssetError::new("Load texture error")))},
            }
        }
       
        let mut mesh_handles = Vec::new();
        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let mesh = Mesh::from_gltf_primitive(
                    &primitive,
                    &to_vec(buffers.clone()),
                    material_handles.get(primitive.material().index().unwrap_or(0)).cloned(),
                    device.clone(),
                )?;
                match self.meshes.load(mesh) {
                    Ok(m) => {mesh_handles.push(m);},
                    Err(_) => {return Err(Box::new(LoadAssetError::new("Load texture error")))},
                }
            }
        }

        Ok(mesh_handles)
    }

    // pub fn load_gltf_data<'a>(
    //     mut self,
    //     gltf: &'a Gltf,
    //     file_name: &str,
    //     device: &wgpu::Device,
    //     queue: &wgpu::Queue,
    //     )-> Result<Vec<Handle<Scene>>,  Box<dyn std::error::Error>> {
    //     let base_path: Option<&Path> = Some(Path::new(&file_name));
    //     let (buffers, images) = load_gltf_file_data(gltf, base_path).unwrap();


    //     let mut material_handles: Vec<Handle<Material>> = Vec::new();
    //     for material in gltf.materials() { 
    //         let pbr = material.pbr_metallic_roughness();
            
    //         let image_data = match load_gltf_texture_source_data(base_path, pbr, &buffers){
    //             Ok(data) => {data},
    //             Err(_) => {return Err(Box::new(LoadAssetError::new("Load texture source error")))},
    //         };

    //         match  GpuTexture::from_bytes(device, queue, &image_data, "lable") {
    //             Ok(texture) => {
    //                 match self.textures.load(texture){
    //                     Ok(t_h) => {
    //                         match self.materials.load(Material::from_gltf_texture(&material, t_h)){
    //                             Ok(h) => {material_handles.push(h)},
    //                             Err(_) => {return Err(Box::new(LoadAssetError::new("Load texture source error")))},
    //                         }
    //                     },
    //                     Err(_) => {return Err(Box::new(LoadAssetError::new("Load texture source error")))},
    //                 };
    //             },
    //             Err(_) => {return Err(Box::new(LoadAssetError::new("Load texture error")))},
    //         }
    //     }

    //     let scene_handles = Vec::new();
    //     for scene in gltf.scenes() {
    //         let scene = load_scene_meshes(scene, buffers.clone(), material_handles.clone());
    //         for nodes in scene {
    //             for node in nodes {
    //                 for mesh in node {  
    //                     self.meshes.load(mesh);
    //                 }
    //             }
    //         }
    //     }

    //     Ok(scene_handles)
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

#[cfg(test)]
mod tests {
    use super::*;
}