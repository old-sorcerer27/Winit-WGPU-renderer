pub enum UniformKind {
    Object {
        model: Model,
    },
    Camera {
        camera: Camera,
    },
    Light{
        light: Light
    },
    
}