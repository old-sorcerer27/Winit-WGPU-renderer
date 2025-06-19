use glam::Mat4;
use gltf::camera;
use wgpu::util::DeviceExt;

use crate::{core::raytracer::sky::SkyParams, math::{angle::Angle, sphere::Sphere, unit_quad_projection_matrix}, res::{material::{GpuMaterial, Material, RayCastMaterial}, texture::{gpu_buffers::{StorageBuffer, UniformTextureBuffer}, Texture, TextureDescriptor}, vertex::{SimpleVertex, VertexUniforms, VERTICES}}, scene::{camera::{Camera, GpuCamera}, transform::Transform}};

pub mod sky;

pub struct Raytracer {
    pub vertex_uniform_bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub frame_data_buffer: UniformTextureBuffer,
    pub image_bind_group: wgpu::BindGroup,
    pub camera_buffer: UniformTextureBuffer,
    pub sampling_parameter_buffer: UniformTextureBuffer,
    pub hw_sky_state_buffer: StorageBuffer,
    pub parameter_bind_group: wgpu::BindGroup,
    pub scene_bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
    pub latest_render_params: RenderParams,
    pub render_progress: RenderProgress,
    pub frame_number: u32,
}

impl Raytracer {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        scene: &Scene,
        render_params: &RenderParams,
        max_viewport_resolution: u32,
        camera_transform: &Transform,
    ) -> Result<Self, RenderParamsValidationError> {
        match render_params.validate() {
            Ok(_) => {}
            Err(err) => return Err(err),
        }

         let uniforms = VertexUniforms {
            view_projection_matrix: unit_quad_projection_matrix(),
            model_matrix: nalgebra_glm::identity(),
        };
        let vertex_uniform_buffer = UniformTextureBuffer::new_from_bytes(
            device,
            bytemuck::bytes_of(&uniforms),
            0_u32,
            Some("uniforms"),
        );
        let vertex_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[vertex_uniform_buffer.layout(wgpu::ShaderStages::VERTEX)],
                label: Some("uniforms layout"),
            });
        let vertex_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &vertex_uniform_bind_group_layout,
            entries: &[vertex_uniform_buffer.binding()],
            label: Some("uniforms bind group"),
        });

        let frame_data_buffer =
            UniformTextureBuffer::new(device, 16_u64, 0_u32, Some("frame data buffer"));

        let image_buffer = {
            let buffer = vec![[0_f32; 3]; max_viewport_resolution as usize];
            StorageBuffer::new_from_bytes(
                device,
                bytemuck::cast_slice(buffer.as_slice()),
                1_u32,
                Some("image buffer"),
            )
        };

        let image_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    frame_data_buffer.layout(wgpu::ShaderStages::FRAGMENT),
                    image_buffer.layout(wgpu::ShaderStages::FRAGMENT, false),
                ],
                label: Some("image layout"),
            });
        let image_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &image_bind_group_layout,
            entries: &[frame_data_buffer.binding(), image_buffer.binding()],
            label: Some("image bind group"),
        });

        let camera_buffer = {
            let camera = GpuCamera::new(&render_params.camera, camera_transform);

            UniformTextureBuffer::new_from_bytes(
                device,
                unsafe {
                std::slice::from_raw_parts(
                    &camera as *const _ as *const u8,
                    std::mem::size_of::<GpuCamera>()
                )},
                0_u32,
                Some("camera buffer"),
            )
        };

        let sampling_parameter_buffer = UniformTextureBuffer::new(
            device,
            std::mem::size_of::<GpuSamplingParams>() as wgpu::BufferAddress,
            1_u32,
            Some("sampling parameter buffer"),
        );

        let hw_sky_state_buffer = {
            let sky_state = render_params.sky.to_sky_state()?;

            StorageBuffer::new_from_bytes(
                device,
                bytemuck::bytes_of(&sky_state),
                2_u32,
                Some("sky state buffer"),
            )
        };

        let parameter_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    camera_buffer.layout(wgpu::ShaderStages::FRAGMENT),
                    sampling_parameter_buffer.layout(wgpu::ShaderStages::FRAGMENT),
                    hw_sky_state_buffer.layout(wgpu::ShaderStages::FRAGMENT, true),
                ],
                label: Some("parameter layout"),
            });

        let parameter_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &parameter_bind_group_layout,
            entries: &[
                camera_buffer.binding(),
                sampling_parameter_buffer.binding(),
                hw_sky_state_buffer.binding(),
            ],
            label: Some("parameter bind group"),
        });

        let (scene_bind_group_layout, scene_bind_group) = {
            let sphere_buffer = StorageBuffer::new_from_bytes(
                device,
                // bytemuck::cast_slice(scene.spheres.as_slice()),
                unsafe {
                std::slice::from_raw_parts(
                    &scene.spheres  as *const _  as *const u8,
                    std::mem::size_of::<Sphere>() * scene.spheres.len()
                )},
                0_u32,
                Some("scene buffer"),
            );

            let mut global_texture_data: Vec<[f32; 3]> = Vec::new();
            let mut material_data: Vec<GpuMaterial> = Vec::with_capacity(scene.materials.len());

            for material in scene.materials.iter() {
                let gpu_material = match material {
                    RayCastMaterial::Lambertian { albedo } => {
                        GpuMaterial::lambertian(albedo, &mut global_texture_data)
                    }
                    RayCastMaterial::Metal { albedo, fuzz } => {
                        GpuMaterial::metal(albedo, *fuzz, &mut global_texture_data)
                    }
                    RayCastMaterial::Dielectric { refraction_index } => {
                        GpuMaterial::dielectric(*refraction_index)
                    }
                    RayCastMaterial::Checkerboard { odd, even } => {
                        GpuMaterial::checkerboard(odd, even, &mut global_texture_data)
                    }
                    RayCastMaterial::Emissive { emit } => {
                        GpuMaterial::emissive(emit, &mut global_texture_data)
                    }
                };

                material_data.push(gpu_material);
            }

            let material_buffer = StorageBuffer::new_from_bytes(
                device,
                bytemuck::cast_slice(material_data.as_slice()),
                1_u32,
                Some("materials buffer"),
            );

            let texture_buffer = StorageBuffer::new_from_bytes(
                device,
                bytemuck::cast_slice(global_texture_data.as_slice()),
                2_u32,
                Some("textures buffer"),
            );

            let light_indices: Vec<u32> = scene
                .spheres
                .iter()
                .enumerate()
                .filter(|(_, s)| {
                    matches!(
                        scene.materials[s.material_idx as usize],
                        RayCastMaterial::Emissive { .. }
                    )
                })
                .map(|(idx, _)| idx as u32)
                .collect();

            let light_buffer = StorageBuffer::new_from_bytes(
                device,
                bytemuck::cast_slice(light_indices.as_slice()),
                3_u32,
                Some("lights buffer"),
            );

            let scene_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        sphere_buffer.layout(wgpu::ShaderStages::FRAGMENT, true),
                        material_buffer.layout(wgpu::ShaderStages::FRAGMENT, true),
                        texture_buffer.layout(wgpu::ShaderStages::FRAGMENT, true),
                        light_buffer.layout(wgpu::ShaderStages::FRAGMENT, true),
                    ],
                    label: Some("scene layout"),
                });
            let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &scene_bind_group_layout,
                entries: &[
                    sphere_buffer.binding(),
                    material_buffer.binding(),
                    texture_buffer.binding(),
                    light_buffer.binding(),
                ],
                label: Some("scene bind group"),
            });

            (scene_bind_group_layout, scene_bind_group)
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../shaders/raycast/raytracer.wgsl").into()),
            label: Some("raytracer.wgsl"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[
                &vertex_uniform_bind_group_layout,
                &image_bind_group_layout,
                &parameter_bind_group_layout,
                &scene_bind_group_layout,
            ],
            push_constant_ranges: &[],
            label: Some("raytracer layout"),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vsMain"),
                buffers: &[SimpleVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fsMain"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                cull_mode: Some(wgpu::Face::Back),
                // Requires Features::DEPTH_CLAMPING
                conservative: false,
                unclipped_depth: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("raytracer pipeline"),
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
            label: Some("VertexInput buffer"),
        });

        let render_progress = RenderProgress::new();

        let frame_number = 1_u32;

        Ok(Self {
            vertex_uniform_bind_group,
            frame_data_buffer,
            image_bind_group,
            camera_buffer,
            sampling_parameter_buffer,
            hw_sky_state_buffer,
            parameter_bind_group,
            scene_bind_group,
            vertex_buffer,
            pipeline,
            latest_render_params: render_params.clone(),
            render_progress,
            frame_number,
        })
    }

    pub fn render_frame<'a>(
        &'a mut self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        {
            let gpu_sampling_params = self
                .render_progress
                .next_frame(&self.latest_render_params.sampling);

            queue.write_buffer(
                &self.sampling_parameter_buffer.handle(),
                0,
                bytemuck::cast_slice(&[gpu_sampling_params]),
            );
        }

        {
            let viewport_size = self.latest_render_params.viewport_size;
            let frame_number = self.frame_number;
            let frame_data = [viewport_size.0, viewport_size.1, frame_number];
            queue.write_buffer(
                &self.frame_data_buffer.handle(),
                0,
                bytemuck::cast_slice(&frame_data),
            );
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.vertex_uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.image_bind_group, &[]);
        render_pass.set_bind_group(2, &self.parameter_bind_group, &[]);
        render_pass.set_bind_group(3, &self.scene_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        let num_vertices = VERTICES.len() as u32;
        render_pass.draw(0..num_vertices, 0..1);

        self.frame_number += 1_u32;
    }

    pub fn set_render_params(
        &mut self,
        queue: &wgpu::Queue,
        render_params: &RenderParams,
        camera_transform: &Transform,
    ) -> Result<(), RenderParamsValidationError> {
        if *render_params == self.latest_render_params {
            return Ok(());
        }

        match render_params.validate() {
            Ok(_) => {}
            Err(err) => return Err(err),
        }

        {
            let sky_state = render_params.sky.to_sky_state()?;
            queue.write_buffer(
                &self.hw_sky_state_buffer.handle(),
                0,
                unsafe {
                std::slice::from_raw_parts(
                    &sky_state as *const _ as *const u8,
                    std::mem::size_of::<Transform>()
                )}
            );
        }

        {
            let camera = GpuCamera::new(&render_params.camera, &camera_transform);
            queue.write_buffer(
                &self.camera_buffer.handle(), 
                0, 
                unsafe {
                std::slice::from_raw_parts(
                    &camera as *const _ as *const u8,
                    std::mem::size_of::<Transform>()
                )
            });
        }

        self.latest_render_params = render_params.clone();

        self.render_progress.reset();

        Ok(())
    }

    pub fn progress(&self) -> f32 {
        self.render_progress.accumulated_samples() as f32
            / self.latest_render_params.sampling.max_samples_per_pixel as f32
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RenderParamsValidationError {
    #[error("max_samples_per_pixel ({0}) is not a multiple of num_samples_per_pixel ({1})")]
    MaxSampleCountNotMultiple(u32, u32),
    #[error("viewport_size elements cannot be zero: ({0}, {1})")]
    ViewportSize(u32, u32),
    #[error("vfov must be between 0..=90 degrees")]
    VfovOutOfRange(f32),
    #[error("aperture must be between 0..=1")]
    ApertureOutOfRange(f32),
    #[error("focus_distance must be greater than zero")]
    FocusDistanceOutOfRange(f32),
    #[error(transparent)]
    HwSkyModelValidationError(#[from] hw_skymodel::rgb::Error),
}

pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub materials: Vec<RayCastMaterial>,
}


#[derive(Clone, Copy, PartialEq)]
pub struct SamplingParams {
    pub max_samples_per_pixel: u32,
    pub num_samples_per_pixel: u32,
    pub num_bounces: u32,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            max_samples_per_pixel: 256_u32,
            num_samples_per_pixel: 1_u32,
            num_bounces: 8_u32,
        }
    }
}

