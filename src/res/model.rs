use gltf::mesh::Mode;

use super::{animation::Animation, material::Material, mesh::Mesh, Handle, ModelKey, Resource};

pub struct Model {
    meshes: Vec<Handle<Mesh>>,
    // animations: Vec<Handle<Animation>>,
}

impl Resource for Model {
    type Key = ModelKey;
    type LoadParams = Model; 
    
    fn load(model: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            meshes: model.meshes,
            // animations: model.animations
        })
    }
}


// impl Model {
//     pub fn load_model(

//     )-> Self {
        
//     }
// }