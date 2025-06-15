use diploma_thesis::{core::{renderer::Renderer, window::{self, WindowManager}}, res::texture::{self, get_texture_bind_group_layout}};
use gltf::Gltf;
use winit::event::{Event, WindowEvent};
use pollster::block_on;
use glam::Mat4;

use std::{path::Path, time::Instant};

fn main() {
    env_logger::init();
    let window = WindowManager::new("example", 720, 540);
    block_on(run(window));
}

async fn run(window: WindowManager<'_>) {

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let gltf_path = Path::new("examples/examples_assets/cube_model/scene.gltf");
    let gltf = Gltf::open(gltf_path).unwrap();
    let (mut renderer, meshes) = Renderer::new(&window, &gltf, "examples/examples_assets/cube_model").await.unwrap();
    

    let (capabilities, mut config) = window.get_surface_config(&window.get_adapter().await);
    window.configure_surface(&renderer.device, &config);

    let shader = renderer.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../examples/examples_shaders/cube_copy.wgsl").into() 
        ),
    });
 
    let transform_buffer = renderer.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Transform Buffer"),
        size: std::mem::size_of::<Mat4>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let transform_bind_group_layout = renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    let transform_bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Transform Bind Group"),
        layout: &transform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: transform_buffer.as_entire_binding(),
        }],
    });



    let cube = renderer.assets.meshes.get(meshes[0].clone());
    //     match meshes{
    //     Ok(asset) => {asset[0].clone()},
    //     Err(_) => todo!(),
    // });

    let material = match renderer.assets.materials.get_mut(cube.unwrap().material.clone().unwrap()){
        Some(mat) => mat,
        None => todo!(),
    };

    let texture = match renderer.assets.textures.get_mut(material.base_color_texture.clone().unwrap()){
        Some(text) => text,
        None => todo!(),
    };

    material.create_bind_group(
        &renderer.device, 
        None, 
        Some(&texture.view), 
        Some(&texture.sampler)
    );

    let render_pipeline = renderer.create_render_pipeline(config.format, &[
        &transform_bind_group_layout,
        &get_texture_bind_group_layout(&renderer.device)
    ]);

    let mut depth_texture = texture::GpuTexture::create_depth_texture(&renderer.device, &config, "depth_texture");

    let start_time = Instant::now();

    match window.event_loop {
        Some(event_loop) => {
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
                        window.configure_surface(&renderer.device, &config);
                        depth_texture = texture::GpuTexture::create_depth_texture(&renderer.device, &config, "depth_texture");
                    }
                    Event::WindowEvent {
                            event: WindowEvent::RedrawRequested,
                            ..
                        } => {
                        
                        let elapsed = start_time.elapsed().as_secs_f32();
                        let model = Mat4::from_rotation_x(elapsed) * Mat4::from_rotation_y(elapsed);
                        
                        renderer.queue.write_buffer(
                            &transform_buffer,
                            0,
                            bytemuck::cast_slice(&model.to_cols_array_2d()),
                        );
                    
                    
                        let frame = window.surface().unwrap().get_current_texture().unwrap();
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    
                        let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                    
                        renderer.queue.submit(std::iter::once(encoder.finish()));
                        frame.present();
                        window.window.request_redraw();
                    }
                    _ => (),
                }

            }).unwrap();
        },
        None => todo!(),
    }
}