struct RenderProgress {
    accumulated_samples_per_pixel: u32,
}

impl RenderProgress {
    pub fn new() -> Self {
        Self {
            accumulated_samples_per_pixel: 0_u32,
        }
    }

    pub fn next_frame(&mut self, sampling_params: &SamplingParams) -> GpuSamplingParams {
        let current_accumulated_samples = self.accumulated_samples_per_pixel;
        let next_accumulated_samples =
            sampling_params.num_samples_per_pixel + current_accumulated_samples;

        // Initial state: no samples have been accumulated yet. This is the first frame
        // after a reset. The image buffer's previous samples should be cleared by
        // setting clear_accumulated_samples to 1_u32.
        if current_accumulated_samples == 0_u32 {
            self.accumulated_samples_per_pixel = next_accumulated_samples;
            GpuSamplingParams {
                num_samples_per_pixel: sampling_params.num_samples_per_pixel,
                num_bounces: sampling_params.num_bounces,
                accumulated_samples_per_pixel: next_accumulated_samples,
                clear_accumulated_samples: 1_u32,
            }
        }
        // Progressive render: accumulating samples in the image buffer over multiple
        // frames.
        else if next_accumulated_samples <= sampling_params.max_samples_per_pixel {
            self.accumulated_samples_per_pixel = next_accumulated_samples;
            GpuSamplingParams {
                num_samples_per_pixel: sampling_params.num_samples_per_pixel,
                num_bounces: sampling_params.num_bounces,
                accumulated_samples_per_pixel: next_accumulated_samples,
                clear_accumulated_samples: 0_u32,
            }
        }
        // Completed render: we have accumulated max_samples_per_pixel samples. Stop rendering
        // by setting num_samples_per_pixel to zero.
        else {
            GpuSamplingParams {
                num_samples_per_pixel: 0_u32,
                num_bounces: sampling_params.num_bounces,
                accumulated_samples_per_pixel: current_accumulated_samples,
                clear_accumulated_samples: 0_u32,
            }
        }
    }

