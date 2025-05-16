use std::path;

use gltf::Gltf;
use winit::window;

use crate::scene::GpuScene;

use super::{renderer::Renderer, window::WindowManager};

pub struct App<'a> {
    pub window: WindowManager<'a>,
    pub renderer: Renderer,
    pub scene: GpuScene,
}

impl<'a> App<'a> {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let window = WindowManager::new("winit 0.19.12", 360, 360);
        let gltf_path = path::Path::new("../../test_assets/cube.glb");
        let gltf = Gltf::open(gltf_path).unwrap();
        let renderer = Renderer::new(&window, &gltf, "assets/").await?;
        Ok(Self {
            window,
            renderer,
            scene: GpuScene::default(),
        })
    }

    pub fn run() {

    }
}

