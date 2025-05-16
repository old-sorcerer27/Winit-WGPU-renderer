pub mod object;
pub mod transform;
pub mod camera;
pub mod light;
pub mod shader;

use std::collections::HashMap;

use object::{SceneEntity, SceneEntityKind};
use transform::Transform;

use crate::res::{texture::GpuTexture, Handle};


#[derive(Default)]
pub struct GpuScene {
    pub objects: HashMap<String, SceneEntity>,
    pub active_camera: String,
    pub lights: Vec<winit::window::Theme>,
    pub ambient_light: [f32; 3],
    pub skybox: Option<Handle<GpuTexture>>,
}

impl GpuScene {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            active_camera: "main_camera".to_string(),
            lights: Vec::new(),
            ambient_light: [0.1, 0.1, 0.1],
            skybox: None,
        }
    }

    /// Добавляет объект в сцену
    pub fn add_object(&mut self, name: &str, object: SceneEntity) {
        self.objects.insert(name.to_string(), object);
    }

    /// Устанавливает активную камеру
    pub fn set_active_camera(&mut self, name: &str) {
        self.active_camera = name.to_string();
    }

    /// Возвращает трансформ камеры
    pub fn get_camera_transform(&self) -> Option<&Transform> {
        self.objects.get(&self.active_camera)
            .and_then(|obj| match &obj.kind {
                // SceneObjectKind::Camera => Some(&obj.transform),
                SceneEntityKind::Camera { fov: _, near: _, far: _ } => Some(&obj.transform),
                _ => None,
            })
    }
}