    pub fn reset(&mut self) {
        self.accumulated_samples_per_pixel = 0_u32;
    }

    pub fn accumulated_samples(&self) -> u32 {
        self.accumulated_samples_per_pixel
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSamplingParams {
    num_samples_per_pixel: u32,
    num_bounces: u32,
    accumulated_samples_per_pixel: u32,
    clear_accumulated_samples: u32,
}

#[derive(Clone, PartialEq)]
pub struct RenderParams {
    pub camera: Camera,
    pub sky: SkyParams,
    pub sampling: SamplingParams,
    pub viewport_size: (u32, u32),
}

impl RenderParams {
    fn validate(&self) -> Result<(), RenderParamsValidationError> {
        match self.camera.rccp {
            Some(rccp) => {
                    if self.sampling.max_samples_per_pixel % self.sampling.num_samples_per_pixel != 0 {
                return Err(RenderParamsValidationError::MaxSampleCountNotMultiple(
                    self.sampling.max_samples_per_pixel,
                    self.sampling.num_samples_per_pixel,
                ));
            }

            if self.viewport_size.0 == 0_u32 || self.viewport_size.1 == 0_u32 {
                return Err(RenderParamsValidationError::ViewportSize(
                    self.viewport_size.0,
                    self.viewport_size.1,
                ));
            }

            if !(Angle::degrees(0.0)..=Angle::degrees(90.0)).contains(&rccp.vfov) {
                return Err(RenderParamsValidationError::VfovOutOfRange(
                    self.camera.fov.to_degrees(),
                ));
            }

            if !(0.0..=1.0).contains(&rccp.aperture) {
                return Err(RenderParamsValidationError::ApertureOutOfRange(
                    rccp.aperture,
                ));
            }

            if rccp.focus_distance < 0.0 {
                return Err(RenderParamsValidationError::FocusDistanceOutOfRange(
                    rccp.focus_distance,
                ));
            }

            Ok(())
        },
            None => todo!(),
        }
    }
}