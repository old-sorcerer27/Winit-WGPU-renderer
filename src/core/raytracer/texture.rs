use image::RgbaImage;
use thiserror::Error;

pub struct Texture {
    dimensions: (u32, u32),
    data: Vec<[f32; 3]>,
}

impl Texture {
    pub fn new_from_image(path: &str) -> Result<Self, TextureError> {
        Self::new_from_scaled_image(path, 1_f32)
    }

    pub fn new_from_scaled_image(path: &str, scale: f32) -> Result<Self, TextureError> {
        use std::fs::*;
        use std::io::BufReader;

        let file = File::open(path)?;
        let pixels: RgbaImage =
            image::load(BufReader::new(file), image::ImageFormat::Jpeg)?.into_rgba8();
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

#[derive(Error, Debug)]
pub enum TextureError {
    #[error(transparent)]
    FileIoError(#[from] std::io::Error),
    #[error(transparent)]
    ImageLoadError(#[from] image::ImageError),
}
