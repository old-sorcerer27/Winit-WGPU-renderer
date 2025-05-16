//матрица преобразований (необходима для преобразований)
struct Transform {
    model: mat4x4<f32>,
}

//привязываем матрицу к групе ()  
@group(0) @binding(0)
var<uniform> transform: Transform;


//вершины
@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return transform.model * vec4<f32>(position, 1.0);
}

//фрагменты (цвет)
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Красный цвет
}


