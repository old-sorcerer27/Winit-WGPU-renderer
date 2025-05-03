use wgpu::{Device, Queue, TextureView, Sampler};

use super::{storage::Storage, Handle, Resource};

#[derive(Debug)] 
pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>, 
}

impl Resource for Texture {
    type Key = super::TextureKey;
    
    type LoadParams = Texture;
    
    fn load(params: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized {
        todo!()
    }
}

pub type TextureStorage = Storage<Texture>;
pub type TextureHandle = Handle<Texture>;


impl Texture {
    pub fn new(bytes: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bytes,
        }
    }

    pub fn from_gltf_image(
        image: gltf::image::Data,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let width = image.width;
        let height = image.height;
        let bytes = image.pixels;

        let texture = Self::new(bytes, width, height);
        texture.create_gpu_texture(device, queue);
        Ok(texture)
    }

    pub fn create_gpu_texture(&self, device: &Device, queue: &Queue) -> GpuTexture {
        let size = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.width),
                rows_per_image: Some(self.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            ..Default::default()
        });

        GpuTexture {
            texture,
            view,
            sampler,
        }
    }
}




