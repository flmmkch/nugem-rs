use crate::game::graphics::{surface::BitmapSurface, State};

macro_rules! prefixed_label {
    ($name:ident) => {
        concat!("sprites::texture_atlas::", stringify!($name))
    };
}

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

#[derive(Debug)]
pub struct SpriteTextureAtlas {
    atlas_dimensions: (u32, u32),
    sprites: Vec<SpriteCanvasInfo>,
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl SpriteTextureAtlas {
    pub fn dimensions(&self, index: usize) -> Option<(u32, u32)> {
        self.sprites.get(index).map(|s| s.pixel_size).clone()
    }
    pub fn sprite_canvas_dimensions(&self, sprite_index: usize) -> Option<&SpriteCanvasInfo> {
        self.sprites.get(sprite_index)
    }
    pub fn atlas_dimensions(&self) -> (u32, u32) {
        self.atlas_dimensions
    }
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}


#[derive(Clone, Debug)]
pub struct SpriteTextureAtlasBuilder {
    surfaces: Vec<BitmapSurface>,
}

impl SpriteTextureAtlasBuilder {
    pub fn new() -> SpriteTextureAtlasBuilder {
        SpriteTextureAtlasBuilder {
            surfaces: Vec::new(),
        }
    }
    pub fn add_surface(&mut self, surface: BitmapSurface) -> usize {
        let id = self.surfaces.len();
        self.surfaces.push(surface);
        id
    }
    fn build_sprites_canvas(&self) -> (Vec<u8>, Vec<SpriteCanvasInfo>, u32, u32) {
        // find the total width and the max height in the textures
        let (max_width, total_height) = {
            let mut max_width = 0u32;
            let mut total_height= 0u32;
            for ref surface in self.surfaces.iter() {
                total_height += surface.height();
                let width = surface.width();
                if width > max_width {
                    max_width = width;
                }
            }
            (max_width, total_height)
        };
        let mut sprites = Vec::new();
        let mut total_rgba = Vec::with_capacity(4 * max_width as usize * total_height as usize);
        let mut current_height = 0u32;
        for surface in self.surfaces.iter() {
            sprites.push(SpriteCanvasInfo {
                    v_start_index: current_height,
                    pixel_size: (surface.width(), surface.height())
                });
            {
                let padding_size = 4 * (max_width - surface.width()) as usize;
                let mut surface_pixel_iter = surface.pixels().iter().copied();
                for _ in 0..(surface.height() as usize) {
                    for _ in 0..(surface.width() as usize) {
                        let pixel = surface_pixel_iter.next().unwrap();
                        total_rgba.push(pixel.r());
                        total_rgba.push(pixel.g());
                        total_rgba.push(pixel.b());
                        total_rgba.push(pixel.a());
                    }
                    total_rgba.extend(std::iter::repeat(0).take(padding_size));
                }
            }
            current_height += surface.height();
        }
        (total_rgba, sprites, max_width, total_height)
    }
    fn build_texture(&self, texture_rgba: &[u8], width: u32, height: u32, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(wgpu::Texture, wgpu::TextureView, wgpu::Sampler), super::Error> {
        if texture_rgba.len() == 0 {
            Err(super::Error::EmptyAtlas)?;
        }

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let atlas_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                // All textures are stored as 3D, we represent our 2D texture
                // by setting depth to 1.
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TEXTURE_FORMAT,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some(prefixed_label!(atlas_texture)),
            }
        );

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            texture_rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                // rgba so 4 bytes per row
                bytes_per_row: core::num::NonZeroU32::new(texture_size.width * 4 * core::mem::size_of::<u8>() as u32),
                rows_per_image: core::num::NonZeroU32::new(texture_size.height * core::mem::size_of::<u8>() as u32),
            },
            texture_size,
        );

        let texture_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // pixelated filter modes
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            label: Some(prefixed_label!(sampler)),
            ..Default::default()
        });

        Ok((atlas_texture, texture_view, sampler))
    }
    pub fn build(self, state: &State) -> Result<SpriteTextureAtlas, super::Error> {
        // build the texture mega-surface: all textures aligned vertically
        let (total_rgba, sprites, max_width, total_height) = self.build_sprites_canvas();
        let (texture, texture_view, sampler) = self.build_texture(total_rgba.as_slice(), max_width, total_height, state.device(), state.queue())?;
        Ok(SpriteTextureAtlas { 
            atlas_dimensions: (max_width, total_height),
            sprites,
            texture,
            texture_view,
            sampler,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SpriteCanvasInfo {
    /// Vertical start index in the mega-surface, from the top; to know the total height, calculate with the next sprite display
    pub v_start_index: u32,
    pub pixel_size: (u32, u32),
}
