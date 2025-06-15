@group(0) @binding(0)
var base_color_texture: texture_2d<f32>;

@group(0) @binding(1)
var texture_sampler: sampler;

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) view_position: vec3<f32>,
}

@vertex
fn vs_main(
    input: VertexInput,
    @builtin(instance_index) instance_idx: u32
) -> VertexOutput {
    var output: VertexOutput;
    
    output.world_position = input.position;
    output.clip_position = camera.view_proj * vec4<f32>(input.position, 1.0);
    output.tex_coords = input.tex_coords;
    output.normal = normalize(input.normal);
    output.view_position = (vec4<f32>(input.position, 1.0)).xyz;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(base_color_texture, texture_sampler, input.tex_coords).rgb;
    
    let light_pos = vec3<f32>(2.0, 5.0, 3.0);
    let light_color = vec3<f32>(1.0, 1.0, 1.0);
    let ambient_strength = 0.5;
    let light_dir = normalize(light_pos - input.world_position);
    let diffuse_strength = max(dot(input.normal, light_dir), 0.0);
    
    let view_dir = normalize(-input.view_position);
    let reflect_dir = reflect(-light_dir, input.normal);
    let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    
    let ambient = ambient_strength * light_color;
    let diffuse = diffuse_strength * light_color;
    let specular = specular_strength * light_color;
    
    return vec4<f32>((ambient + diffuse + specular) * texture_color, 1.0);
}

