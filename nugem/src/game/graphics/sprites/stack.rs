use super::SpriteTextureAtlas;

macro_rules! prefixed_label {
    ($name:ident) => {
        concat!("sprites::drawer::", stringify!($name))
    };
}

const VERTEX_SHADER_ENTRY_POINT: &'static str = "vs_main";
const FRAGMENT_SHADER_ENTRY_POINT: &'static str = "fs_main";

#[derive(Clone, Debug, Eq, PartialEq)]
struct AtlasSpriteCanvas {
    pub index: usize,
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
}

pub struct SpriteStack {
    sprite_canvas_stack: Vec<AtlasSpriteCanvas>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: wgpu::Buffer,
    full_surface_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: Option<wgpu::BindGroup>,
    surface_size_storage_buffer: wgpu::Buffer,
    pending_changes: Vec<(usize, AtlasSpriteCanvas)>,
    render_pipeline: wgpu::RenderPipeline,
}

impl SpriteStack {
    pub fn new(device: &wgpu::Device, render_texture_format: wgpu::TextureFormat, (width, height): (u32, u32)) -> Self {
        use wgpu::util::DeviceExt;

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader/sprite.wgsl"));

        let full_surface_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(prefixed_label!(full_surface_bind_group_layout)),
            entries: &[wgpu::BindGroupLayoutEntry { binding: 0,
                count: None,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: core::num::NonZeroU64::new(2 * core::mem::size_of::<u32>() as u64),
                },
            }],
        });

        let surface_size_array: [u32; 2] = [width, height];
        let surface_size_storage_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(prefixed_label!(surface_size_storage_buffer)),
                contents: bytemuck::cast_slice(surface_size_array.as_slice()),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }
        );

        let full_surface_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(prefixed_label!(full_surface_bind_group)),
            layout: &full_surface_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &surface_size_storage_buffer, offset: 0, size: None }),
                },
            ],
        });


        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the corresponding Texture entry.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some(prefixed_label!(texture_bind_group_layout)),
            });
        
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(prefixed_label!(render_pipeline_layout)),
            bind_group_layouts: &[&full_surface_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(prefixed_label!(render_pipeline)),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: VERTEX_SHADER_ENTRY_POINT,
                buffers: &[SpriteVertex::buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: FRAGMENT_SHADER_ENTRY_POINT,
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_texture_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(prefixed_label!(index_buffer)),
                contents: bytemuck::cast_slice(SPRITE_VERTICES_INDICES.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        SpriteStack {
            sprite_canvas_stack: Vec::new(),
            vertex_buffer: None,
            index_buffer,
            full_surface_bind_group,
            texture_bind_group_layout,
            texture_bind_group: None,
            surface_size_storage_buffer,
            pending_changes: Vec::new(),
            render_pipeline,
        }
    }

    /// Push a new sprite on top. Always invalidates the vertex buffer.
    pub fn push_sprite(&mut self, atlas_index: usize, x: u32, y: u32, width: u32, height: u32) -> usize {
        let result = self.sprite_canvas_stack.len();
        let sprite_canvas = AtlasSpriteCanvas {
            index: atlas_index,
            width,
            height,
            x,
            y,
        };
        self.sprite_canvas_stack.push(sprite_canvas.clone());
        self.invalidate_vertex_buffer();
        result
    }
    /// Pop the sprite on top. Never invalidates the vertex buffer.
    pub fn pop_sprite(&mut self) -> bool {
        self.sprite_canvas_stack.pop().is_some()
    }
    /// Remove a sprite from the sprite stack. Invalidates the vertex buffer except if the removed sprite was on top of the stack.
    pub fn remove_sprite(&mut self, index: usize) {
        self.sprite_canvas_stack.remove(index); 
        // invalidate if we popped the last item
        if index == self.sprite_canvas_stack.len() {
            self.invalidate_vertex_buffer();
        }
    }
    /// Insert a sprite in the sprite stack. Always invalidates the vertex buffer.
    pub fn insert_sprite(&mut self, sprite_index: usize, atlas_index: usize, x: u32, y: u32, width: u32, height: u32) {
        let sprite_canvas = AtlasSpriteCanvas {
            index: atlas_index,
            width,
            height,
            x,
            y,
        };
        self.sprite_canvas_stack.insert(sprite_index, sprite_canvas);
        self.invalidate_vertex_buffer();
    }
    /// Clear the sprite stack. Never invalidates the vertex buffer.
    pub fn clear_sprites(&mut self) {
        self.sprite_canvas_stack.clear();
    }
    /// Swap two sprites in the sprite stack. Never invalidates the vertex buffer.
    pub fn swap_sprites(&mut self, index1: usize, index2: usize) {
        self.pending_changes.push((index2, self.sprite_canvas_stack[index1].clone()));
        self.pending_changes.push((index1, self.sprite_canvas_stack[index2].clone()));
    }
    /// Modify a sprite in the sprite stack. Never invalidates the vertex buffer.
    pub fn update_sprite(&mut self, sprite_index: usize, atlas_index: usize, x: u32, y: u32, width: u32, height: u32) {
        let canvas = AtlasSpriteCanvas {
            index: atlas_index,
            width,
            height,
            x,
            y,
        };
        let pending_change = (sprite_index, canvas);
        self.pending_changes.push(pending_change);
    }

    fn is_compiled(&self) -> bool {
        self.vertex_buffer.is_some()
    }

    /// need to rebuild the vertex buffer
    fn invalidate_vertex_buffer(&mut self) {
        self.vertex_buffer = None;
    }

    fn sprite_vertice(texture_atlas: &SpriteTextureAtlas, sprite_canvas: &AtlasSpriteCanvas) -> SpriteVertice {
        // world coordinates are have the y-axis pointing up
        let position_corners = {
            let x = sprite_canvas.x;
            let y = sprite_canvas.y;
            let width = sprite_canvas.width;
            let height = sprite_canvas.height;
            (
                [x, y + height], // top left
                [x, y], // bottom left
                [x + width, y + height], // top right
                [x + width, y], // bottom right
            )
        };
        // textures coordinates are have the y-axis pointing down
        let texture_corners = {
            let sprite_canvas_dimensions = texture_atlas.sprite_canvas_dimensions(sprite_canvas.index).expect("Unable to find sprite in atlas");
            let atlas_dimensions = texture_atlas.atlas_dimensions();
            let (atlas_width, atlas_height) = (atlas_dimensions.0 as f32, atlas_dimensions.1 as f32);
            let (width, height) = (sprite_canvas_dimensions.pixel_size.0 as f32 / atlas_width, sprite_canvas_dimensions.pixel_size.1 as f32/ atlas_height);
            let vertical_row = sprite_canvas_dimensions.v_start_index as f32/ atlas_height;
            // texture dimensions are vertically inverted
            (
                [0., vertical_row], // top left
                [0., vertical_row + height], // bottom left
                [width, vertical_row], // top right
                [width, vertical_row + height], // bottom right
            )
        };
        [
            SpriteVertex { position: position_corners.0, sprite_texture_coords: texture_corners.0 }, // top left
            SpriteVertex { position: position_corners.1, sprite_texture_coords: texture_corners.1 }, // bottom left
            SpriteVertex { position: position_corners.2, sprite_texture_coords: texture_corners.2 }, // top right
            SpriteVertex { position: position_corners.3, sprite_texture_coords: texture_corners.3 }, // bottom right
        ]
    }

    fn sprite_vertice_buffer_data(sprite_vertice: &SpriteVertice) -> &[u8] {
        bytemuck::cast_slice(sprite_vertice.as_slice())
    }

    const fn sprite_vertex_buffer_size() -> u64 {
        core::mem::size_of::<SpriteVertice>() as u64
    }
    fn buffer_write_sprite(sprite_canvas: &AtlasSpriteCanvas, buffer_sprite_index: usize, vertex_buffer: &wgpu::Buffer, texture_atlas: &SpriteTextureAtlas, queue: &wgpu::Queue) {
        let sprite_vertice = Self::sprite_vertice(texture_atlas, sprite_canvas);
        let data = Self::sprite_vertice_buffer_data(&sprite_vertice);
        let offset = buffer_sprite_index as u64 * Self::sprite_vertex_buffer_size();
        queue.write_buffer(&vertex_buffer, offset, data);
    }
    fn queue_pending_changes(&mut self, texture_atlas: &SpriteTextureAtlas, queue: &wgpu::Queue) {
        for (canvas_index, pending_change) in self.pending_changes.drain(..) {
            // change in the drawn sprites array
            self.sprite_canvas_stack[canvas_index] = pending_change.clone();
            // change in the buffer slice
            if let Some(buffer) = self.vertex_buffer.as_ref() {
                Self::buffer_write_sprite(&pending_change, canvas_index, buffer, texture_atlas, queue);
            }
        }
    }
    pub fn total_vertex_buffer_slice_length(&self) -> u64 {
        self.sprite_canvas_stack.len() as u64 * Self::sprite_vertex_buffer_size()
    }
    pub fn compile_sprites(&mut self, texture_atlas: &SpriteTextureAtlas, device: &wgpu::Device, queue: &wgpu::Queue) {
        // sprite vertice
        let required_buffer_size = self.total_vertex_buffer_slice_length();
        let buffer = match self.vertex_buffer.take() {
            Some(buffer) if buffer.size() >= required_buffer_size => buffer,
            _ => device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(prefixed_label!(vertice_buffer)),
                mapped_at_creation: false,
                size: self.sprite_canvas_stack.len() as u64 * Self::sprite_vertex_buffer_size(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }),
        };
        for (draw_index, sprite_canvas) in self.sprite_canvas_stack.iter().enumerate() {
            Self::buffer_write_sprite(sprite_canvas, draw_index, &buffer, texture_atlas, queue);
        }
        self.vertex_buffer = Some(buffer);
    }
    pub fn resize(&self, (width, height): (u32, u32), queue: &wgpu::Queue) {
        let dimensions_array: [u32; 2] = [width, height];
        queue.write_buffer(&self.surface_size_storage_buffer, 0, bytemuck::cast_slice(dimensions_array.as_slice()));
    }
    pub fn set_texture_atlas(&mut self, texture_atlas: &SpriteTextureAtlas, device: &wgpu::Device) {
        let texture_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(texture_atlas.texture_view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(texture_atlas.sampler()),
                    }
                ],
                label: Some(prefixed_label!(texture_bind_group)),
            }
        );
        self.texture_bind_group = Some(texture_bind_group);
    }
    pub fn apply_changes(&mut self, texture_atlas: &SpriteTextureAtlas, device: &wgpu::Device, queue: &wgpu::Queue) {
        if !self.is_compiled() {
            self.compile_sprites(&texture_atlas, device, queue);
        }
        self.queue_pending_changes(texture_atlas, queue);
    }
    pub fn render(&self, surface_texture_view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(prefixed_label!(command_encoder)),
        });

        self.render_pass(surface_texture_view, &mut command_encoder);

        // submit will accept anything that implements IntoIter
        queue.submit(std::iter::once(command_encoder.finish()));
    }
    fn render_pass(&self, surface_texture_view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(prefixed_label!(render_pass)),
            color_attachments: &[
                // This is what @location(0) in the fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view: surface_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }
                        ),
                        store: false,
                    }
                })
            ],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        
        render_pass.set_bind_group(0, &self.full_surface_bind_group, &[]);
        if let Some(texture_bind_group) = self.texture_bind_group.as_ref() {
            render_pass.set_bind_group(1, texture_bind_group, &[]);
        }
        if let Some(vertex_buffer) = self.vertex_buffer.as_ref() {
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        }
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        for sprite_index in 0..self.sprite_canvas_stack.len() as u32 {
            let base_vertex = sprite_index * Self::sprite_vertice_count();
            render_pass.draw_indexed(0..Self::sprite_indices_number(), base_vertex as i32, 0..1);
        }
    }
    const fn sprite_vertice_count() -> u32 {
        4
    }
    const fn sprite_indices_number() -> u32 {
        SPRITE_VERTICES_INDICES.len() as u32
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteVertex {
    position: [u32; 2],
    sprite_texture_coords: [f32; 2],
}

impl SpriteVertex {
    fn buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Uint32x2, 1 => Float32x2];
        wgpu::VertexBufferLayout {
            array_stride: core::mem::size_of::<SpriteVertex>() as _,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

type SpriteVertice = [SpriteVertex; SpriteStack::sprite_vertice_count() as usize];

const SPRITE_VERTICES_INDICES: [u16; 6] = [
    0, 1, 2, // top left, bottom left, top right
    2, 1, 3, // top right, bottom left, bottom right
];
