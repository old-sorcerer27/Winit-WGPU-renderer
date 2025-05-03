use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop, window::{Window, WindowId}};

use crate::{res::asset::AssetManager, scene::Scene};

use super::{renderer::Renderer, window::WindowManager};

pub struct App<'a> {
    pub window: WindowManager<'a>,
    pub renderer: Renderer<'a>,
    pub scene: Scene,
    pub assets: AssetManager,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let window = WindowManager::new();
        let renderer = Renderer::new(&window);
        Self {
            window,
            renderer,
            scene: Scene::default(),
            assets: ResourceManager::new(),
        }
    }
}


pub struct App2 {
    window: Option<winit::window::Window>,
}

impl ApplicationHandler for App2 {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("demo");
            let window = event_loop.create_window(win_attr).unwrap();
            self.window = Some(window);
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {}
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("run app error.");
}