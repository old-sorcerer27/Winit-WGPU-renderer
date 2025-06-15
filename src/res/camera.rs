use glam::{Quat, Vec3};

use super::{CameraKey, Resource};

#[derive(Debug, Clone)] 
pub struct Camera {
    pub name: String,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Resource for Camera {
    type Key = CameraKey;
    type LoadParams = Camera;
    

    fn load(camera: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(camera) 
    }
}