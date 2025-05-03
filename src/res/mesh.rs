use gltf::Primitive;

use super::{ MeshKey, Resource, Vertex};

pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl Resource for Mesh {
    type Key = MeshKey;
    type LoadParams = Mesh; 
    
    fn load(path: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>> {
        // Здесь будет реальная загрузка меша
        Ok(Self {
            vertices: vec![[0.0, 0.0, 0.0]],
            indices: vec![0],
        })
    }
}

impl Mesh {
    pub fn from_gltf_primitive(
        primitive: &Primitive,
        buffers: &[gltf::buffer::Data],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let positions = reader.read_positions()
            .ok_or("GLTF: No positions")?;
        let normals = reader.read_normals()
            .ok_or("GLTF: No normals")?;
        let tex_coords = reader.read_tex_coords(0)
            .map(|v| v.into_f32())
            .ok_or("GLTF: No tex coords")?;

        let vertices: Vec<[f32; 3]> = positions.zip(normals)
            .zip(tex_coords)
            .map(|((position, normal), tex_coord)| {
                Vertex {
                    position: position.into(),
                    normal: normal.into(),
                    tex_coord: tex_coord.into(),
                }
            })
            .collect();

        let indices = reader.read_indices()
            .ok_or("GLTF: No indices")?
            .into_u32()
            .collect();

        Ok(Self {
            vertices,
            indices,
        })
    }
}

