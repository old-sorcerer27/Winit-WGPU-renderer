use std::ops::Range;


use glam::Vec3;
use wgpu::{BindGroup, Buffer};

use crate::{res::vertex::Vertex, scene::{camera::{Camera, CameraUniform}, light::{get_light_bind_group_layout, Light}}};

use super::transform::Transform;

pub enum SceneEntityKind {
    Object {
        // model: Model,
        model: Vertex
    },
    Camera {
        camera: Camera,
        uniform: CameraUniform
    },
    Light{
        light: Light
    },
    
}

pub struct SceneEntity {
    pub kind: SceneEntityKind,
    pub transform: Transform,
    pub buffer: Buffer,
    pub visible: bool,
}



impl SceneEntity {
    // pub fn new_object_from_model<T: Device>(
    //     model: Model,
    //     file_name: String,
    //     device: &wgpu::Device,) -> Self {
    //         let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             label: Some(&format!("{:?} Vertex Buffer", file_name)),
    //             contents: bytemuck::cast_slice(&vertices),
    //             usage: wgpu::BufferUsages::VERTEX,
    //         });
    //         let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             label: Some(&format!("{:?} Index Buffer", file_name)),
    //             contents: bytemuck::cast_slice(&indices),
    //             usage: wgpu::BufferUsages::INDEX,
    //         });
    //     Self {
    //         kind: SceneEntityKind::Object {vertex_buffer: todo!(), index_buffer: todo!() },
    //         transform: Transform::default(),
    //         visible: true,
    //     }
    // }

    pub fn new_object(
        device: &wgpu::Device,
        position: Vec3,
        rotation: glam::Quat,
        scale: Vec3,
        vertex: Vertex,
    ) -> Self {
        let mut transform = Transform::new( 
            position, 
            rotation, 
            scale,
        );

        let buffer = Transform::create_buffer(device, transform);

        Self {
            kind: SceneEntityKind::Object {
                model: vertex, 
                // uniform
            },
            transform,
            buffer,
            visible: true,
        }
    }
    

    pub fn new_camera(
        device: &wgpu::Device,
        position: Vec3,
        rotation: glam::Quat,
        scale: Vec3,
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let transform = Transform::new( 
            position, 
            rotation, 
            scale,
        );
        
        let mut camera = Camera::new( fov, aspect, near, far);

        let mut uniform = CameraUniform::new();
        let view_proj = transform.calculate_view_projection(aspect);
        uniform.update_view_proj(view_proj);

        let buffer = CameraUniform::create_buffer(device, uniform);

        camera.create_bind_group(
            device, 
            &get_light_bind_group_layout(device), 
            buffer.clone()
        );

        Self {
            kind: SceneEntityKind::Camera {
                camera, 
                uniform
            },
            transform,
            buffer,
            visible: true,
        }
    }

    // pub fn new_light(
    //     device: &wgpu::Device,
    //     position: Vec3,
    //     rotation: glam::Quat,
    //     scale: Vec3,

    //     light_type: super::light::LightType,
    //     color: Vec3,
    //     shadows_enabled: bool,
    //     intensity: f32,
    // ) -> Self {
    //     let transform = Transform::new(
    //         position, 
    //         rotation, 
    //         scale,
    //     );

    //     let buffer = 

    //     let mut light = Light::new( light_type, color, shadows_enabled, intensity);
    //     light.create_bind_group(
    //         device, 
    //         Some(&get_light_bind_group_layout(device)), 
    //         buffer,
    //     );
        
    //     Self {
    //         kind: SceneEntityKind::Light {light},
    //         transform,
    //         visible: true,
    //     }
    // }

    pub fn get_bind_group(&self)-> Option<BindGroup>{
        match &self.kind {
            SceneEntityKind::Object { model } => todo!(),
            SceneEntityKind::Camera { camera , ..} => return camera.bind_group.clone(),
            SceneEntityKind::Light { light } => return light.bind_group.clone(),
        }
    }
}
