// use winit::{
//     dpi::PhysicalSize,
//     event::{Event, WindowEvent},
//     event_loop::{self, ControlFlow, EventLoop},
//     window::{Window, WindowBuilder},
// };
// use wgpu::{Adapter, Device, Instance, MemoryHints, Queue, Surface, SurfaceCapabilities, SurfaceConfiguration};
// use std::sync::Arc;

// use crate::res::texture;

// /// Менеджер окна, отвечающий за создание и управление окном приложения
// /// и графической поверхностью (surface)
// pub struct WindowManager<'a> {
//     /// Окно приложения
//     pub window: Arc<Window>,
//     /// Цикл обработки событий
//     pub event_loop: Option<EventLoop<()>>,
//     /// Текущий размер окна
//     pub size: PhysicalSize<u32>,
//     /// Графическая поверхность для рендеринга
//     surface: Option<Surface<'a>>,

// }

// impl<'a> WindowManager<'a> {
//     /// Создает новое окно с указанными параметрами
//     ///
//     /// # Аргументы
//     /// * `title` - Заголовок окна
//     /// * `width` - Начальная ширина окна
//     /// * `height` - Начальная высота окна
//     ///
//     /// # Пример
//     /// ```
//     /// let window_manager = WindowManager::new("My App", 1280, 720);
//     /// ```
//     pub fn new(title: &str, width: u32, height: u32) -> Self {
//         let event_loop = EventLoop::new()
//             .expect("Не удалось создать цикл обработки событий");
        
//         let window = WindowBuilder::new()
//             .with_title(title)
//             .with_inner_size(PhysicalSize::new(width, height))
//             .build(&event_loop)
//             .expect("Не удалось создать окно");

//         Self {
//             window: Arc::new(window),
//             event_loop: Some(event_loop),
//             size: PhysicalSize::new(width, height),
//             surface: None,
//         }
//     }

//     pub fn take_event_loop(self) -> Option<EventLoop<()>>{
//         return self.event_loop;
//     }

//     /// Создает графическую поверхность для рендеринга
//     pub fn create_surface(&mut self, instance: &Instance) {
//         self.surface = Some(
//             unsafe { instance.create_surface(self.window.clone()) }
//                 .expect("Не удалось создать графическую поверхность")
//         );
//     }

//     /// Настраивает поверхность для рендеринга
//     pub fn configure_surface(&mut self, device: &Device, config: &wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>>){
//         let surface = self.surface.as_ref()
//             .expect("Поверхность не была создана");
//         surface.configure(device, &config);
//     }

//     /// Получает графический адаптер
//     pub async fn get_adapter(&self) -> Adapter {
//         let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
//             backends: wgpu::Backends::all(),
//             ..Default::default()
//         });

//         instance.request_adapter(&wgpu::RequestAdapterOptions {
//             power_preference: wgpu::PowerPreference::HighPerformance,
//             compatible_surface: self.surface.as_ref(),
//             force_fallback_adapter: false,
//         })
//         .await
//         .expect("Не удалось найти подходящий графический адаптер")
//     }

//     /// Инициализирует графическое устройство и очередь команд
//     pub async fn init_device(adapter: &Adapter) -> (Device, Queue) {
//         adapter.request_device(
//             &wgpu::DeviceDescriptor {
//                 label: None,
//                 required_features: wgpu::Features::empty(),
//                 required_limits: wgpu::Limits::default(),
//                 memory_hints: MemoryHints::default(),
//                 trace: wgpu::Trace::Off,
//             },
//         )
//         .await
//         .expect("Не удалось создать графическое устройство")
//     }

//     /// Запускает главный цикл приложения
//     pub fn run<F>(&mut self, config: SurfaceConfiguration)
//     where
//         F: 'static + FnMut(Event<()>, &Window, &mut ControlFlow)
//     {
//         let event_loop = self.event_loop.take().unwrap();
//         let mut depth_texture = texture::GpuTexture::create_depth_texture(&device, &config, "depth_texture");

