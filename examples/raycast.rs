use std::time::Instant;
use diploma_thesis::{controll::camera::raycast_camera::RayCastCameraController, core::raytracer::{
    sky::SkyParams, Raytracer, RenderParams, SamplingParams, Scene}, gui::GpuContext, math::{angle::Angle, sphere::Sphere}, res::{material::RayCastMaterial, texture::Texture}, scene::{camera::RayCastCameraParams, entity::SceneEntity}};
use glam::{Quat, Vec3};
use wgpu::StoreOp;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() {

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut context = pollster::block_on(GpuContext::new(&window));

    let viewport_size = {
        let viewport = window.inner_size();
        (viewport.width, viewport.height)
    };
    let max_viewport_resolution = window
        .available_monitors()
        .map(|monitor| -> u32 {
            let size = monitor.size();
            size.width * size.height
        })
        .max()
        .expect("There should be at least one monitor available");

    let scene = scene();


    let look_from = Vec3::new(-10.0, 2.0, -4.0);
    let look_at = Vec3::new(0.0, 1.0, 0.0);
    let focus_distance = (look_at - look_from).length();
    let aspect_ratio = window.inner_size().width as f32 / window.inner_size().height as f32;
// ... предыдущий код остается без изменений до этой части ...

let mut camera = SceneEntity::new_camera(
    &context.device, 
    look_from, 
    Quat::from_rotation_y(0.0), 
    Vec3::ONE, 
    45., 
    aspect_ratio, 
    0.1,
    100.,
    Some(RayCastCameraParams{
        aperture: 1.0,
        focus_distance,
        vfov: Angle::degrees(45.),
    })
);
let mut camera_controller = RayCastCameraController::default();

// Извлекаем camera и uniform из структуры один раз
let (camera_data, _camera_uniform) = match &camera.kind {
    diploma_thesis::scene::entity::SceneEntityKind::Camera { camera, uniform } => (camera.clone(), uniform.clone()),
    _ => panic!("Expected camera entity"),
};

let mut render_params = RenderParams {
    camera: camera_data,
    sky: SkyParams::default(),
    sampling: SamplingParams::default(),
    viewport_size,
};

let mut raytracer = Raytracer::new(
    &context.device,
    &context.surface_config,
    &scene,
    &render_params,
    max_viewport_resolution,
    &camera.transform
)
.expect("The default values should be selected correctly");


    let mut last_time = Instant::now();

event_loop.run( |event, _elwt: &winit::event_loop::EventLoopWindowTarget<()>| {
    match event {
        Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
        } => {
                if physical_size.width > 0 && physical_size.height > 0 {
                    render_params.viewport_size =
                        (physical_size.width, physical_size.height);
                    context.surface_config.width = physical_size.width;
                    context.surface_config.height = physical_size.height;
                    context
                        .surface
                        .configure(&context.device, &context.surface_config);
                }
            }
    
         Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
          } => {
            let dt = last_time.elapsed().as_secs_f32();
            let now = Instant::now();
            camera_controller.update_camera(render_params.viewport_size, 2.0 * dt, &mut camera);
            last_time = now;

            // Обновляем camera_data в render_params
            if let diploma_thesis::scene::entity::SceneEntityKind::Camera { camera, uniform: _ } = &camera.kind {
                render_params.camera = camera.clone();
            }

            match raytracer.set_render_params(&context.queue, &render_params, &camera.transform) {
                Err(e) => eprintln!("Error setting render params: {e}"),
                Ok(_) => {}
            }

                let frame = match context.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("Surface error: {:?}", e);
                        return;
                    }
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.012,
                                    g: 0.012,
                                    b: 0.012,
                                    a: 1.0,
                                }),
                                store: StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        label: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    raytracer.render_frame(&context.queue, &mut render_pass);

                }

                context.queue.submit(Some(encoder.finish()));
                frame.present();
                window.request_redraw();
             }
         Event::WindowEvent { event, .. } => {
            camera_controller.process_events(&event);
         }
        _ => {}
    }
}).unwrap();
}


fn scene() -> Scene {
    let materials = vec![
        RayCastMaterial::Checkerboard {
            even: Texture::new_from_color(Vec3::new(0.5, 0.7, 0.8)),
            odd: Texture::new_from_color(Vec3::new(0.9, 0.9, 0.9)),
        },
        RayCastMaterial::Lambertian {
            albedo: Texture::new_from_image("examples/assets/jpeg/moon.jpeg")
                .expect("Hardcoded path should be valid"),
        },
        RayCastMaterial::Metal {
            albedo: Texture::new_from_color(Vec3::new(1.0, 0.85, 0.57)),
            fuzz: 0.4,
        },
        RayCastMaterial::Dielectric {
            refraction_index: 1.5,
        },
        RayCastMaterial::Lambertian {
            albedo: Texture::new_from_image("examples/assets/jpeg/earthmap.jpeg")
                .expect("Hardcoded path should be valid"),
        },
        RayCastMaterial::Emissive {
            emit: Texture::new_from_scaled_image("examples/assets/jpeg/sun.jpeg", 50.0)
                .expect("Hardcoded path should be valid"),
        },
        RayCastMaterial::Lambertian {
            albedo: Texture::new_from_color(Vec3::new(0.3, 0.9, 0.9)),
        },
        RayCastMaterial::Emissive {
            emit: Texture::new_from_color(Vec3::new(50.0, 0.0, 0.0)),
        },
        RayCastMaterial::Emissive {
            emit: Texture::new_from_color(Vec3::new(0.0, 50.0, 0.0)),
        },
        RayCastMaterial::Emissive {
            emit: Texture::new_from_color(Vec3::new(0.0, 0.0, 50.0)),
        },
    ];

    let spheres = vec![
        Sphere::new(Vec3::new(0.0, -500.0, -1.0), 500.0, 0_u32),
        // left row
        Sphere::new(Vec3::new(-5.0, 1.0, -4.0), 1.0, 7_u32),
        Sphere::new(Vec3::new(0.0, 1.0, -4.0), 1.0, 8_u32),
        Sphere::new(Vec3::new(5.0, 1.0, -4.0), 1.0, 9_u32),
        // middle row
        Sphere::new(Vec3::new(-5.0, 1.0, 0.0), 1.0, 2_u32),
        Sphere::new(Vec3::new(0.0, 1.0, 0.0), 1.0, 3_u32),
        Sphere::new(Vec3::new(5.0, 1.0, 0.0), 1.0, 6_u32),
        // right row
        Sphere::new(Vec3::new(-5.0, 0.8, 4.0), 0.8, 1_u32),
        Sphere::new(Vec3::new(0.0, 1.2, 4.0), 1.2, 4_u32),
        Sphere::new(Vec3::new(5.0, 2.0, 4.0), 2.0, 5_u32),
    ];

    Scene { spheres, materials }
}