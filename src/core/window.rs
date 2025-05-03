use winit::{
    dpi::PhysicalSize, event::Event, event_loop::{ControlFlow, EventLoop}, window::Window
};
use wgpu::{Surface, Instance, Adapter, Device, Queue};
use std::sync::Arc;

pub struct WindowManager<'a> {
    pub window: Arc<Window>,
    pub event_loop: Option<EventLoop<()>>,
    pub size: PhysicalSize<u32>,
    pub surface: Option<Surface<'a>>,
}

impl<'a> WindowManager<'a> {
    pub fn new(title: &str, width: u32, height: u32) -> Self {

        let event_loop = EventLoop::new();
        // let window = WindowBuilder::new()
        //     .with_title("winit 0.30")
        //     .build(&event_loop)?;

        Self {
            window: Arc::new(window),
            event_loop: Some(event_loop),
            size: PhysicalSize::new(width, height),
            surface: None,
        }
    }

    pub fn create_surface(&mut self, instance: &Instance) {
        self.surface = Some(unsafe { 
            instance.create_surface(self.window.clone()).unwrap() 
        });
    }

    pub fn get_adapter(&self, instance: &Instance) -> Adapter {
        pollster::block_on(
            instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: self.surface.as_ref(),
                force_fallback_adapter: false,
            })
        ).expect("Failed to find suitable GPU adapter")
    }

    pub fn init_device(&self, adapter: &Adapter) -> (Device, Queue) {
        pollster::block_on(
            adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
        ).expect("Failed to create GPU device")
    }

    pub fn run<F>(mut self, mut event_handler: F)
    where
        F: 'static + FnMut(Event<()>, &Window, &mut ControlFlow)
    {
        let event_loop = self.event_loop.take().unwrap();
        let window = self.window.clone();

        event_loop.run(move |event, _, control_flow| {
            event_handler(event, &window, control_flow);
        });
    }

    pub fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            if let Some(surface) = &self.surface {
                // Ресайз поверхности будет обработан в рендерере
            }
        }
    }
}