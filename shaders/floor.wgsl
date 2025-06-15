struct Camera {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.view_proj * vec4<f32>(position, 1.0);
    output.uv = uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Шахматный пол
    let tile = floor(input.uv * 10.0);
    let color = select(vec3<f32>(0.8), vec3<f32>(0.5), (tile.x + tile.y) % 2.0 == 0.0);
    return vec4<f32>(color, 1.0);
}
