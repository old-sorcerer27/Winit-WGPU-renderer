// use std::collections::HashMap;
// use gltf::Gltf;
// use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, PipelineCompilationOptions, ShaderModule, TextureFormat};
// use winit::dpi::PhysicalSize;

// use crate::res::{asset_manager::AssetManager, mesh::Mesh, vertex::Vertex, Handle};

// use super::{window::WindowManager, PipelineType};

// /// Основной рендерер приложения
// pub struct Renderer {
//     /// Графическое устройство
//     pub device: wgpu::Device,
//     /// Очередь команд
//     pub queue: wgpu::Queue,
//     /// Менеджер ассетов
//     pub assets: AssetManager,
//     /// Загруженные шейдеры
//     // shaders: HashMap<u8, ShaderModule>,
//     pub shader: ShaderModule
// }

// impl<'a> Renderer {
//     /// Создает новый рендерер
//     pub async fn new(
//         window: &WindowManager<'a>,
//         gltf: &Gltf,
//         base_path: &str,
//     ) -> Result<(Self, Vec<Handle<Mesh>>), Box<dyn std::error::Error>> {        
//         // Получаем адаптер и создаем устройство
//         let adapter = window.get_adapter().await;
//         let (device, queue) = WindowManager::init_device(&adapter).await;
        
//         // Загружаем ассеты
//         let mut assets = AssetManager::new();
//         let meshes: Vec<Handle<Mesh>> = assets.load_gltf_meshes(gltf, base_path, &device, &queue)?;


//         // Настройка пайплайна рендеринга
//         let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
//             label: Some("Shader"),
//             source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/cube.wgsl").into()),
//         });

//         Ok(((Self {
//             device,
//             queue,
//             assets,
//             // shaders: HashMap::new(),
//             shader
//         }), meshes))
//     }

//     /// Создает пайплайн рендеринга
//     pub fn create_render_pipeline(
//         &self,
//         format: TextureFormat,
//         bind_groups:  &[&BindGroupLayout],
//     ) -> wgpu::RenderPipeline {
//         let render_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//             label: Some("Render Pipeline Layout"),
//             bind_group_layouts: bind_groups,
//             push_constant_ranges: &[],
//         });

//         return self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//             label: Some("Render Pipeline"),
//             layout: Some(&render_pipeline_layout),
//             vertex: wgpu::VertexState {
//                 module: &self.shader,
//                 entry_point: Some("vs_main"),
//                 buffers: &[
//                     crate::res::vertex::Vertex::desc(),
//                 ],
//                 compilation_options: PipelineCompilationOptions::default(),
//             },
//             fragment: Some(wgpu::FragmentState {
//                 module: &self.shader,
//                 entry_point: Some("fs_main"),
//                 targets: &[Some(wgpu::ColorTargetState {
//                     format: format,
//                     blend: Some(wgpu::BlendState::REPLACE),
//                     write_mask: wgpu::ColorWrites::ALL,
//                 })],
//                 compilation_options: PipelineCompilationOptions::default(),
//             }),
//             primitive: wgpu::PrimitiveState {
//                 topology: wgpu::PrimitiveTopology:: TriangleList,
//                 strip_index_format: None,
//                 front_face: wgpu::FrontFace::Ccw,
//                 cull_mode: Some(wgpu::Face::Back),
//                 polygon_mode: wgpu::PolygonMode::Fill,
//                 unclipped_depth: false,
//                 conservative: false,
//             },
//             // primitive: wgpu::PrimitiveState {
//             //     polygon_mode: wgpu::PolygonMode::Line,
//             //     cull_mode: None,
//             //     ..Default::default()
//             // },
//             depth_stencil: Some(wgpu::DepthStencilState{
//                 format: crate::res::texture::DEPTH_FORMAT,
//                 depth_write_enabled: true,
//                 depth_compare: wgpu::CompareFunction::Less,
//                 stencil: wgpu::StencilState::default(),
//                 bias: wgpu::DepthBiasState::default(),
//             }),
//             multisample: wgpu::MultisampleState::default(),
//             multiview: None,
//             cache: Default::default(),
//         });
//     }
    
//     pub fn resize(&mut self, size: PhysicalSize<f32>) {

//     }
// }



