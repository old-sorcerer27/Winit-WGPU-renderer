use winit::{
    dpi::PhysicalSize,
    event::Event,
    event_loop::{self, ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use wgpu::{Adapter, Device, Instance, MemoryHints, Queue, Surface, SurfaceCapabilities, SurfaceConfiguration};
use std::sync::Arc;

/// Менеджер окна, отвечающий за создание и управление окном приложения
/// и графической поверхностью (surface)
pub struct WindowManager<'a> {
    /// Окно приложения
    pub window: Arc<Window>,
    /// Цикл обработки событий
    event_loop: Option<EventLoop<()>>,
    /// Текущий размер окна
    pub size: PhysicalSize<u32>,
    /// Графическая поверхность для рендеринга
    surface: Option<Surface<'a>>,
}

impl<'a> WindowManager<'a> {
    /// Создает новое окно с указанными параметрами
    ///
    /// # Аргументы
    /// * `title` - Заголовок окна
    /// * `width` - Начальная ширина окна
    /// * `height` - Начальная высота окна
    ///
    /// # Пример
    /// ```
    /// let window_manager = WindowManager::new("My App", 1280, 720);
    /// ```
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        let event_loop = EventLoop::new()
            .expect("Не удалось создать цикл обработки событий");
        
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(PhysicalSize::new(width, height))
            .build(&event_loop)
            .expect("Не удалось создать окно");

        Self {
            window: Arc::new(window),
            event_loop: Some(event_loop),
            size: PhysicalSize::new(width, height),
            surface: None,
        }
    }

    pub fn take_event_loop(self) -> Option<EventLoop<()>>{
        return self.event_loop;
    }

    /// Создает графическую поверхность для рендеринга
    pub fn create_surface(&mut self, instance: &Instance) {
        self.surface = Some(
            unsafe { instance.create_surface(self.window.clone()) }
                .expect("Не удалось создать графическую поверхность")
        );
    }

    /// Настраивает поверхность для рендеринга
    pub fn configure_surface(&self, device: &Device, adapter: &Adapter) -> (SurfaceCapabilities, SurfaceConfiguration) {
        let surface = self.surface.as_ref()
            .expect("Поверхность не была создана");

        let surface_caps = surface.get_capabilities(adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: self.size.width,
            height: self.size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(device, &config);
        return (surface_caps, config);
    }

    /// Получает графический адаптер
    pub async fn get_adapter(&self, instance: &Instance) -> Adapter {
        instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: self.surface.as_ref(),
            force_fallback_adapter: false,
        })
        .await
        .expect("Не удалось найти подходящий графический адаптер")
    }

    /// Инициализирует графическое устройство и очередь команд
    pub async fn init_device(adapter: &Adapter) -> (Device, Queue) {
        adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: MemoryHints::default(),
                trace: wgpu::Trace::Off,
            },
        )
        .await
        .expect("Не удалось создать графическое устройство")
    }

    /// Запускает главный цикл приложения
    pub fn run<F>(mut self, event_handler: F)
    where
        F: 'static + FnMut(Event<()>, &Window, &mut ControlFlow)
    {
        let event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);
        });

    }

    /// Обрабатывает изменение размера окна
    pub fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            // Поверхность будет переконфигурирована в рендерере
        }
    }

    /// Возвращает ссылку на поверхность рендеринга
    pub fn surface(&self) -> Option<&Surface> {
        self.surface.as_ref()
    }
}