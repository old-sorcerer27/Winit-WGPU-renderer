use std::ops::Range;

use gltf::mesh::Mode;

use super::{animation::Animation, material::Material, mesh::{Mesh}, Handle, ModelKey, Resource};

#[derive(Debug, Clone)] 
pub struct Model {
    pub meshes: Vec<Handle<Mesh>>,
    pub animations: Option<Vec<Handle<Animation>>>,
}

impl Resource for Model {
    type Key = ModelKey;
    type LoadParams = Model; 
    
    fn load(model: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(model) 
    }
}

