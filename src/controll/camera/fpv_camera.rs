use glam::{EulerRot, Quat, Vec3};
use wgpu::Queue;
use winit::{event::{DeviceEvent, ElementState, KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

use crate::scene::{entity::SceneEntity, transform::{Transform}};

pub struct FpvCameraController {
    speed: f32,
    sensitivity: f32,
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

    mouse_delta: Option<(f32, f32)>,
    last_mouse_pos: Option<(f32, f32)>,
}


impl FpvCameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
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

            mouse_delta: None,
            last_mouse_pos: None,
        }
    }

     pub fn process_window_events(&mut self, event: &WindowEvent) -> bool {
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

    pub fn process_device_events(&mut self, event: &DeviceEvent) -> bool {
        match event {
           DeviceEvent::MouseMotion { 
            delta
        } => {
                self.mouse_delta = Some((delta.0 as f32, delta.1 as f32));
                true
            }
            _ => false,
        }
    }

    pub fn update_camera_transform(
        &mut self,
        camera_transform: &mut Transform,
        // delta_time: f32 
    ) {
        // let actual_move_speed = self.move_speed * delta_time;
        // let actual_rotation_speed = self.rotation_speed * delta_time;

        let mut translation = Vec3::ZERO;
        
        if self.is_forward_pressed {
            translation -= Vec3::Z * self.speed;
        }
        if self.is_backward_pressed {
            translation += Vec3::Z * self.speed;
        }
        if self.is_left_pressed {
            translation -= Vec3::X * self.speed;
        }
        if self.is_right_pressed {
            translation += Vec3::X * self.speed;
        }
        if self.is_up_pressed {
            translation += Vec3::Y * self.speed;
        }
        if self.is_down_pressed {
            translation -= Vec3::Y * self.speed;
        }

        camera_transform.position += camera_transform.rotation * translation;

        if let Some((dx, dy)) = self.mouse_delta {
            let delta_yaw = dx * self.sensitivity;
            let delta_pitch = dy * self.sensitivity;
            
            let (mut yaw, mut pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
            yaw -= delta_yaw;
            pitch = (pitch - delta_pitch).clamp(
                -std::f32::consts::FRAC_PI_2 + 0.1,  
                std::f32::consts::FRAC_PI_2 - 0.1     
            );
            
            camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
            
            self.mouse_delta = None;
        }
    }




    pub fn update_camera(&mut self, camera_entity: &mut SceneEntity, queue: &Queue) {
        if let crate::scene::entity::SceneEntityKind::Camera { camera, mut uniform } = &camera_entity.kind {
            let transform = &mut camera_entity.transform;
            self.update_camera_transform(transform);
            let view_proj = transform.calculate_view_projection(camera);
            uniform.update_view_proj(view_proj);
            queue.write_buffer(
                &camera_entity.buffer,
                0,
                bytemuck::cast_slice(&[uniform]),
            );
        }
    }

}

