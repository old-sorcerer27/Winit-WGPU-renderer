#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                    wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}


pub const VERTICES: &[SimpleVertex] = &[
    SimpleVertex {
        position: [-0.5, 0.5],
        tex_coords: [0.0, 0.0],
    },
    SimpleVertex {
        position: [-0.5, -0.5],
        tex_coords: [0.0, 1.0],
    },
    SimpleVertex {
        position: [0.5, -0.5],
        tex_coords: [1.0, 1.0],
    },
    SimpleVertex {
        position: [-0.5, 0.5],
        tex_coords: [0.0, 0.0],
    },
    SimpleVertex {
        position: [0.5, -0.5],
        tex_coords: [1.0, 1.0],
    },
    SimpleVertex {
        position: [0.5, 0.5],
        tex_coords: [1.0, 0.0],
    },
];


#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexUniforms {
    pub view_projection_matrix: nalgebra_glm::Mat4,
    pub model_matrix: nalgebra_glm::Mat4,
}

// #[repr(C)]
// #[derive(Clone, Copy, Debug)]
// pub struct VertexUniforms {
//     pub view_projection_matrix: glam::Mat4,
//     pub model_matrix: glam::Mat4,
// }

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimpleVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl SimpleVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 2]>() as u64,
                    shader_location: 1,
                },
            ],
        }
    }
}


