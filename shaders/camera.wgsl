struct Camera {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    padding: f32,
};

@group(0) @binding(0)
var<uniform> camera: Camera;



