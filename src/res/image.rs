use std::{error::Error, fmt, path::Path};
use gltf::{image::{Source, Format}, Gltf, Image};

use super::buffer::BufferData;

#[derive(Debug, Clone, PartialEq)]
pub struct ImageData {
    pub data: Vec<u8>,
    pub uri: Option<String>,
    pub format: Format,
}

#[derive(Debug)]
pub struct LoadImageDataError {
    message: String,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl fmt::Display for LoadImageDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Image loading error: {}", self.message)
    }
}

impl Error for LoadImageDataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| &**e as _)
    }
}

impl LoadImageDataError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source(message: impl Into<String>, source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

/// Загружает данные изображения из GLTF
pub fn load_gltf_image_data(
    base_path: Option<&Path>,
    buffers: &[BufferData],
    image: Image,
) -> Result<ImageData, LoadImageDataError> {
    let (data, format, uri) = match image.source() {
        Source::Uri { uri, mime_type } => {
            let path = base_path.ok_or_else(|| {
                LoadImageDataError::new("Base path required for external images")
            })?.join(uri);
            
            let data = std::fs::read(&path).map_err(|e| {
                LoadImageDataError::with_source(
                    format!("Failed to read image at {}", path.display()),
                    e
                )
            })?;
            
            let format = match mime_type.as_deref() {
                Some("image/png") => Format::R8G8B8A8,
                Some("image/jpeg") => Format::R8G8B8,
                _ => Format::R8G8B8,
                // _ => {println!("BLEEEEEEEEEEEEEEEH Unsupported image mime type");
                //     return Err(LoadImageDataError::new(format!(
                //     "Unsupported image mime type: {:?}", mime_type
                    
                // )))},
            };
            
            (data, format, Some(uri.to_string()))
        },
        Source::View { view, mime_type } => {
            let buffer = &buffers[view.buffer().index()].data;
            let start = view.offset();
            let end = start + view.length();
            
            if end > buffer.len() {
                return Err(LoadImageDataError::new(format!(
                    "Image view exceeds buffer bounds ({} > {})",
                    end, buffer.len()
                )));
            }
            
            let format = match mime_type {
                "image/png" => Format::R8G8B8A8,
                "image/jpeg" => Format::R8G8B8,
                _ => return Err(LoadImageDataError::new(format!(
                    "Unsupported image mime type: {}", mime_type
                ))),
            };
            
            (buffer[start..end].to_vec(), format, None)
        }
    };
    
    Ok(ImageData {
        data,
        format,
        uri
    })
}

/// Загружает все изображения из GLTF документа
pub fn load_gltf_images(
    gltf: &Gltf,
    base_path: Option<&Path>,
    buffers: &[BufferData],
) -> Result<Vec<ImageData>, LoadImageDataError> {
    gltf.images()
        .map(|image| load_gltf_image_data(base_path, buffers, image))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_embedded_image() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let glb_data = include_bytes!("../../test_assets/textured_cube.glb");
        // let glb_data = include_bytes!("../../test_assets/none_textured_cube.glb");
        temp_file.write_all(glb_data).unwrap();
        
        let gltf = Gltf::open(temp_file.path()).unwrap();
        let buffers = super::super::buffer::load_gltf_buffers(&gltf, None).unwrap();
        let images = load_gltf_images(&gltf, None, &buffers).unwrap();
        
        assert!(!images.is_empty());
        assert!(images[0].uri.is_none());
        assert_eq!(images[0].format, Format::R8G8B8A8);
    }

    #[test]
    fn test_unsupported_mime_type() {
        let dir = tempfile::tempdir().unwrap();
        let gltf_path = dir.path().join("model.gltf");
        
        std::fs::write(
            &gltf_path,
            r#"{
                "images": [{
                    "uri": "texture.webp",
                    "mimeType": "image/webp"
                }]
            }"#
        ).unwrap();
        
        let gltf = Gltf::open(&gltf_path).unwrap();
        let result = load_gltf_images(&gltf, Some(dir.path()), &[]);
        
        assert!(matches!(result, Err(_)));
        if let Err(e) = result {
            assert!(e.to_string().contains("Unsupported image mime type"));
        }
    }
}