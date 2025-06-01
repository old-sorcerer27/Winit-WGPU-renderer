use gltf::Primitive;
use wgpu::{util::DeviceExt, Device};
use super::{material::Material, vertex::Vertex, Handle, MeshKey, Resource};

/// Меш, содержащий вершины, индексы и идентификатор материала.
#[derive(Debug, Clone)] 
pub struct Mesh {
    pub material: Option<Handle<Material>>,
    pub indices: Vec<u32>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer:  wgpu::Buffer,
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
            material: mesh.material,
            indices: mesh.indices,
            vertex_buffer: mesh.vertex_buffer,
            index_buffer: mesh.index_buffer,
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
        material: Option<Handle<Material>>,
        device: Device,
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
        let vertices: Vec<Vertex> = positions.zip(normals)
            .zip(tex_coords)
            .map(|((position, normal), tex_coord)| Vertex {
                position: position.into(),
                normal: normal.into(),
                tex_coord: tex_coord.into(),
            })
            .collect();

        // Чтение индексов
        let indices: Vec<u32> = reader.read_indices()
            .ok_or("GLTF: No indices in primitive")?
            .into_u32()
            .collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        println!("{:?}", vertices);
        println!("{:?}", indices);

        Ok(Self {
            material,
            indices,
            vertex_buffer,
            index_buffer,
        })
    }

    pub fn from_matrix() {
        
    }
}



