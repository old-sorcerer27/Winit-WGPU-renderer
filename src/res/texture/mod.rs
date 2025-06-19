pub mod gpu_texture;
pub mod gpu_buffers;
use std::{fmt, path::Path};

use image::RgbaImage;

use crate::res::{buffer::BufferData, image::load_gltf_image_data, load_binary, texture::gpu_texture::GpuTexture};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub offset: u32,
}

impl TextureDescriptor {
    pub fn empty() -> Self {
        Self {
            width: 0_u32,
            height: 0_u32,
            offset: 0xffffffff,
        }
    }
}

pub struct Texture {
    dimensions: (u32, u32),
    data: Vec<[f32; 3]>,
}

impl Texture {
    pub fn new_from_image(path: &str) -> Result<Self, LoadTextureDataError> {
        Self::new_from_scaled_image(path, 1_f32)
    }

    pub fn new_from_scaled_image(path: &str, scale: f32) -> Result<Self, LoadTextureDataError> {
        use std::fs::*;
        use std::io::BufReader;

        let file = File::open(path).unwrap();
        let pixels: RgbaImage = image::load(BufReader::new(file), image::ImageFormat::Jpeg).unwrap().into_rgba8();
        let tex_scale = scale / 255_f32;
        let dimensions = pixels.dimensions();
        let data = pixels
            .pixels()
            .map(|p| -> [f32; 3] {
                [
                    tex_scale * (p[0] as f32),
                    tex_scale * (p[1] as f32),
                    tex_scale * (p[2] as f32),
                ]
            })
            .collect();

        Ok(Self { dimensions, data })
    }

    pub fn new_from_color(color: glam::Vec3) -> Self {
        let data = vec![[color.x, color.y, color.z]];
        let dimensions = (1_u32, 1_u32);

        Self { dimensions, data }
    }

    pub fn as_slice(&self) -> &[[f32; 3]] {
        self.data.as_slice()
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
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
                Ok(image) => return Ok(image.data),
                Err(_) => return Err(LoadTextureDataError::new(format!("Texture load error"))),
            };
        },
        None => {return Err(LoadTextureDataError::new(format!("Texture load error")))},
    };
}