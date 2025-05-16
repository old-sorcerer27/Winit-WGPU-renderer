use std::{fmt, path::Path};

use gltf::Error;
use image::GenericImageView;
use wgpu::{Sampler, TextureView};

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
    
    fn load(params: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized {
        todo!()
    }
}

pub type GpuTextureStorage = Storage<GpuTexture>;
pub type GpuTextureHandle = Handle<GpuTexture>;


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
                Ok(image) => {return Ok(image.data)},
                Err(_) => {return Err(LoadTextureDataError::new(format!("Texture load error")))},
            };
        },
        None => {return Err(LoadTextureDataError::new(format!("Texture load error")))},
    };
}