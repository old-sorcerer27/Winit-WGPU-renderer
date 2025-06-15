struct PointLight {
    position: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
};

@group(1) @binding(0)
var<storage> lights: array<PointLight>;

fn calculate_lighting(normal: vec3<f32>) -> vec3<f32> {
    var result = vec3<f32>(0.0);
    for (var i = 0u; i < arrayLength(&lights); i++) {
        result += lights[i].color * max(dot(normal, normalize(lights[i].position)), 0.0);
    }
    return result;
}




// struct VertexInput {
//     @location(0) position: vec3<f32>,
// };

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) color: vec3<f32>,
// };

// @vertex
// fn vs_main(
//     model: VertexInput,
// ) -> VertexOutput {
//     let scale = 0.25;
//     var out: VertexOutput;
//     out.clip_position = camera.view_proj * vec4<f32>(model.position * scale + light.position, 1.0);
//     out.color = light.color;
//     return out;
// }

// // Fragment shader

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     return vec4<f32>(in.color, 1.0);
// }