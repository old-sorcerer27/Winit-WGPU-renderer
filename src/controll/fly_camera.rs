use glam::{Quat, Vec3};
use wgpu::{wgc::device::queue, Queue};
use winit::{event::{ElementState, KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

use crate::scene::{camera::Camera, entity::SceneEntity, transform::{Transform}};

pub struct CameraController {
    speed: f32,
    sensivity: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_rotate_left_pressed: bool,
    is_rotate_right_pressed: bool,
    is_rotate_up_pressed: bool,
    is_rotate_down_pressed: bool,
}


impl CameraController {
    pub fn new(speed: f32, sensivity: f32) -> Self {
        Self {
            speed,
            sensivity,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_rotate_left_pressed: false,
            is_rotate_right_pressed: false,
            is_rotate_up_pressed: false,
            is_rotate_down_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
           WindowEvent::KeyboardInput {
                event: KeyEvent { 
                    physical_key: keycode,
                    state, 
                    .. 
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    PhysicalKey::Code(code) => {
                        match code {
                            KeyCode::KeyW => self.is_forward_pressed = is_pressed,
                            KeyCode::KeyS  =>self.is_backward_pressed = is_pressed,
                            KeyCode::KeyA => self.is_left_pressed = is_pressed,
                            KeyCode::KeyD  => self.is_right_pressed = is_pressed,
                            KeyCode::Space => self.is_up_pressed = is_pressed,
                            KeyCode::ShiftLeft => self.is_down_pressed = is_pressed,
                            KeyCode::ArrowDown => self.is_rotate_down_pressed = is_pressed,
                            KeyCode::ArrowUp => self.is_rotate_up_pressed = is_pressed,
                            KeyCode::ArrowLeft => self.is_rotate_left_pressed = is_pressed,
                            KeyCode::ArrowRight => self.is_rotate_right_pressed = is_pressed,
                             _ => return false,
                        }
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera_transform(
        &self,
        camera_transform: &mut Transform,
        // delta_time: f32 
    ) {
       if self.is_rotate_left_pressed {
            camera_transform.rotation *= Quat::from_rotation_y(-self.sensivity);
        }
        if self.is_rotate_right_pressed {
            camera_transform.rotation *= Quat::from_rotation_y(self.sensivity);
        }
        // if self.is_rotate_up_pressed {
        //     camera_transform.rotation *= Quat::from_rotation_x(self.sensivity);
        // }
        // if self.is_rotate_down_pressed {
        //     camera_transform.rotation *= Quat::from_rotation_x(-self.sensivity);
        // }

           if self.is_rotate_up_pressed || self.is_rotate_down_pressed {
            let (yaw, mut pitch, roll) = camera_transform.rotation.to_euler(glam::EulerRot::YXZ);
            if self.is_rotate_up_pressed {
                pitch += self.sensivity;
            }
            if self.is_rotate_down_pressed {
                pitch -= self.sensivity;
            }
            pitch = pitch.clamp(
                -std::f32::consts::PI / 2.0 + 0.1,
                std::f32::consts::PI / 2.0 - 0.1    
            );
            camera_transform.rotation = Quat::from_euler(glam::EulerRot::YXZ, yaw, pitch, roll);
        }


        camera_transform.rotation = camera_transform.rotation.normalize();

        let forward = camera_transform.rotation * -Vec3::Z;
        let right = camera_transform.rotation * Vec3::X;
        let up = camera_transform.rotation * Vec3::Y;

        if self.is_forward_pressed {
            camera_transform.position += forward * self.speed;
        }
        if self.is_backward_pressed {
            camera_transform.position -= forward * self.speed;
        }
        if self.is_left_pressed {
            camera_transform.position -= right * self.speed;
        }
        if self.is_right_pressed {
            camera_transform.position += right * self.speed;
        }
        if self.is_up_pressed {
            camera_transform.position += up * self.speed;
        }
        if self.is_down_pressed {
            camera_transform.position -= up * self.speed;
        }
    }


    pub fn update_camera(&mut self, camera_entity: &mut SceneEntity, aspect: f32, queue: &Queue) {
        if let crate::scene::entity::SceneEntityKind::Camera { camera: _, mut uniform } = camera_entity.kind {
            let transform = &mut camera_entity.transform;
            self.update_camera_transform(transform);
            let view_proj = transform.calculate_view_projection(aspect);
            uniform.update_view_proj(view_proj);
            queue.write_buffer(
                &camera_entity.buffer,
                0,
                bytemuck::cast_slice(&[uniform]),
            );
        }
    }

}

