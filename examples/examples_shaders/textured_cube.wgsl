// Структуры данных
struct Transform {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

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
}

// Uniform-буферы
@group(0) @binding(0)
var<uniform> transform: Transform;

@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

// Вершинный шейдер
@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var output: VertexOutput;
    
    // Преобразование позиции
    let model_view = transform.view * transform.model;
    let mvp = transform.projection * model_view;
    output.clip_position = mvp * vec4<f32>(input.position, 1.0);
    
    // Передаем данные во фрагментный шейдер
    output.tex_coords = input.tex_coords;
    
    // Преобразование нормалей
    let normal_matrix = mat3x3<f32>(
        transform.model[0].xyz,
        transform.model[1].xyz,
        transform.model[2].xyz
    );
    output.normal = normalize(normal_matrix * input.normal);
    
    // Мировая позиция для освещения
    output.world_position = (transform.model * vec4<f32>(input.position, 1.0)).xyz;
    
    return output;
}

// Фрагментный шейдер
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Чтение текстуры
    let texture_color = textureSample(base_color_texture, texture_sampler, input.tex_coords).rgb;
    
    // Простое освещение (диффузное)
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let light_color = vec3<f32>(1.0, 1.0, 1.0);
    let ambient = 0.1;
    
    let diffuse = max(dot(input.normal, light_dir), 0.0);
    let lighting = ambient + diffuse;
    
    // Итоговый цвет
    return vec4<f32>(texture_color * lighting * light_color, 1.0);
}

