use std::{fmt, path::Path};

use gltf::Error;
use image::GenericImageView;
use wgpu::{util::DeviceExt, BindGroupLayout, Sampler, TextureView};

use super::{{buffer::BufferData, image::load_gltf_image_data, load_binary}, storage::Storage, Handle, Resource};


#[derive(Debug, Clone)] 
pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl Resource for GpuTexture {
    type Key = super::TextureKey;
    
    type LoadParams = GpuTexture;
    
    fn load(res: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>>{
        Ok(res) 
    }
}

pub type GpuTextureStorage = Storage<GpuTexture>;
pub type GpuTextureHandle = Handle<GpuTexture>;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

impl GpuTexture {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &Vec<u8>,
        label: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let img = image::load_from_memory(bytes).unwrap();
        Ok(Self::from_image(device, queue, &img, Some(label))?)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self, Error> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats:  &[wgpu::TextureFormat::Rgba8UnormSrgb]
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: Default::default(),
        };

        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }

}

pub fn get_texture_bind_group_layout(
    device: &wgpu::Device,
) -> BindGroupLayout {
    return 
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                   binding: 1,
                   visibility: wgpu::ShaderStages::FRAGMENT,
                   ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                   count: None,
                },
            ],
        label: Some("texture_bind_group_layout"),
    });
}



pub async fn load_texture<'a>(
    file_name: &'a str,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
) ->Result<GpuTexture, Box<dyn std::error::Error>> {
    let data = load_binary(file_name).await?;
    GpuTexture::from_bytes(device, queue, &data, file_name)
}

#[derive(Debug)]
pub struct LoadTextureDataError {
    message: std::string::String,
}

impl fmt::Display for LoadTextureDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Texture loading error: {}", self.message)  // Исправлено сообщение об ошибке
    }
}

impl std::error::Error for LoadTextureDataError {}

impl LoadTextureDataError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub fn load_gltf_texture_source_data(
    base_path: Option<&Path>,
    pbr: gltf::material::PbrMetallicRoughness<'_>,
    buffers: &[BufferData],  
) -> Result<Vec<u8>, LoadTextureDataError> {
    match &pbr.base_color_texture().map(|tex|{tex.texture().source()}) {
        Some(source_image) => {
            match load_gltf_image_data(base_path, buffers, source_image.clone()) {
                Ok(image) => return Ok(image.data),
                Err(_) => return Err(LoadTextureDataError::new(format!("Texture load error"))),
            };
        },
        None => {return Err(LoadTextureDataError::new(format!("Texture load error")))},
    };
}

pub struct UniformTextureBuffer {
    handle: wgpu::Buffer,
    binding_idx: u32,
}

impl UniformTextureBuffer {
    pub fn new(
        device: &wgpu::Device,
        buffer_size: wgpu::BufferAddress,
        binding_idx: u32,
        label: Option<&str>,
    ) -> Self {
        let handle = device.create_buffer(&wgpu::BufferDescriptor {
            size: buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
            label,
        });

        Self {
            handle,
            binding_idx,
        }
    }

    pub fn new_from_bytes(
        device: &wgpu::Device,
        bytes: &[u8],
        binding_idx: u32,
        label: Option<&str>,
    ) -> Self {
        let handle = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            label,
        });

        Self {
            handle,
            binding_idx,
        }
    }

    pub fn handle(&self) -> &wgpu::Buffer {
        &self.handle
    }

    pub fn layout(&self, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: self.binding_idx,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub fn binding(&self) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: self.binding_idx,
            resource: self.handle.as_entire_binding(),
        }
    }
}

pub struct StorageBuffer {
    handle: wgpu::Buffer,
    binding_idx: u32,
}

impl StorageBuffer {
    pub fn new_from_bytes(
        device: &wgpu::Device,
        bytes: &[u8],
        binding_idx: u32,
        label: Option<&str>,
    ) -> Self {
        let handle = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            label,
        });

        Self {
            handle,
            binding_idx,
        }
    }

    pub fn handle(&self) -> &wgpu::Buffer {
        &self.handle
    }

    pub fn layout(
        &self,
        visibility: wgpu::ShaderStages,
        read_only: bool,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: self.binding_idx,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub fn binding(&self) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: self.binding_idx,
            resource: self.handle.as_entire_binding(),
        }
    }
}