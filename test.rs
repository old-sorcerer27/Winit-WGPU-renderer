// PLEASE NOTE: Not all files are included, spisifically alot of the wgpu helper functions
 
// ===================================================================================================
// Model.rs
// ===================================================================================================
 
pub struct Model 
{
    pub meshes: Vec<Mesh>,
    pub materials: HashMap<u64, Material>
}
 
impl Model
{
    pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static>
    {
        Vertex::layout()
    }
 
    pub fn from_glb(bytes: &[u8], device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self, String> 
    {
        let data = match gltf::Gltf::from_slice(bytes)
        {
            Ok(ok) => ok,
            Err(e) => return Err(e.to_string())
        };
 
        let mut buffer_data: Vec<Vec<u8>> = Vec::new();
        for buffer in data.buffers() 
        {
            match buffer.source() 
            {
                gltf::buffer::Source::Bin => 
                {
                    if let Some(blob) = data.blob.as_deref() 
                    {
                        buffer_data.push(blob.into());
                    };
                }
                gltf::buffer::Source::Uri(_uri) => 
                {
                    return Err("URI's are not implemented yet".into())
                }
            }
        }
 
        let Some(mesh) = data.meshes().next() else { return Err("glb file must have a mesh".into()); };
        let meshes: Result<_, String> = mesh.primitives().map(|p| Mesh::from_primitive(p, &buffer_data, device)).collect();
 
        let mut materials = HashMap::new();
        for m in mesh.primitives().map(|p| p.material())
        {
            let id = get_gltf_material_id(m.clone());
            if materials.get(&id).is_none()
            {
                let material = match Material::from_glb(m, &buffer_data, device, queue) 
                {
                    Ok(ok) => ok,
                    Err(e) => return Err(e)
                };
                materials.insert(id, material);
            }
        }
 
        match meshes
        {
            Ok(ok) => Ok(Self { 
                meshes: ok,
                materials,
            }),
            Err(err) => Err(err)
        }
    }
}
 
pub struct Mesh 
{
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub material_id: u64,
}
 
impl Mesh 
{
    pub fn from_primitive<'a>(primitive: gltf::Primitive<'a>, buffer_data: &Vec<Vec<u8>>, device: &wgpu::Device) -> Result<Self, String> 
    {
        let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
 
        let mut vertices: Vec<Vertex> = match reader.read_positions()
        {
            Some(p) => p.map(|p| Vertex::new(p, Vec2::ZERO, Vec3::ZERO)),
            None => return Err("Mesh needs vertex positions".into())
        }.collect();
 
        println!("{:?}", vertices);
 
        match reader.read_colors(0)
        {
            Some(cs) => 
            {
                cs.into_rgb_f32().enumerate().for_each(|(i, c)| vertices[i].color = c);
            },
            None => {}
        };
 
        match reader.read_tex_coords(0)
        {
            Some(uvs) => 
            {
                uvs.into_f32().enumerate().for_each(|(i, uv)| { vertices[i].uv = uv});
            },
            None => {}
        }
 
        let indices: Vec<u32> = match reader.read_indices()
        {
            Some(i) => i.into_u32(),
            None => return Err("Mesh does not have an index buffer".into())
        }.collect();
 
        let vertex_buffer = make_vertex_buffer(VertexBufferDescriptor {
            data: &vertices,
            device,
            additional_usages: None,
            label: None
        });
 
        let index_buffer = make_index_buffer(IndexBufferDescriptor {
            data: IndexType::U32(&indices),
            device,
            additional_usages: None,
            label: None
        });
 
        let material_id = get_gltf_material_id(primitive.material());
 
        Ok(Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            material_id,
        })
    }
}
 
pub struct Texture
{
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}
 
impl Texture
{
    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
    {
        make_bind_group_layout(device, None, &[
            EntryType::Texture { 
                visibility: wgpu::ShaderStages::FRAGMENT, 
                view_dim: wgpu::TextureViewDimension::D2, 
                sample_type: wgpu::TextureSampleType::Float { filterable: true } 
            },
            EntryType::Sampler { 
                visibility: wgpu::ShaderStages::FRAGMENT, 
                binding_type: wgpu::SamplerBindingType::Filtering 
            }
        ])
    }
 
