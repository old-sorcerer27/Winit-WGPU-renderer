#[cfg(test)]
mod tests {
    use gltf::Gltf;

    #[test]
    fn run() {
        let gltf = Gltf::open("assets/scene.gltf").unwrap();
        print!(
            "Buffers count {} Images count {}",
            gltf.buffers().len(),
            gltf.images().count(),
        );
        for scene in gltf.scenes() {
            for node in scene.nodes() {
                println!(
                    "Node #{} has {} children",
                    node.index(),
                    node.children().count(),
                );
            }
        }
    }
}

