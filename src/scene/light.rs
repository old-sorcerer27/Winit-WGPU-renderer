use glam::Vec3;

pub enum LightType {
    Directional {
        direction: Vec3,
        intensity: f32,
    },
    Point {
        position: Vec3,
        range: f32,
        intensity: f32,
    },
    Spot {
        position: Vec3,
        direction: Vec3,
        angle: f32,
        range: f32,
        intensity: f32,
    },
}

pub struct Light {
    pub light_type: LightType,
    pub color: [f32; 3],
    pub shadows_enabled: bool,
}