struct Camera {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec3<f32>,
}

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.view_proj * vec4<f32>(position, 1.0);
    output.tex_coords = position;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let direction = normalize(input.tex_coords);
    let sky_color = mix(vec3<f32>(0.1, 0.3, 0.8), vec3<f32>(0.6, 0.8, 1.0), direction.y * 0.5 + 0.5);
    return vec4<f32>(sky_color, 1.0);
}