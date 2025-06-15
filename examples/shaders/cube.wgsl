struct Transform {
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> transform: Transform;

@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = transform.model * vec4<f32>(position * 0.3, 1.0);
    output.color = vec3<f32>(1.0, 0.0, 0.0);
    return output;
}



@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}


