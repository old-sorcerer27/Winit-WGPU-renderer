pub mod object;
pub mod transform;
pub mod camera;
pub mod light;

use std::collections::HashMap;
use object::{SceneObject, SceneObjectKind};
use transform::Transform;
use crate::res::{texture::Texture, Handle};

pub struct Scene {
    pub objects: HashMap<String, SceneObject>,
    pub active_camera: String,
    pub lights: Vec<winit::window::Theme>,
    pub ambient_light: [f32; 3],
    pub skybox: Option<Handle<Texture>>,
}

// impl Serialize for Scene {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         // Реализация сериализации
//     }
// }

impl Scene {
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
    pub fn add_object(&mut self, name: &str, object: SceneObject) {
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
                SceneObjectKind::Camera { fov: _, near: _, far: _ } => Some(&obj.transform),
                _ => None,
            })
    }
}