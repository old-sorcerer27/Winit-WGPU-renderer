use std::{error::Error, fmt, path::Path};
use gltf::json::extensions::scene;
use wgpu::Device;

use crate::res::mesh::Mesh;
use super::{buffer::{to_vec, BufferData}, material::Material, model::Model, Handle, Resource, SceneKey};

#[derive(Debug, Clone)] 
pub struct Scene {
   pub models: Vec<Handle<Model>>
}

impl Resource for Scene {
    type Key = SceneKey;
    type LoadParams = Scene;
    

    fn load(params: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(params) 
    }
}

#[derive(Debug)]
pub struct LoadSceneDataError {
    message: String,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl fmt::Display for LoadSceneDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scene loading error: {}", self.message)
    }
}

impl Error for LoadSceneDataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| &**e as _)
    }
}

impl LoadSceneDataError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source(message: impl Into<String>, source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl Scene {
    // pub fn from_gltf_scene( 
    // scene: gltf::Scene,
    // buffers: Vec<BufferData>,
    // material_handles: Vec<Handle<Material>>
    // )-> Result<Scene,  Box<dyn std::error::Error>> {
    //     let buff = to_vec(buffers);
    //     let mut nodes = Vec::new();
    //     for node in scene.nodes() {
    //         let mut meshes = Vec::new();
    //         let mesh = node.mesh().expect("Got mesh");
    //         let primitives = mesh.primitives();
    //         primitives.for_each(|primitive| {
    //              match Mesh::from_gltf_primitive(
    //                 &primitive, 
    //                 &buff,
    //                 material_handles.get(primitive.material().index().unwrap_or(0)).cloned(),
    //             ) {
    //                 Ok(mesh) =>  meshes.push(mesh),
    //                 Err(_) => todo!(),
    //             }
    //         });
    //         nodes.push(meshes);
    //     }
    //     Ok(nodes)
    // }
}

/// Загружает данные сцены из GLTF-файла, преобразуя их в меши.
///
/// # Параметры
/// - `scene`: Сцена из GLTF-файла, содержащая узлы (ноды) с мешами.
/// - `buffers`: Вектор буферов, содержащих данные вершин, индексов и т. д.
///
/// # Возвращает
/// - `Result<Vec<Vec<Mesh>>, Box<dyn std::error::Error>>`:  
///   - В случае успеха — двумерный вектор мешей (по одному вектору на каждый узел).  
///   - В случае ошибки — `Box<dyn Error>` (например, если не удалось распарсить буферы).  
///
/// # Пример использования
/// ```rust
/// let gltf_doc = gltf::Gltf::open("model.glb")?;
/// let scene = gltf_doc.default_scene().unwrap();
/// let buffers = load_buffers(&gltf_doc)?; // Предположим, что буферы загружены
/// let meshes = load_scene_data(scene, buffers)?;
/// ```
pub fn load_scene_meshes(
    scene: gltf::Scene,
    buffers: Vec<BufferData>,
    material_handles: Vec<Handle<Material>>,
    device: Device
)-> Result<Vec<Vec<Mesh>>,  Box<dyn std::error::Error>> {
    let buff = to_vec(buffers);
    let mut nodes = Vec::new();
    for node in scene.nodes() {
        let mut meshes = Vec::new();
        let mesh = node.mesh().expect("Got mesh");
        let primitives = mesh.primitives();
        primitives.for_each(|primitive| {
             match Mesh::from_gltf_primitive(
                &primitive, 
                &buff,
                material_handles.get(primitive.material().index().unwrap_or(0)).cloned(),
                device.clone()
            ) {
                Ok(mesh) =>  meshes.push(mesh),
                Err(_) => todo!(),
            }
        });
        nodes.push(meshes);
    }
    Ok(nodes)
}


pub fn load_children_nodes_meshes(
    node: gltf::Node,
    buffers: Vec<BufferData>,
    material_handles: Vec<Handle<Material>>,
    device: Device
)-> Result<Vec<Mesh>,  Box<dyn std::error::Error>> {
    let buff = to_vec(buffers);
    let children = node.children();
    let mut meshes = Vec::new();
    for node in children {
        let mesh = node.mesh().expect("Got mesh");
        let primitives = mesh.primitives();
        primitives.for_each(|primitive| {
            match Mesh::from_gltf_primitive(
                &primitive, 
                &buff,
                material_handles.get(primitive.material().index().unwrap_or(0)).cloned(),
                device.clone()
            ) {
                Ok(mesh) =>  meshes.push(mesh),
                Err(_) => todo!(),
            }
        });
    }
    Ok(meshes)
}


// #[cfg(test)]
// mod tests {
//     use crate::res::buffer::load_gltf_buffers;

//     use super::*;
//     use gltf::Gltf;
//     use std::path::Path;
//     use std::fs::File;
//     use std::io::Read;

//     /// Загружает тестовый GLB-файл (например, куб)
//     fn load_test_glb(path: &str) -> Result<Gltf, Box<dyn std::error::Error>> {
//         let mut file = File::open(path)?;
//         let mut data = Vec::new();
//         file.read_to_end(&mut data)?;
//         Ok(Gltf::from_slice(&data)?)
//     }

//     #[test]
//     fn test_load_scene_data_with_real_glb() {
//         // 1. Загружаем тестовый GLB (например, `cube.glb` из папки `test_assets`)
//         let gltf = load_test_glb("test_assets/cube.glb").expect("Failed to load test GLB file");
        
//         // 2. Загружаем буферы (используя вашу функцию `load_gltf_buffers`)
//         let buffers = load_gltf_buffers(&gltf, Some(Path::new("test_assets/")))
//             .expect("Failed to load GLTF buffers");

//         // 3. Берём сцену (например, дефолтную)
//         let scene = gltf.default_scene().expect("GLTF has no default scene");

//         // 4. Тестируем `load_scene_data`
//         let result = load_scene_data(scene, buffers);
//         assert!(result.is_ok(), "Failed to load scene data: {:?}", result.err());

//         let meshes = result.unwrap();
//         assert!(!meshes.is_empty(), "No nodes loaded in the scene");
//         assert!(!meshes[0].is_empty(), "First node has no meshes");
//     }
// }