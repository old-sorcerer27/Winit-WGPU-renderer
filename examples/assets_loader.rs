use std::path::Path;

use diploma_thesis::res::asset_manager::AssetManager;
use gltf::Gltf;
use pollster::block_on;
use wgpu::MemoryHints;
use winit::{event_loop::EventLoop, window::{Window, WindowBuilder}};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    block_on(run(window));
}

async fn run(window: Window) {
    
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


    let mut assets = AssetManager::new();
    
    let gltf_path = Path::new("examples/examples_assets/cube_model/scene.gltf");
    let gltf = Gltf::open(gltf_path).unwrap();
    let meshes = match assets.load_gltf_meshes(&gltf, "examples/examples_assets/cube_model/", &device, &queue) {
        Ok(meshes) => meshes,
        Err(_) => todo!(),
    };

    let gltf_path = Path::new("examples/examples_assets/none_textured_cube.glb");
    let gltf = Gltf::open(gltf_path).unwrap();

    let meshes2 = assets.load_gltf_meshes(&gltf, "examples/examples_assets/", &device, &queue);

    // let meshes2 = match assets.load_gltf_meshes(&gltf, "examples/examples_assets/", &device, &queue) 
    // {
    //     Ok(meshes) => meshes,
    //     Err(_) => todo!(),
    // };

    // meshes.append(assets.load_gltf_meshes(&gltf, base_path,  &device, &queue));


    // println!("{:?}", assets);
    // println!("{:?}", meshes);

}