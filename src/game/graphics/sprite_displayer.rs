use super::surface::BitmapSurface;
use super::Error;
use super::gfx_types;
use gfx;
use gfx_core::Factory;
use gfx::traits::FactoryExt;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    pipeline sprite_pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        out: gfx::BlendTarget<gfx_types::RenderFormat> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
    }
}

pub struct DrawingContext {
    pso: gfx::PipelineState<gfx_types::Resources, sprite_pipe::Meta>,
}

pub struct SpriteTextureAtlas {
    sprites: Vec<SpriteCanvasInfo>,
    resource_view: gfx_types::ShaderResourceView,
}

struct SpriteCanvasInfo {
    pub v_start_index: f32, // vertical start index in the mega-surface; to know the total height, calculate with the next sprite display
    pub width: f32,
}

pub struct TextureAtlasBuilder {
    surfaces: Vec<BitmapSurface>,
}

struct SpriteDrawingCanvas {
    pub index: usize,
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
}

pub struct SpritesDrawer<'a> {
    texture_atlas: &'a SpriteTextureAtlas,
    draw_sprites: Vec<SpriteDrawingCanvas>,
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
            let mut total_rgba = Vec::new();
            let mut current_height = 0.0;
            for surface in self.surfaces {
                let height = (surface.height() as f32) / (total_height as f32);
                let width = (surface.width() as f32) / (max_width as f32);
                sprite_canvases.push(SpriteCanvasInfo::new(current_height, width));
                total_rgba.extend(surface.to_rgba());
                current_height += height;
            }
            (total_rgba, sprite_canvases, max_width, total_height)
        };
        if total_rgba.len() > 0 {
            let kind = gfx::texture::Kind::D2(max_width as u16, total_height as u16, gfx::texture::AaMode::Single);
            let (_, resource_view) = factory.create_texture_immutable_u8::<gfx_types::RenderFormat>(kind, &[&total_rgba])?;
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
    pub fn new(v_start_index: f32, width: f32) -> SpriteCanvasInfo {
        SpriteCanvasInfo {
            v_start_index,
            width
        }
    }
}

impl<'a> SpritesDrawer<'a> {
    pub fn new(texture_atlas: &'a SpriteTextureAtlas) -> SpritesDrawer<'a> {
        SpritesDrawer {
            texture_atlas,
            draw_sprites: Vec::new(),
        }
    }
    pub fn add_sprite(&mut self, index: usize, x: u32, y: u32, width: u32, height: u32) {
        self.draw_sprites.push(SpriteDrawingCanvas {
            index,
            width,
            height,
            x,
            y,
        });
    }
    pub fn draw(&self, context: &DrawingContext, factory: &mut gfx_types::Factory, encoder: &mut gfx_types::Encoder, render_target_view: gfx_types::RenderTargetView) {
        let mut shape_vertex = Vec::new();
        // let dimensions_float = (render_target_view.get_dimensions().0 as f32, render_target_view.get_dimensions().1 as f32);
        for draw_sprite in self.draw_sprites.iter() {
            let texture_corners = {
                let (v_top, v_bottom) = self.texture_atlas.v_bounds(draw_sprite.index);
                let (h_left, h_right) = self.texture_atlas.h_bounds(draw_sprite.index);
                (
                    [h_left, v_top], // top left
                    [h_right, v_top], // top right
                    [h_right, v_bottom], // bottom right
                    [h_left, v_bottom], // bottom left
                )
            };
            let canvas_corners = {
                // let x = 2.0 * ((draw_sprite.x as f32) - dimensions_float.0) / dimensions_float.0;
                // let y = 2.0 * ((draw_sprite.y as f32) - dimensions_float.1) / dimensions_float.1;
                // let width = (draw_sprite.width as f32) / dimensions_float.0;
                // let height = (draw_sprite.height as f32) / dimensions_float.1;
                // (
                //     [x, y + height], // top left
                //     [x + width, y + height], // top right
                //     [x + width, y], // bottom right
                //     [x, y], // bottom left
                // )
                // (
                //     [1.0, 1.0], // top left
                //     [-1.0, 1.0], // top right
                //     [-1.0, -1.0], // bottom right
                //     [1.0, -1.0], // bottom left
                // )
                (
                    [1.0, 1.0], // top left
                    [-1.0, 1.0], // top right
                    [-1.0, -1.0], // bottom right
                    [1.0, -1.0], // bottom left
                )
            };
            shape_vertex.push(Vertex { pos: canvas_corners.0, uv: texture_corners.0 }); // top left
            shape_vertex.push(Vertex { pos: canvas_corners.1, uv: texture_corners.1 }); // top right
            shape_vertex.push(Vertex { pos: canvas_corners.2, uv: texture_corners.2 }); // bottom right
            shape_vertex.push(Vertex { pos: canvas_corners.3, uv: texture_corners.3 }); // bottom left
        }
        let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&shape_vertex[..], ());
        let sampler = factory.create_sampler_linear();
        let texture = self.texture_atlas.resource_view().clone();
        let data = sprite_pipe::Data {
            vbuf: vertex_buffer,
            tex: (texture, sampler),
            out: render_target_view,
        };
        encoder.draw(&slice, &context.pso(), &data);
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
