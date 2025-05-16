#[cfg(test)]
mod tests {
    pub fn run() {
        let window = WindowManager::new("winit 0.19.12", 360, 360);
        let gltf_path = path::Path::new("../../test_assets/cube.glb");
        let gltf = Gltf::open(gltf_path).unwrap();
        let renderer = Renderer::new(&window, &gltf, "assets/").await?;
    }
   
}


