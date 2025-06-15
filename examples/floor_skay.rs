use diploma_thesis::res::{asset_manager::AssetManager, texture};
use gltf::Gltf;
use wgpu::{util::DeviceExt, MemoryHints, PipelineCompilationOptions};
use winit::{
    event::{ElementState, Event, WindowEvent}, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowBuilder}
};
use pollster::block_on;
use glam::Mat4;

use std::{path::Path, time::Instant};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
    test: [f32; 2],
}

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
            include_str!("../examples/shaders/cube.wgsl").into() 
        ),
    });
    
    let vertices = [
        Vertex { position: [-1.0, -1.0,  1.0], color: [1.0, 0.0, 0.0], test: [1.0, 1.0]},
        Vertex { position: [ 1.0, -1.0,  1.0], color: [0.0, 1.0, 0.0], test: [1.0, 1.0]},
        Vertex { position: [ 1.0,  1.0,  1.0], color: [0.0, 0.0, 1.0], test: [1.0, 1.0]},
        Vertex { position: [-1.0,  1.0,  1.0], color: [1.0, 1.0, 0.0], test: [1.0, 1.0]},
        Vertex { position: [-1.0, -1.0, -1.0], color: [1.0, 0.0, 1.0], test: [1.0, 1.0]},
        Vertex { position: [ 1.0, -1.0, -1.0], color: [0.0, 1.0, 1.0], test: [1.0, 1.0]},
        Vertex { position: [ 1.0,  1.0, -1.0], color: [1.0, 1.0, 1.0], test: [1.0, 1.0]},
        Vertex { position: [-1.0,  1.0, -1.0], color: [0.0, 0.0, 0.0], test: [1.0, 1.0]},
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

    let mut camera_state = CameraState::new();
    let mut camera_uniform = CameraUniform::new();

    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
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

    let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Camera Bind Group"),
        layout: &camera_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
    });


    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&camera_bind_group_layout],
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
                         wgpu::VertexAttribute {
                        offset: std::mem::size_of::<[f32; 6]>() as u64,
                        shader_location: 2,
                        format: wgpu::VertexFormat::Float32x2,
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
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float, // Или другой поддерживаемый формат
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less, // Стандартное сравнение (ближние объекты перекрывают дальние)
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: Default::default(),
    });

    let depth_texture = texture::GpuTexture::create_depth_texture(&device, &config, "depth_texture");

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
            }
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
                            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &depth_texture.view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: Default::default(),
                        occlusion_query_set: Default::default(),
                    });

                    let aspect_ratio = config.width as f32 / config.height as f32;
                    camera_state.update();
                    let view_proj = camera_state.calculate_view_projection(aspect_ratio);
                    camera_uniform.update_view_proj(view_proj);
                    queue.write_buffer(
                        &camera_buffer,
                        0,
                        bytemuck::cast_slice(&[camera_uniform]),
                    );
                    
                    render_pass.set_viewport(0.0, 0.0, config.width as f32, config.height as f32, 0.0, 1.0);
                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.set_bind_group(0, &camera_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                }

                queue.submit(std::iter::once(encoder.finish()));
                frame.present();
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    event: winit::event::KeyEvent { 
                        physical_key: keycode,
                        state, 
                        .. 
                    },
                ..
                }, .. } => {
                let is_pressed = state == ElementState::Pressed;
                match keycode {
                    PhysicalKey::Code(code) => {
                        match code {
                            KeyCode::KeyW => camera_state.is_forward_pressed = is_pressed,
                            KeyCode::KeyS  =>camera_state.is_backward_pressed = is_pressed,
                            KeyCode::KeyA => camera_state.is_left_pressed = is_pressed,
                            KeyCode::KeyD  => camera_state.is_right_pressed = is_pressed,
                            KeyCode::Space => camera_state.is_up_pressed = is_pressed,
                            KeyCode::ShiftLeft => camera_state.is_down_pressed = is_pressed,
                            KeyCode::ArrowDown => camera_state.is_rotate_down_pressed = is_pressed,
                            KeyCode::ArrowUp => camera_state.is_rotate_up_pressed = is_pressed,
                            KeyCode::ArrowLeft => camera_state.is_rotate_left_pressed = is_pressed,
                            KeyCode::ArrowRight => camera_state.is_rotate_right_pressed = is_pressed,
                            _ => {}
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }).unwrap();
}




use glam::{Vec3, Quat};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, view_proj: Mat4) {
        self.view_proj = view_proj.to_cols_array_2d();
    }
}

struct CameraState {
    pub position: Vec3,
    pub rotation: Quat,
    pub speed: f32,
    pub rotation_speed: f32,

    pub near: f32,
    pub far: f32,

    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
    pub is_rotate_left_pressed: bool,
    pub is_rotate_right_pressed: bool,
    pub is_rotate_up_pressed: bool,
    pub is_rotate_down_pressed: bool,
}

impl CameraState {
    fn new() -> Self {
         // Начальная позиция - на расстоянии 3 единицы перед кубом по оси Z
        let position = glam::Vec3::new(0.0, 0.0, 3.0);
        
        // Начальный поворот - смотрим вдоль отрицательной оси Z (на куб)
        let rotation = Quat::from_rotation_y(0.0); // Без поворота по Y
        
        Self {
            position,
            rotation,
            speed: 0.1,
            rotation_speed: 0.01,
            near: 0.1,
            far: 100.,

            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_rotate_left_pressed: false,
            is_rotate_right_pressed: false,
            is_rotate_up_pressed: false,
            is_rotate_down_pressed: false,
        }
    }

   fn calculate_view_projection(&self, aspect_ratio: f32) -> Mat4 {
        let projection = Mat4::perspective_rh(
            45.0f32.to_radians(),
            aspect_ratio,
            0.1,
            100.0,
        );

        // Получаем матрицу вращения из кватерниона
        let rotation_matrix = Mat4::from_quat(self.rotation);
        
        // Создаем матрицу трансляции (обратную позиции камеры)
        let translation_matrix = Mat4::from_translation(-self.position);
        
        // Комбинируем вращение и трансляцию для получения view матрицы
        let view = rotation_matrix * translation_matrix;
        
        projection * view
    }

    fn update(&mut self) {
         // Вращение камеры
        if self.is_rotate_left_pressed {
            self.rotation *= Quat::from_rotation_y(self.rotation_speed);
        }
        if self.is_rotate_right_pressed {
            self.rotation *= Quat::from_rotation_y(-self.rotation_speed);
        }
        if self.is_rotate_up_pressed {
            self.rotation *= Quat::from_rotation_x(self.rotation_speed);
        }
        if self.is_rotate_down_pressed {
            self.rotation *= Quat::from_rotation_x(-self.rotation_speed);
        }

        // Нормализуем кватернион, чтобы избежать дрифта
        self.rotation = self.rotation.normalize();

        // Получаем направления вперед, вправо и вверх из кватерниона
        let forward = self.rotation * -Vec3::Z;
        let right = self.rotation * Vec3::X;
        let up = self.rotation * Vec3::Y;

        // Движение камеры
        if self.is_forward_pressed {
            self.position += forward * self.speed;
        }
        if self.is_backward_pressed {
            self.position -= forward * self.speed;
        }
        if self.is_left_pressed {
            self.position -= right * self.speed;
        }
        if self.is_right_pressed {
            self.position += right * self.speed;
        }
        if self.is_up_pressed {
            self.position += up * self.speed;
        }
        if self.is_down_pressed {
            self.position -= up * self.speed;
        }
    }
}