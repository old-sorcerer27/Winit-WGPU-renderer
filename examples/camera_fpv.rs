use diploma_thesis::{controll::camera::fpv_camera::FpvCameraController, res::{asset_manager::AssetManager, texture::gpu_texture::{get_texture_bind_group_layout, DEPTH_FORMAT}, vertex::Vertex}, scene::{camera::get_camera_bind_group_layout, entity::SceneEntity}};
use gltf::Gltf;
use wgpu::{DepthStencilState, MemoryHints, PipelineCompilationOptions};
use winit::{
    event::{Event, WindowEvent}, event_loop::EventLoop, window::{Window, WindowBuilder}
};
use pollster::block_on;
use glam::{Quat, Vec3};

use std::path::Path;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    block_on(run(event_loop, window));
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    window.set_cursor_visible(false);

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = { instance.create_surface(&window) }.unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
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
            include_str!("../examples/shaders/test_light_camera.wgsl").into() 
        ),
    });

    let mut assets = AssetManager::new();
    let gltf_path = Path::new("examples/assets/cube_model/scene.gltf");
    let gltf = Gltf::open(gltf_path).unwrap();
    let meshes = assets.load_gltf_meshes(&gltf, "examples/assets/cube_model/", &device, &queue);


    let cube = assets.meshes.get(
        match meshes{
        Ok(asset) => {asset[0].clone()},
        Err(_) => todo!(),
    });

    println!("{:?}", cube);

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


    let mut aspect_ratio = config.width as f32 / config.height as f32;
    let mut camera = SceneEntity::new_camera(
        &device, 
        Vec3::new(0., 0., 5.), 
        Quat::from_rotation_y(0.0), 
        Vec3::ONE, 
        45., 
        aspect_ratio, 
        0.1,
        100.,
          None
    );
    let mut camera_controler = FpvCameraController::new(0.1, 0.02);

    // let light = SceneEntity::new_light(
    //     &device, 
    //     Vec3::new(0., 5., -10.), 
    //     Quat::IDENTITY, 
    //     Vec3::new(0., 0., 0.),
    //     LightType::Directional, 
    //     Vec3::new(1., 1., 1.), 
    //     true, 
    //     10.
    // );


    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
                // &get_light_bind_group_layout(&device),
                &get_texture_bind_group_layout(&device),
                &get_camera_bind_group_layout(&device),
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

    let mut depth_texture = diploma_thesis::res::texture::gpu_texture::GpuTexture::create_depth_texture(&device, &config, "depth_texture");

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
                aspect_ratio = new_size.width as f32 / new_size.height as f32;
                depth_texture = diploma_thesis::res::texture::gpu_texture::GpuTexture::create_depth_texture(&device, &config, "depth_texture");
            }
            // Event::WindowEvent {
            //     event: WindowEvent::Focused(gained_focus),
            //     ..
            // } => {
            //     if gained_focus {
            //         window.set_cursor_position(
            //             winit::dpi::PhysicalPosition::new(window.inner_size().width/2, window.inner_size().height/2)
            //         ).unwrap();
            //     }
            // }
            Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
            } => {
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
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &depth_texture.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                    });

                    camera_controler.update_camera(&mut camera, &queue);
                   
                    render_pass.set_viewport(0.0, 0.0, config.width as f32, config.height as f32, 0.0, 1.0);
                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_bind_group(0, &material.bind_group, &[]);
                    render_pass.set_bind_group(1, &camera.get_bind_group(), &[]);
                    // render_pass.set_bind_group(2, &light.get_bind_group(), &[]);
                    render_pass.set_vertex_buffer(0, cube.unwrap().vertex_buffer.slice(..));
                    render_pass.set_index_buffer(cube.unwrap().index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..cube.unwrap().indices.len() as u32, 0, 0..1);
                }

                queue.submit(std::iter::once(encoder.finish()));
                frame.present();
                window.request_redraw();
            }
            Event::WindowEvent { 
                event: key_event,
                ..
            } => {
                camera_controler.process_window_events(&key_event);
            }
            Event::DeviceEvent { 
                event: key_event,
                ..
            } => {
                camera_controler.process_device_events(&key_event);
            }
            _ => (),
        }

    }).unwrap();
}


