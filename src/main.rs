use wgpu::{util::DeviceExt, MemoryHints, PipelineCompilationOptions};
use winit::{
    application::ApplicationHandler, event::{Event, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowId}
};
use pollster::block_on;
use glam::Mat4;

use std::time::Instant;

#[derive(Default)]
pub struct App {
    window: Option<winit::window::Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("demo");
            let window = event_loop.create_window(win_attr).unwrap();
            self.window = Some(window);
        }
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            // Treat exit
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => ()
        }
    }
}

fn main() {
    // env_logger::init();
    // let event_loop = EventLoop::new().unwrap();
    // let window = event_loop.create_window(Window::default_attributes()).unwrap();
    // block_on(run(event_loop, window));


    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("run app error.");  
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = unsafe { instance.create_surface(&window) }.unwrap();

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
            include_str!("../src/shaders/cube.wgsl").into() 
        ),
    });
    
       // Вершины куба (8 вершин)
       let vertices = [
        // Передняя грань
        Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
        Vertex { position: [ 0.5, -0.5,  0.5], color: [0.0, 1.0, 0.0] },
        Vertex { position: [ 0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0] },
        Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 1.0, 0.0] },
        // Задняя грань
        Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 0.0, 1.0] },
        Vertex { position: [ 0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0] },
        Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 1.0, 1.0] },
        Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 0.0, 0.0] },
    ];
    
    // Индексы (12 треугольников)
    let indices: [u16; 36] = [
        0, 1, 2, 2, 3, 0, // Перед
        1, 5, 6, 6, 2, 1, // Право
        7, 6, 5, 5, 4, 7, // Зад
        4, 0, 3, 3, 7, 4, // Лево
        4, 5, 1, 1, 0, 4, // Низ
        3, 2, 6, 6, 7, 3, // Верх
    ];
    
    // Буферы
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    
    // Uniform буфер для матрицы
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


    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&transform_bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
    });

    let start_time = Instant::now();

    event_loop.run( |event, elwt| {
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
            }
            Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                let elapsed = start_time.elapsed().as_secs_f32();
                let model = Mat4::from_rotation_y(elapsed * 0.5) * Mat4::from_rotation_y(elapsed * 0.3);

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
                        depth_stencil_attachment: None,
                        timestamp_writes: Default::default(),
                        occlusion_query_set: Default::default(),
                    });

                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_bind_group(0, &transform_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                }

                queue.submit(std::iter::once(encoder.finish()));
                frame.present();
                window.request_redraw();
            }
            _ => (),
        }
    }).unwrap();
}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}