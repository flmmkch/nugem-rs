use gfx;

pub mod surface;

pub mod window;

pub mod sprite_displayer;

pub mod gfx_types {
    use gfx;
    use gfx_core;
    use gfx_device_gl;
    pub type Resources = gfx_device_gl::Resources;
    // pub type CommandBuffer = gfx_device_gl::CommandBuffer;
    pub type Encoder = gfx::Encoder<Resources, gfx_device_gl::CommandBuffer>;
    pub type Factory = gfx_device_gl::Factory;
    pub type Device = gfx_device_gl::Device;
    pub type RenderFormat = gfx_core::format::Rgba8;
    pub type DepthFormat = gfx_core::format::DepthStencil;
    pub type RenderTargetView = gfx_core::handle::RenderTargetView<Resources, RenderFormat>;
    pub type DepthView = gfx_core::handle::DepthStencilView<Resources, (gfx_core::format::D24_S8, gfx_core::format::Unorm)>;
    pub type ShaderResourceView = gfx::handle::ShaderResourceView<Resources, [f32; 4]>;
}

#[derive(Debug)]
pub enum Error {
    EmptyAtlas,
    GfxError(gfx::CombinedError),
}

impl From<gfx::CombinedError> for Error {
    fn from(e: gfx::CombinedError) -> Error {
        Error::GfxError(e)
    }
}
