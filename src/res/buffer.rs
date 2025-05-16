use std::{error::Error, fmt, path::Path};
use gltf::{buffer::Source, Gltf};

#[derive(Debug, Clone, PartialEq)]
pub struct BufferData {
    pub data: Vec<u8>,
    pub uri: Option<String>,
}

#[derive(Debug)]
pub struct LoadBufferDataError {
    message: String,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl fmt::Display for LoadBufferDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Buffer loading error: {}", self.message)
    }
}

impl Error for LoadBufferDataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| &**e as _)
    }
}

impl LoadBufferDataError {
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

/// Загружает все буферы из GLTF файла
///
/// # Аргументы
/// * `gltf` - Загруженный GLTF документ
/// * `base_path` - Базовый путь для поиска внешних ресурсов
///
/// # Пример
/// ```
/// let gltf = Gltf::open("model.gltf")?;
/// let buffers = load_gltf_buffers(&gltf, Some("models/".as_ref()))?;
/// ```
pub fn load_gltf_buffers(
    gltf: &Gltf,
    base_path: Option<&Path>,
) -> Result<Vec<BufferData>, LoadBufferDataError> {
    gltf.buffers()
        .enumerate()
        .map(|(i, buffer)| {
            let uri = match buffer.source() {
                Source::Uri(uri) => Some(uri.to_string()),
                Source::Bin => None,
            };
            
            let data = match buffer.source() {
                Source::Uri(uri) => {
                    let path = base_path.map(|p| p.join(uri))
                        .ok_or_else(|| {
                            LoadBufferDataError::new(format!(
                                "Buffer [{}]: Base path required for external URI '{}'",
                                i, uri
                            ))
                        })?;
                    
                    std::fs::read(&path).map_err(|e| {
                        LoadBufferDataError::with_source(
                            format!("Buffer [{}]: Failed to read '{}'", i, path.display()),
                            e
                        )
                    })?
                },
                Source::Bin => {
                    gltf.blob.clone().ok_or_else(|| {
                        LoadBufferDataError::new(format!(
                            "Buffer [{}]: Missing binary data for embedded buffer",
                            i
                        ))
                    })?
                }
            };
            
            Ok(BufferData { data, uri })
        })
        .collect()
}

/// Преобразует Vec<BufferData> в Vec<Vec<u8>>, сохраняя порядок
pub fn to_vec(buffers: Vec<BufferData>) -> Vec<Vec<u8>> {
    buffers.into_iter().map(|buffer| buffer.data).collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_embedded_buffer() {
        // Создаем временный GLB файл с бинарными данными
        let mut temp_file = NamedTempFile::new().unwrap();
        let glb_data = include_bytes!("../../test_assets/cube.glb");
        temp_file.write_all(glb_data).unwrap();
        
        let gltf = Gltf::open(temp_file.path()).unwrap();
        let buffers = load_gltf_buffers(&gltf, None).unwrap();
        
        assert_eq!(buffers.len(), 1);
        assert!(buffers[0].uri.is_none());
        assert!(!buffers[0].data.is_empty());
    }

    #[test]
    fn test_load_external_buffer() {
        // Создаем временную директорию с gltf и бинарным файлом
        let dir = tempfile::tempdir().unwrap();
        let gltf_path = dir.path().join("model.gltf");
        let bin_path = dir.path().join("buffer.bin");
        
        std::fs::write(&bin_path, b"test buffer data").unwrap();
        std::fs::write(
            &gltf_path,
            r#"
            {
                "buffers": [{
                    "uri": "buffer.bin",
                    "byteLength": 15
                }]
            }
            "#
        ).unwrap();
        
        let gltf = Gltf::open(&gltf_path).unwrap();
        let buffers = load_gltf_buffers(&gltf, Some(dir.path())).unwrap();
        
        assert_eq!(buffers.len(), 1);
        assert_eq!(buffers[0].uri.as_deref(), Some("buffer.bin"));
        assert_eq!(buffers[0].data, b"test buffer data");
    }

    #[test]
    fn test_load_missing_external_buffer() {
        let dir = tempfile::tempdir().unwrap();
        let gltf_path = dir.path().join("model.gltf");
        
        std::fs::write(
            &gltf_path,
            r#"{"buffers": [{"uri": "missing.bin", "byteLength": 15}]}"#
        ).unwrap();
        
        let gltf = Gltf::open(&gltf_path).unwrap();
        let result = load_gltf_buffers(&gltf, Some(dir.path()));
        
        assert!(matches!(result, Err(_)));
        if let Err(e) = result {
            assert!(e.to_string().contains("Failed to read"));
        }
    }

    #[test]
    fn test_to_vec_conversion() {
        let buffers = vec![
            BufferData { data: vec![1, 2, 3], uri: None },
            BufferData { data: vec![4, 5], uri: Some("test.bin".into()) },
        ];
        
        let vecs = to_vec(buffers);
        assert_eq!(vecs, vec![vec![1, 2, 3], vec![4, 5]]);
    }
}


