use std::collections::HashMap;
use glam::Mat4;
use gltf::{Gltf, Scene};
use wgpu::{util::DeviceExt, PipelineCompilationOptions, ShaderModule};
use crate::{
    res::{asset_manager::AssetManager, mesh::Mesh, Handle},
};

use super::window::WindowManager;

/// Основной рендерер приложения
pub struct Renderer {
    /// Графическое устройство
    pub device: wgpu::Device,
    /// Очередь команд
    pub queue: wgpu::Queue,
    /// Вершинный буфер
    pub vertex_buffer: wgpu::Buffer,
    /// Менеджер ассетов
    assets: AssetManager,
    /// Загруженные пайплайны рендеринга
    pipelines: HashMap<PipelineType, wgpu::RenderPipeline>,
    /// Загруженные шейдеры
    shaders: HashMap<u8, ShaderModule>,
}

impl<'a> Renderer {
    /// Создает новый рендерер
    pub async fn new(
        window: &WindowManager<'a>,
        gltf: &Gltf,
        base_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Инициализация графического контекста
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        
        // Получаем адаптер и создаем устройство
        let adapter = window.get_adapter(&instance).await;
        let (device, queue) = WindowManager::init_device(&adapter).await;
        
        // Загружаем ассеты
        let mut assets = AssetManager::new();
        assets.load_gltf_meshes(gltf, base_path, &device, &queue)?;

        // Создаем вершинный буфер (пример с кубом)
        let vertices = [
            Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], color: [0.0, 1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0] },
            Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 1.0, 0.0] },
            Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 1.0, 1.0] },
            Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 0.0, 0.0] },
        ];
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Настройка пайплайна рендеринга
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/cube.wgsl").into()),
        });

        let render_pipeline = Self::create_render_pipeline(
            &device,
            &shader,
            window.surface().unwrap().get_capabilities(&adapter).formats[0],
        );

        Ok(Self {
            device,
            queue,
            assets,
            vertex_buffer,
            pipelines: HashMap::from([(PipelineType::Simple, render_pipeline)]),
            shaders: HashMap::new(),
        })
    }

    /// Создает пайплайн рендеринга
    fn create_render_pipeline(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        surface_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
         let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 3]>() as u64,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                    ],
                }],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: Default::default(),
        })
    }

    /// Отрисовывает всю сцену
    pub fn render_scene(&mut self, scene: &Scene) {
        // Реализация отрисовки сцены
    }

    /// Отрисовывает конкретный меш
    pub fn render_mesh(&mut self, mesh_handle: Handle<Mesh>) {
        if let Some(mesh) = self.assets.meshes.get(mesh_handle) {
            self.queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&mesh.vertices),
            );
        }
    }
}

/// Вершина для рендеринга
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    /// Описание формата вершины для wgpu
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Типы пайплайнов рендеринга
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineType {
    /// Простой пайплайн для отрисовки мешей
    Simple,
}


