use glam::{Vec2, Vec3};
use gltf::Primitive;
use super::{material::Material, Handle, MeshKey, Resource};

/// Вершина меша с позицией, нормалью и текстурными координатами.
/// Реализует `bytemuck::Pod` и `bytemuck::Zeroable` для эффективной работы с GPU.
#[derive(Debug, Clone, Copy, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coord: Vec2,
}

// Безопасные реализации для работы с GPU буферами
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

/// Меш, содержащий вершины, индексы и идентификатор материала.
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material_id: Option<usize>,
    pub material: Option<Handle<Material>>,
}

impl Resource for Mesh {
    type Key = MeshKey;
    type LoadParams = Mesh;
    
    /// Загружает меш из параметров загрузки.
    ///
    /// # Пример
    /// ```
    /// let mesh = Mesh::load(existing_mesh)?;
    /// ```
    fn load(mesh: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            vertices: mesh.vertices,
            indices: mesh.indices,
            material_id: mesh.material_id,
            material: mesh.material,
            
        })
    }
}

impl Mesh {
    
    /// Создаёт меш из GLTF-примитива.
    ///
    /// # Аргументы
    /// * `primitive` - GLTF примитив
    /// * `buffers` - Список буферов GLTF файла
    ///
    /// # Пример
    /// ```
    /// let mesh = Mesh::from_gltf_primitive(&primitive, &buffers)?;
    /// ```
    pub fn from_gltf_primitive(
        primitive: &Primitive,
        buffers: &[Vec<u8>], 
        material: Option<Handle<Material>>
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        // Чтение атрибутов с обработкой ошибок
        let positions = reader.read_positions()
            .ok_or("GLTF: No positions in primitive")?;
        let normals = reader.read_normals()
            .ok_or("GLTF: No normals in primitive")?;
        let tex_coords = reader.read_tex_coords(0)
            .map(|v| v.into_f32())
            .ok_or("GLTF: No texture coordinates in primitive")?;

        // Сборка вершин
        let vertices = positions.zip(normals)
            .zip(tex_coords)
            .map(|((position, normal), tex_coord)| Vertex {
                position: position.into(),
                normal: normal.into(),
                tex_coord: tex_coord.into(),
            })
            .collect();

        // Чтение индексов
        let indices = reader.read_indices()
            .ok_or("GLTF: No indices in primitive")?
            .into_u32()
            .collect();

        Ok(Self {
            vertices,
            indices,
            material_id: primitive.material().index(),
            material
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gltf::Gltf;
    use std::fs::File;
    use std::io::Read;

    /// Загружает тестовый GLB файл
    fn load_test_glb() -> Gltf {
        let mut file = File::open("test_assets/cube.glb").unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        Gltf::from_slice(&data).unwrap()
    }

    #[test]
    fn test_from_gltf_primitive() {
        // 1. Загружаем тестовый файл
        let gltf = load_test_glb();
        let primitive = gltf.meshes().next()
            .and_then(|m| m.primitives().next())
            .expect("No primitives in test file");

        // 2. Загружаем буферы
        let buffers = gltf.buffers()
            .map(|b| vec![0u8; b.length()])
            .collect::<Vec<_>>();

        // 3. Тестируем создание меша
        let mesh = Mesh::from_gltf_primitive(&primitive, &buffers, Option::None);
        assert!(mesh.is_ok(), "Failed to create mesh from primitive");

        let mesh = mesh.unwrap();
        assert!(!mesh.vertices.is_empty(), "Mesh has no vertices");
        assert!(!mesh.indices.is_empty(), "Mesh has no indices");
    }

    #[test]
    fn test_vertex_layout() {
        // Проверяем, что Vertex соответствует ожиданиям шейдера
        let _vertex = Vertex::default();
        assert_eq!(std::mem::size_of::<Vertex>(), 32); // vec3 + vec3 + vec2
        assert_eq!(std::mem::align_of::<Vertex>(), 4);
    }
}