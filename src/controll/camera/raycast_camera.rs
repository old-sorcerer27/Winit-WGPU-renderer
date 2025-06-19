use glam::{Quat, Vec3, Vec3Swizzles};
use gltf::camera;
use winit::{event::{ElementState, KeyEvent, MouseButton, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

use crate::{math::angle::Angle, scene::{camera::Camera, entity::SceneEntity, transform::Transform}};

pub struct RayCastCameraController {
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
    look_pressed: bool,
    pub previous_mouse_pos: Option<(f32, f32)>,
    pub mouse_pos: (f32, f32),
}

impl Default for RayCastCameraController {
    fn default() -> Self {
        Self {
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
            look_pressed: false,

            previous_mouse_pos: None,
            mouse_pos: (0.0, 0.0),
        }
    }
}

impl RayCastCameraController {
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
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = (position.x as f32, position.y as f32);
                true
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if *button == MouseButton::Right {
                    self.look_pressed = *state == ElementState::Pressed;
                    return true;
                }
                false
            }
            _ => false
        }
    }

    pub fn update_camera(
        &mut self,
        viewport_size: (u32, u32),
        translation_scale: f32,
        camera_entity: &mut SceneEntity,
    ) {
        if let crate::scene::entity::SceneEntityKind::Camera { camera, mut uniform } = &camera_entity.kind {
            // Обработка вращения камеры мышью
            if self.look_pressed {
                if let Some(prev_mouse_pos) = self.previous_mouse_pos {                
                    // Конвертация в локальное пространство камеры
                    let current_dir = self.generate_camera_ray_dir(
                        self.mouse_pos,
                        viewport_size,
                        camera,
                        &camera_entity.transform
                    );
                    let previous_dir = self.generate_camera_ray_dir(
                        prev_mouse_pos,
                        viewport_size,
                        camera,
                        &camera_entity.transform
                    );

                    // Конвертация направлений в сферические координаты
                    let to_spherical = |dir: Vec3| {
                        let pitch = dir.y.atan2(dir.xz().length());
                        let yaw = dir.z.atan2(dir.x);
                        (yaw, pitch)
                    };

                    let (current_yaw, current_pitch) = to_spherical(current_dir);
                    let (prev_yaw, prev_pitch) = to_spherical(previous_dir);

                    // Обновляем вращение в Transform
                    let mut rotation = camera_entity.transform.rotation;
                    rotation = Quat::from_rotation_y(current_yaw - prev_yaw) * rotation;
                    rotation = Quat::from_rotation_x(current_pitch - prev_pitch) * rotation;
                    camera_entity.transform.rotation = rotation.normalize();
                }
            }

            // Обработка перемещения клавиатурой
            {
                let v = |b| if b { 1.0 } else { 0.0 };
                let translation = Vec3::new(
                    translation_scale * (v(self.is_right_pressed) - v(self.is_left_pressed)),
                    translation_scale * (v(self.is_up_pressed) - v(self.is_down_pressed)),
                    translation_scale * (v(self.is_forward_pressed) - v(self.is_backward_pressed)),
                );

                let orientation = self.camera_orientation(&camera_entity.transform);
               camera_entity.transform.position += orientation.right * translation.x
                    + orientation.up * translation.y
                    + orientation.forward * translation.z;
            }

            self.previous_mouse_pos = Some(self.mouse_pos);
        }
    }

    fn generate_camera_ray_dir(
        &self,
        mouse_pos: (f32, f32),
        viewport_size: (u32, u32),
        camera: &Camera,
        camera_transform: &Transform
    ) -> Vec3 {
        let aspect_ratio = viewport_size.0 as f32 / viewport_size.1 as f32;
        let half_height = match camera.rccp {
            Some(params) => params.focus_distance * (0.5 * params.vfov.as_radians()).tan(),
            None => 1.0 * (0.5 * camera.fov.to_radians()).tan(),
        };
        let half_width = aspect_ratio * half_height;

        let x = mouse_pos.0 / viewport_size.0 as f32;
        let y = mouse_pos.1 / viewport_size.1 as f32;

        let orientation = self.camera_orientation(camera_transform);

        let point_on_plane = camera_transform.position
            + match camera.rccp {
                Some(params) => params.focus_distance,
                None => 1.0,
            } * orientation.forward
            + (2.0 * x - 1.0) * half_width * orientation.right
            + (1.0 - 2.0 * y) * half_height * orientation.up;

        (point_on_plane - camera_transform.position).normalize()
    }

    fn camera_orientation(&self, transform: &Transform) -> Orientation {
        let forward = transform.rotation * -Vec3::Z;
        let world_up = Vec3::Y;
        let right = forward.cross(world_up).normalize();
        let up = right.cross(forward).normalize();
        
        Orientation {
            forward,
            right,
            up,
        }
    }
}

struct Orientation {
    forward: Vec3,
    right: Vec3,
    up: Vec3,
}