    pub fn new(texture: wgpu::Texture, device: &wgpu::Device) -> Self 
    {
        let view = make_default_texture_view(&texture);
        let sampler = make_pixel_sampler(device);
        let layout = Self::layout(device);
        let bind_group = make_bind_group(device, &layout, None, [
            wgpu::BindingResource::TextureView(&view),
            wgpu::BindingResource::Sampler(&sampler)
        ]);
 
        Self 
        {
            texture,
            view,
            sampler,
            bind_group
        }
    }
}
 
pub struct Material 
{
    pub color: Color,
    pub texture: Option<Texture>
}
 
impl Material
{
    pub fn from_glb(mat: gltf::Material, buffer_data: &Vec<Vec<u8>>, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self, String>
    {
        let color = mat.pbr_metallic_roughness().base_color_factor();
        let texture = match mat.pbr_metallic_roughness().base_color_texture()
        {
            Some(texture) => 
            {
                let source = texture.texture().source().source();
                match source
                {
                    Source::View { view, mime_type } => 
                    {
                        let parent_buffer_data = &buffer_data[view.buffer().index()];
                        let data = &parent_buffer_data[view.offset()..view.offset() + view.length()];
                        let mime_type = mime_type.replace('/', ".");
                        let image = image::load_from_memory_with_format(
                                data,
                                image::ImageFormat::from_path(mime_type).unwrap(),
                            ).unwrap();
                        
                        let rgba = image.to_rgba8();
                        let texture = make_rgba_texture(MakeRgbaTextureDescriptor {
                            rgba: &rgba,
                            device,
                            queue,
                            label: None,
                            additional_usages: None
                        });
 
                        Some(Texture::new(texture, device))
                    },
                    Source::Uri { uri: _, mime_type: _ } => return Err("Texture URI's not supported".into()),
                }
            }
            None => None,
        };
 
        Ok(Self 
        {
            color: Color::from_vec4(color.into()),
            texture,
        })
    }
}
 
fn get_gltf_material_id(mat: gltf::Material) -> u64
{
    let mut hasher = DefaultHasher::new();
    mat.index().hash(&mut hasher);
    hasher.finish()
}
 
#[derive(ecs::Resource)]
pub struct ModelRenderer
{
    color_pipeline: wgpu::RenderPipeline,
    texture_pipeline: wgpu::RenderPipeline,
 
    camera_uniform: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    
    color_uniform: wgpu::Buffer,
    color_bind_group: wgpu::BindGroup,
 
    // TEMP
    model: Model,
}
 
impl ModelRenderer
{
    pub fn new(state: &WgpuState) -> Self 
    {
        let device = &state.device();
 
        let camera_uniform = make_uniform_buffer(UniformBufferDescriptor {
            label: None,
            device,
            data: Mat4::ZERO,
            additional_usages: Some(BufferUsages::COPY_DST)
        });
 
        let camera_layout = make_bind_group_layout(&state.device(), None, &[
            EntryType::Uniform(ShaderStages::VERTEX),
        ]);
 
        let camera_bind_group = make_bind_group(&state.device, &camera_layout, None, [
            camera_uniform.as_entire_binding(),
        ]);
 
        let color_uniform = make_uniform_buffer(UniformBufferDescriptor {
            label: None,
            device,
            data: Vec4::ZERO,
            additional_usages: Some(BufferUsages::COPY_DST)
        });
 
        let color_layout = make_bind_group_layout(&state.device(), None, &[
            EntryType::Uniform(ShaderStages::VERTEX),
        ]);
 
        let color_bind_group = make_bind_group(&state.device, &color_layout, None, [
            color_uniform.as_entire_binding(),
        ]);
 
        let color_pipeline = make_color_render_pipeline(state, &camera_layout, &color_layout);
        let texture_pipeline = make_texture_render_pipeline(state, &camera_layout, &color_layout);
 
        let model = Model::from_glb(include_bytes!("../../assets/models/test_slab.glb"), &device, &state.queue()).unwrap();
 
        Self 
        {
            color_pipeline,
            texture_pipeline,
            camera_uniform,
            camera_bind_group,
            color_uniform,
            color_bind_group,
            model
        }
    }
 
