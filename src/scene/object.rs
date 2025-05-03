use crate::res::{material::Material, mesh::Mesh, Handle};

use super::transform::Transform;

pub enum SceneObjectKind {
    Mesh {
        mesh: Handle<Mesh>,
        material: Handle<Material>,
    },
    Camera {
        fov: f32,
        near: f32,
        far: f32,
    },
    Light,
    Empty,
}

pub struct SceneObject {
    pub kind: SceneObjectKind,
    pub transform: Transform,
    pub visible: bool,
}

impl SceneObject {
    pub fn new_mesh(mesh: Handle<Mesh>, material: Handle<Material>) -> Self {
        Self {
            kind: SceneObjectKind::Mesh { mesh, material },
            transform: Transform::default(),
            visible: true,
        }
    }

    pub fn new_camera(fov: f32, near: f32, far: f32) -> Self {
        Self {
            kind: SceneObjectKind::Camera { fov, near, far },
            transform: Transform::default(),
            visible: false,
        }
    }
}