//         event_loop.run( |event, elwt: &winit::event_loop::EventLoopWindowTarget<()>| {
//             match event {
//                 Event::WindowEvent {
//                     event: WindowEvent::CloseRequested,
//                     ..
//                 } => elwt.exit(),
//                 Event::WindowEvent {
//                     event: WindowEvent::Resized(new_size),
//                     ..
//                 } => {
//                     config.width = new_size.width;
//                     config.height = new_size.height;
//                     self.handle_resize(new_size);
                    
//                     self.configure_surface(config.&config);
//                     depth_texture = texture::GpuTexture::create_depth_texture(&renderer.device, &config, "depth_texture");
//                 }
//                 Event::WindowEvent {
//                         event: WindowEvent::RedrawRequested,
//                         ..
//                     } => {
                    
//                     let elapsed = start_time.elapsed().as_secs_f32();
//                     let model = Mat4::from_rotation_x(elapsed) * Mat4::from_rotation_y(elapsed);
                    
//                     renderer.queue.write_buffer(
//                         &transform_buffer,
//                         0,
//                         bytemuck::cast_slice(&model.to_cols_array_2d()),
//                     );
                
                
//                     let frame = window.surface().unwrap().get_current_texture().unwrap();
//                     let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                
//                     let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
//                         label: Some("Render Encoder"),
//                     });
                
//                     {
//                         let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                             label: Some("Render Pass"),
//                             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                                 view: &view,
//                                 resolve_target: None,
//                                 ops: wgpu::Operations {
//                                     load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
//                                     store: wgpu::StoreOp::Store,
//                                 },
//                             })],
//                             timestamp_writes: Default::default(),
//                             occlusion_query_set: Default::default(),
//                             depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
//                                 view: &depth_texture.view,
//                                 depth_ops: Some(wgpu::Operations {
//                                     load: wgpu::LoadOp::Clear(1.0),
//                                     store: wgpu::StoreOp::Store,
//                                 }),
//                                 stencil_ops: None,
//                             }),
//                         });
//                         render_pass.set_viewport( 
//                             0.0, 0.0, 
//                             config.width as f32, config.height as f32, 
//                             0.0, 1.0
//                         );
//                         render_pass.set_pipeline(&render_pipeline);
//                         render_pass.set_bind_group(0, &transform_bind_group, &[]);
//                         render_pass.set_bind_group(1, &material.bind_group, &[]);
//                         render_pass.set_vertex_buffer(0, cube.unwrap().vertex_buffer.slice(..));
//                         render_pass.set_index_buffer(cube.unwrap().index_buffer.slice(..), wgpu::IndexFormat::Uint32);
//                         render_pass.draw_indexed(0..cube.unwrap().indices.len() as u32, 0, 0..1);
//                     }
                
//                     renderer.queue.submit(std::iter::once(encoder.finish()));
//                     frame.present();
//                     window.window.request_redraw();
//                 }
//                 _ => (),
//             }
//         }).unwrap();
//     }

//     /// Обрабатывает изменение размера окна
//     pub fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
//         if new_size.width > 0 && new_size.height > 0 {
//             self.size = new_size;
//         }
//     }

//     /// Возвращает ссылку на поверхность рендеринга
//     pub fn surface(&self) -> Option<&Surface> {
//         self.surface.as_ref()
//     }

// }

// pub fn get_surface_config(surface: Surface, adapter: &Adapter, size: PhysicalSize<u32>) -> (SurfaceCapabilities, SurfaceConfiguration) {
//     let surface_caps = surface.get_capabilities(adapter);
//     let config = wgpu::SurfaceConfiguration {
//         usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//         format: surface_caps.formats[0],
//         width: size.width,
//         height: size.height,
//         present_mode: wgpu::PresentMode::Fifo,
//         desired_maximum_frame_latency: 2,
//         alpha_mode: surface_caps.alpha_modes[0],
//         view_formats: vec![],
//     };
//     return (surface_caps, config);
// }