    pub fn draw(&self, state: &WgpuState, view_proj: &Mat4, depth_texture: &DepthTexture, view: &wgpu::TextureView)
    {
        state.queue().write_buffer(&self.camera_uniform, 0, bytemuck::bytes_of(view_proj));
 
        for mesh in &self.model.meshes
        {
            let mut encoder = make_command_encoder(&state.device());
        
            let material = self.model.materials.get(&mesh.material_id).unwrap();
            state.queue().write_buffer(&self.color_uniform, 0, bytemuck::bytes_of(&material.color.as_vec4()));
 
            let mut render_pass = make_render_pass(&mut encoder, view, Some(depth_texture));
 
            if let Some(texture) = &material.texture
            {
                render_pass.set_pipeline(&self.texture_pipeline);
                render_pass.set_bind_group(2, &texture.bind_group, &[]);
            } 
            else 
            {
                render_pass.set_pipeline(&self.color_pipeline);
            }
 
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.color_bind_group, &[]);
 
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            
            drop(render_pass);
            state.queue().submit(std::iter::once(encoder.finish()));
        }
    }
}
 
pub fn make_color_render_pipeline(state: &WgpuState, camera_layout: &wgpu::BindGroupLayout, color_layout: &wgpu::BindGroupLayout) -> wgpu::RenderPipeline
{
    let shader = &state.device().create_shader_module(include_spirv!(env!("color_mesh_shader.spv")));
    make_render_pipeline(&state.device(), state.surface_config(), &RenderPipelineInfo {
        shader,
        vs_main: "vs_main",
        fs_main: "fs_main",
        opacity: Opacity::Opaque,
        vertex_buffers: &[&Model::vertex_layout()],
        bind_groups: &[&camera_layout, &color_layout],
        label: None,
        has_depth_texture: true
    })
}
 
pub fn make_texture_render_pipeline(state: &WgpuState, camera_layout: &wgpu::BindGroupLayout, color_layout: &wgpu::BindGroupLayout) -> wgpu::RenderPipeline
{
    let shader = &state.device().create_shader_module(include_spirv!(env!("texture_mesh_shader.spv")));
    let model_texture_layout = Texture::layout(&state.device());
 
    make_render_pipeline(&state.device(), state.surface_config(), &RenderPipelineInfo {
        shader,
        vs_main: "vs_main",
        fs_main: "fs_main",
        opacity: Opacity::Opaque,
        vertex_buffers: &[&Model::vertex_layout()],
        bind_groups: &[&camera_layout, &color_layout, &model_texture_layout],
        label: None,
        has_depth_texture: true
    })
}
 
// ===================================================================================================
// Texture.rs
// ===================================================================================================
#[derive(Debug, Clone, Copy)]
pub struct MakeRgbaTextureDescriptor<'a>
{
    pub rgba: &'a ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub label: Option<&'a str>,
    pub additional_usages: Option<wgpu::TextureUsages>,
}
 
pub fn make_rgba_texture(desc: MakeRgbaTextureDescriptor) -> wgpu::Texture
{
    let MakeRgbaTextureDescriptor {
        rgba,
        device,
        queue,
        label,
        additional_usages
    } = desc;
 
    let dim = rgba.dimensions();
 
    let usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | match additional_usages {
        Some(u) => u,
        None => wgpu::TextureUsages::empty()
    };
 
    let texture_size = wgpu::Extent3d {
        width: dim.0,
        height: dim.1,
        depth_or_array_layers: 1,
    };
 
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage,
        label,
        view_formats: &[],
    });
 
    let copy_texture = wgpu::ImageCopyTexture {
        texture: &texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All
    };
 
    let data_layout = wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(4 * dim.0),
        rows_per_image: Some(dim.1),
    };
 
    queue.write_texture(copy_texture, &rgba, data_layout, texture_size);
 
    texture
}