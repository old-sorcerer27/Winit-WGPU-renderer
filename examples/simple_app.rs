use diploma_thesis::res::{asset_manager::AssetManager, texture::{self, get_texture_bind_group_layout, DEPTH_FORMAT}, vertex::Vertex};
use gltf::Gltf;
use wgpu::{DepthStencilState, MemoryHints, PipelineCompilationOptions};
use winit::{
    event::{Event, WindowEvent}, event_loop::EventLoop, window::{Window, WindowBuilder}
};
use pollster::block_on;
use glam::Mat4;

use std::{path::Path, time::Instant};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    block_on(run(event_loop, window));
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = { instance.create_surface(&window) }.unwrap();

    // Выбор адаптера (GPU)
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    // Устройство и очередь
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // required_features: wgpu::Features::POLYGON_MODE_LINE,
                required_limits: wgpu::Limits::default(),
                memory_hints: MemoryHints::default(),
                trace: wgpu::Trace::Off,
            },
        )
        .await
        .unwrap();



    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats[0];
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../examples/examples_shaders/cube_copy.wgsl").into() 
        ),
    });
    
    let transform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Transform Buffer"),
        size: std::mem::size_of::<Mat4>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let transform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Transform Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Transform Bind Group"),
        layout: &transform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: transform_buffer.as_entire_binding(),
        }],
    });


    // let gltf_path = Path::new("examples/examples_assets/tesseract_cube.glb");
    let mut assets = AssetManager::new();

    let gltf_path = Path::new("examples/examples_assets/cube_model/scene.gltf");
    let gltf = Gltf::open(gltf_path).unwrap();
    let meshes = assets.load_gltf_meshes(&gltf, "examples/examples_assets/cube_model/", &device, &queue);

    // let gltf_path = Path::new("examples/examples_assets/cube.glb");
    // let gltf = Gltf::open(gltf_path).unwrap();
    // let meshes2 = assets.load_gltf_meshes(&gltf, "examples/examples_assets/", &device, &queue);

    let cube = assets.meshes.get(
        match meshes{
        Ok(asset) => {asset[0].clone()},
        Err(_) => todo!(),
    });

    let material = match assets.materials.get_mut(cube.unwrap().material.clone().unwrap()){
        Some(mat) => mat,
        None => todo!(),
    };

    let texture = match assets.textures.get_mut(material.base_color_texture.clone().unwrap()){
        Some(text) => text,
        None => todo!(),
    };

    material.create_bind_group(
        &device, 
        None, 
        Some(&texture.view), 
        Some(&texture.sampler)
    );

    let mut depth_texture = texture::GpuTexture::create_depth_texture(&device, &config, "depth_texture");


    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
                &transform_bind_group_layout,
                &get_texture_bind_group_layout(&device),
            ],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[
                Vertex::desc(),
            ],
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
            topology: wgpu::PrimitiveTopology:: TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        // primitive: wgpu::PrimitiveState {
        //     polygon_mode: wgpu::PolygonMode::Line,
        //     cull_mode: None,
        //     ..Default::default()
        // },
        // depth_stencil: None,
        depth_stencil: Some(DepthStencilState{
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: Default::default(),
    });




    let start_time = Instant::now();

    event_loop.run( |event, elwt: &winit::event_loop::EventLoopWindowTarget<()>| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => elwt.exit(),
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                config.width = new_size.width;
                config.height = new_size.height;
                surface.configure(&device, &config);
                depth_texture = texture::GpuTexture::create_depth_texture(&device, &config, "depth_texture");
            }
            Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                let elapsed = start_time.elapsed().as_secs_f32();
                let model = Mat4::from_rotation_x(elapsed);

                queue.write_buffer(
                    &transform_buffer,
                    0,
                    bytemuck::cast_slice(&model.to_cols_array_2d()),
                );


                let frame = surface.get_current_texture().unwrap();
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        timestamp_writes: Default::default(),
                        occlusion_query_set: Default::default(),
                        // depth_stencil_attachment: None,
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &depth_texture.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                    });
                    render_pass.set_viewport( 
                        0.0, 0.0, 
                        config.width as f32, config.height as f32, 
                        0.0, 1.0
                    );
                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_bind_group(0, &transform_bind_group, &[]);
                    render_pass.set_bind_group(1, &material.bind_group, &[]);
                    render_pass.set_vertex_buffer(0, cube.unwrap().vertex_buffer.slice(..));
                    render_pass.set_index_buffer(cube.unwrap().index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..cube.unwrap().indices.len() as u32, 0, 0..1);
                }

                queue.submit(std::iter::once(encoder.finish()));
                frame.present();
                window.request_redraw();
            }
            _ => (),
        }
    }).unwrap();
}


