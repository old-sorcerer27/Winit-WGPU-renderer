pub mod storage;
pub mod asset;
pub mod mesh;
pub mod texture;
pub mod material;


use std::marker::PhantomData;
use glam::{Vec2, Vec3};
use slotmap::new_key_type;

new_key_type! {
    pub struct MeshKey;
    pub struct TextureKey;
    pub struct MaterialKey;
    pub struct ShaderKey;
}


pub trait Resource {
    type Key: slotmap::Key;
    type LoadParams;
    
    fn load(params: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle<T: Resource> {
    key: T::Key,
    _phantom: PhantomData<T>,
}


#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coord: Vec2,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}
