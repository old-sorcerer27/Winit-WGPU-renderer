// use std::path::Path;

// use gltf::Gltf;
// use winit::{dpi::PhysicalSize, event_loop::{EventLoop, EventLoopWindowTarget}};

// use crate::scene::AppScene;

// use super::{renderer::Renderer, window::WindowManager};

// /// Главный класс приложения, объединяющий окно, рендерер и сцену
// pub struct App<'a> {
//     /// Менеджер окна
//     pub window: WindowManager<'a>,
//     /// Рендерер
//     pub renderer: Renderer,
//     /// Текущая сцена на GPU
//     pub scene: AppScene,
// }

// impl<'a> App<'a> {
//     /// Создает новое приложение
//     pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
//         // 1. Создаем окно
//         let window = WindowManager::new("3D Renderer", 1280, 720);
        
//         // 2. Загружаем GLTF модель
//         let gltf_path = Path::new("assets/models/cube.glb");
//         let gltf = Gltf::open(gltf_path)?;
        
//         // 3. Инициализируем рендерер
//         let (renderer, meshes) = Renderer::new(&window, &gltf, "assets/").await?;
        
//         // 4. Создаем сцену по умолчанию
//         let scene = AppScene::default();

//         Ok(Self {
//             window,
//             renderer,
//             scene,
//         })
//     }

//     /// Запускает главный цикл приложения
//     pub fn run(mut self) {        
//         // Запускаем главный цикл
//         self.window.run(move |event, _, control_flow| {
//             match event {
//                 winit::event::Event::WindowEvent { event, .. } => {
//                     self.handle_window_event(event, control_flow);
//                 }
//                 _ => {}
//             }
//         });
//     }

    
//     /// Обновляет состояние приложения
//     pub fn update(&mut self, delta_time: f32) {
//         self.scene.update(delta_time);
//     }


    // fn handle_window_event(
    //     &mut self,
    //     event: winit::event::WindowEvent,
    //     // control_flow: &mut winit::event_loop::ControlFlow,
    //     elwt: EventLoopWindowTarget<()>
    // ) {
    //     match event {
    //         winit::event::WindowEvent::CloseRequested => {
    //             // *control_flow = winit::event_loop::ControlFlow::Exit;
    //             elwt.exit();
    //         }
    //         winit::event::WindowEvent::Resized(size) => {
    //             self.window.handle_resize(size);
    //             // self.renderer.resize(size.width, size.height);
    //         }
    //         winit::event::WindowEvent::RedrawRequested => {
    //             self.renderer.render_scene(&self.scene);
    //         }
    //         _ => {}
    //     }
    // }


    // /// Обрабатывает изменение размера окна
    // pub fn resize(&mut self, size: PhysicalSize<u32>) {
    //     self.window.handle_resize(size);
    //     self.renderer.resize(size.width, size.height);
    // }
// }

