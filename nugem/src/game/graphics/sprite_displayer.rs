use super::surface::BitmapSurface;
use super::Error;
use super::gfx_types;
use gfx;
use gfx::*;
use gfx_core;
use gfx::traits::{Factory, FactoryExt};
use log::error;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    pipeline sprite_pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        out: gfx::BlendTarget<gfx_types::RenderFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}

pub struct DrawingContext {
    pso: gfx::PipelineState<gfx_types::Resources, sprite_pipe::Meta>,
}

#[derive(Clone, Debug)]
pub struct SpriteTextureAtlas {
    sprites: Vec<SpriteCanvasInfo>,
    resource_view: gfx_types::ShaderResourceView,
}

#[derive(Clone, Debug)]
struct SpriteCanvasInfo {
    pub v_start_index: f32, // vertical start index in the mega-surface; to know the total height, calculate with the next sprite display
    pub width: f32,
    pub pixel_size: (u32, u32),
}

#[derive(Clone, Debug)]
pub struct TextureAtlasBuilder {
    surfaces: Vec<BitmapSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SpriteDrawingCanvas {
    pub index: usize,
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
}

pub struct SpritesDrawer {
    draw_sprites: Vec<SpriteDrawingCanvas>,
    buffer: Option<gfx::handle::Buffer<gfx_types::Resources, Vertex>>,
    pending_changes: Vec<(usize, SpriteDrawingCanvas)>,
}

impl TextureAtlasBuilder {
    pub fn new() -> TextureAtlasBuilder {
        TextureAtlasBuilder {
            surfaces: Vec::new(),
        }
    }
    pub fn add_surface(&mut self, surface: BitmapSurface) -> usize {
        let id = self.surfaces.len();
        self.surfaces.push(surface);
        id
    }
    pub fn build(self, factory: &mut gfx_types::Factory) -> Result<SpriteTextureAtlas, Error> {
        // build the texture mega-surface: all textures aligned vertically
        let (total_rgba, sprite_canvases, max_width, total_height) = {
            // find the total width and the max height in the textures
            let (max_width, total_height) = {
                let mut max_width : usize = 0;
                let mut total_height : usize = 0;
                for ref surface in self.surfaces.iter() {
                    total_height += surface.height() as usize;
                    let width = surface.width() as usize;
                    if width > max_width {
                        max_width = width;
                    }
                }
                (max_width, total_height)
            };
            let mut sprite_canvases = Vec::new();
            let mut total_rgba = Vec::with_capacity(4 * max_width * total_height);
            let mut current_height = 0.0;
            for surface in self.surfaces {
                let width = (surface.width() as f32) / (max_width as f32);
                let height = (surface.height() as f32) / (total_height as f32);
                sprite_canvases.push(
                    SpriteCanvasInfo::new(
                        current_height,
                        width,
                        (surface.width(), surface.height())
                        )
                    );
                {
                    let padding : Vec<u8> = {
                        let padding_size = 4 * (max_width - (surface.width() as usize)); 
                        vec![0; padding_size]
                    };
                    let mut surface_index = 0;
                    for _ in 0..(surface.height() as usize) {
                        for _ in 0..(surface.width() as usize) {
                            let pixel = &surface.pixels()[surface_index];
                            total_rgba.push(pixel.r());
                            total_rgba.push(pixel.g());
                            total_rgba.push(pixel.b());
                            total_rgba.push(pixel.a());
                            surface_index += 1;
                        }
                        total_rgba.extend(padding.clone());
                    }
                }
                current_height += height;
            }
            (total_rgba, sprite_canvases, max_width, total_height)
        };
        if total_rgba.len() > 0 {
            let kind = gfx::texture::Kind::D2(max_width as u16, total_height as u16, gfx::texture::AaMode::Single);
            let (_, resource_view) = factory.create_texture_immutable_u8::<gfx_types::RenderFormat>(kind, gfx::texture::Mipmap::Provided, &[&total_rgba])?;
            Ok(SpriteTextureAtlas::new(sprite_canvases, resource_view))
        }
        else {
            Err(Error::EmptyAtlas)
        }
    }
}

impl SpriteTextureAtlas {
    fn new(sprites: Vec<SpriteCanvasInfo>, resource_view: gfx_types::ShaderResourceView) -> SpriteTextureAtlas {
        SpriteTextureAtlas {
            sprites,
            resource_view,
        }
    }
    pub fn dimensions(&self, index: usize) -> (u32, u32) {
        self.sprites[index].pixel_size.clone()
    }
    pub fn resource_view(&self) -> &gfx_types::ShaderResourceView {
        &self.resource_view
    }
    pub fn v_bounds(&self, sprite_index: usize) -> (f32, f32) {
        if sprite_index == self.sprites.len() - 1 {
            (self.sprites[sprite_index].v_start_index, 1.0)
        }
        else {
            (self.sprites[sprite_index].v_start_index, self.sprites[sprite_index + 1].v_start_index)
        }
    }
    pub fn h_bounds(&self, sprite_index: usize) -> (f32, f32) {
        (0.0, self.sprites[sprite_index].width)
    }
}

impl SpriteCanvasInfo {
    pub fn new(v_start_index: f32, width: f32, pixel_size: (u32, u32)) -> SpriteCanvasInfo {
        SpriteCanvasInfo {
            v_start_index,
            width,
            pixel_size,
        }
    }
}

impl SpritesDrawer {
    pub fn new() -> SpritesDrawer {
        SpritesDrawer {
            draw_sprites: Vec::new(),
            buffer: None,
            pending_changes: Vec::new(),
        }
    }
    pub fn add_sprite(&mut self, index: usize, x: u32, y: u32, width: u32, height: u32) -> usize {
        let result = self.draw_sprites.len();
        self.draw_sprites.push(SpriteDrawingCanvas {
            index,
            width,
            height,
            x,
            y,
        });
        self.buffer = None;
        result
    }
    fn is_compiled(&self) -> bool {
        self.buffer.is_some()
    }
    pub fn update_canvas(&mut self, canvas_index: usize, atlas_index: usize, x: u32, y: u32, width: u32, height: u32) {
        let canvas = SpriteDrawingCanvas {
            index: atlas_index,
            width,
            height,
            x,
            y,
        };
        let pending_change = (canvas_index, canvas);
        self.pending_changes.push(pending_change);
    }
    fn apply_pending_changes(&mut self, texture_atlas: &SpriteTextureAtlas, encoder: &mut gfx_types::Encoder, render_target_view: gfx_types::RenderTargetView) {
        let dimensions_float = (render_target_view.get_dimensions().0 as f32, render_target_view.get_dimensions().1 as f32);
        for (canvas_index, pending_change) in self.pending_changes.drain(..) {
            // change in the drawn sprites array
            self.draw_sprites[canvas_index] = pending_change.clone();
            // change in the buffer slice
            if let Some(buffer) = self.buffer.as_mut() {
                // TODO factorize that code block with the compilation
                let texture_corners = {
                    let (v_top, v_bottom) = texture_atlas.v_bounds(pending_change.index);
                    let (h_left, h_right) = texture_atlas.h_bounds(pending_change.index);
                    (
                        [h_left, v_top], // top left
                        [h_right, v_top], // top right
                        [h_right, v_bottom], // bottom right
                        [h_left, v_bottom], // bottom left
                    )
                };
                let canvas_corners = {
                    let x = (2.0 * (pending_change.x as f32) + dimensions_float.0) / dimensions_float.0 - 2.0;
                    let y = (2.0 * (pending_change.y as f32) + dimensions_float.1) / dimensions_float.1 - 2.0;
                    let width = 2.0 * (pending_change.width as f32) / dimensions_float.0;
                    let height = 2.0 * (pending_change.height as f32) / dimensions_float.1;
                    (
                        [x, y + height], // top left
                        [x + width, y + height], // top right
                        [x + width, y], // bottom right
                        [x, y], // bottom left
                    )
                };
                let mut shape_vertex = Vec::new();
                shape_vertex.push(Vertex { pos: canvas_corners.0, uv: texture_corners.0 }); // top left
                shape_vertex.push(Vertex { pos: canvas_corners.1, uv: texture_corners.1 }); // top right
                shape_vertex.push(Vertex { pos: canvas_corners.2, uv: texture_corners.2 }); // bottom right
                shape_vertex.push(Vertex { pos: canvas_corners.0, uv: texture_corners.0 }); // top left
                shape_vertex.push(Vertex { pos: canvas_corners.3, uv: texture_corners.3 }); // bottom left
                shape_vertex.push(Vertex { pos: canvas_corners.2, uv: texture_corners.2 }); // bottom right
                let slice_index = 6 * canvas_index;
                if let Err(e) = encoder.update_buffer(&buffer, &shape_vertex[..], slice_index) {
                    error!("Error updating buffer: {}", e);
                }
            }
        }
    }
    pub fn compile(&mut self, texture_atlas: &SpriteTextureAtlas, encoder: &mut gfx_types::Encoder, factory: &mut gfx_types::Factory, render_target_view: gfx_types::RenderTargetView) {
        let mut shape_vertex = Vec::new();
        let dimensions_float = (render_target_view.get_dimensions().0 as f32, render_target_view.get_dimensions().1 as f32);
        for draw_sprite in self.draw_sprites.iter() {
            let texture_corners = {
                let (v_top, v_bottom) = texture_atlas.v_bounds(draw_sprite.index);
                let (h_left, h_right) = texture_atlas.h_bounds(draw_sprite.index);
                (
                    [h_left, v_top], // top left
                    [h_right, v_top], // top right
                    [h_right, v_bottom], // bottom right
                    [h_left, v_bottom], // bottom left
                )
            };
            let canvas_corners = {
                let x = (2.0 * (draw_sprite.x as f32) + dimensions_float.0) / dimensions_float.0 - 2.0;
                let y = (2.0 * (draw_sprite.y as f32) + dimensions_float.1) / dimensions_float.1 - 2.0;
                let width = 2.0 * (draw_sprite.width as f32) / dimensions_float.0;
                let height = 2.0 * (draw_sprite.height as f32) / dimensions_float.1;
                (
                    [x, y + height], // top left
                    [x + width, y + height], // top right
                    [x + width, y], // bottom right
                    [x, y], // bottom left
                )
            };
            shape_vertex.push(Vertex { pos: canvas_corners.0, uv: texture_corners.0 }); // top left
            shape_vertex.push(Vertex { pos: canvas_corners.1, uv: texture_corners.1 }); // top right
            shape_vertex.push(Vertex { pos: canvas_corners.2, uv: texture_corners.2 }); // bottom right
            shape_vertex.push(Vertex { pos: canvas_corners.0, uv: texture_corners.0 }); // top left
            shape_vertex.push(Vertex { pos: canvas_corners.3, uv: texture_corners.3 }); // bottom left
            shape_vertex.push(Vertex { pos: canvas_corners.2, uv: texture_corners.2 }); // bottom right
        }
        self.buffer = {
            let buffer = factory.create_buffer::<Vertex>(shape_vertex.len(), gfx::buffer::Role::Vertex, gfx_core::memory::Usage::Dynamic, gfx::memory::Bind::empty()).unwrap();
            if let Err(e) = encoder.update_buffer(&buffer, &shape_vertex[..], 0) {
                error!("Error initializing buffer: {}", e);
            }
            Some(buffer)
        };
    }
    pub fn draw(&mut self, context: &DrawingContext, texture_atlas: &SpriteTextureAtlas, factory: &mut gfx_types::Factory, encoder: &mut gfx_types::Encoder, render_target_view: gfx_types::RenderTargetView) {
        self.apply_pending_changes(texture_atlas, encoder, render_target_view.clone());
        if !self.is_compiled() {
            self.compile(&texture_atlas, encoder, factory, render_target_view.clone());
        }
        let sampler = {
            use gfx::texture;
            let sampler_info = texture::SamplerInfo::new(texture::FilterMethod::Scale, texture::WrapMode::Tile);
            factory.create_sampler(sampler_info)
        };
        let texture = texture_atlas.resource_view().clone();
        if let Some(buffer) = self.buffer.as_ref() {
            let data = sprite_pipe::Data {
                vbuf: buffer.clone(),
                tex: (texture, sampler),
                out: render_target_view,
            };
            encoder.draw(&gfx::Slice::new_match_vertex_buffer(&buffer), &context.pso(), &data);
        }
    }
}

impl DrawingContext {
    pub fn new(factory: &mut gfx_types::Factory) -> DrawingContext {
        let pso = factory.create_pipeline_simple(
            include_bytes!("shader/sprite.glslv"),
            include_bytes!("shader/sprite.glslf"),
            sprite_pipe::new()
            ).unwrap();
        DrawingContext {
            pso,
        }
    }
    pub fn pso(&self) -> &gfx::pso::PipelineState<gfx_types::Resources, sprite_pipe::Meta> {
        &self.pso
    }
}
