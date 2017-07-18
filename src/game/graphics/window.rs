use ::game::Config;
use sdl2;
use gfx_window_sdl;
use gfx_core::Device;

pub mod gfx_types {
    use super::*;
    use gfx;
    use gfx_core;
    use gfx_device_gl;
    pub type CommandBuffer = gfx_device_gl::CommandBuffer;
    pub type Encoder = gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>;
    pub type Factory = gfx_device_gl::Factory;
    pub type Device = gfx_device_gl::Device;
    pub type RenderFormat = gfx_core::format::Rgba8;
    pub type DepthFormat = gfx_core::format::DepthStencil;
    pub type RenderTargetView = gfx_core::handle::RenderTargetView<gfx_device_gl::Resources, RenderFormat>;
    pub type DepthView = gfx_core::handle::DepthStencilView<gfx_device_gl::Resources, (gfx_core::format::D24_S8, gfx_core::format::Unorm)>;
}


pub struct Window {
    sdl_window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    device: gfx_types::Device,
    encoder: gfx_types::Encoder,
    factory: gfx_types::Factory,
    render_target_view: gfx_types::RenderTargetView,
    depth_view: gfx_types::DepthView,
}

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.6, 1.0];

impl Window {
    pub fn new(config: &Config, sdl_video: &sdl2::VideoSubsystem) -> Window {
        let (sdl_window, gl_context, mut device, mut factory, render_target_view, depth_view) = {
            let mut window_builder = sdl_video.window("Rugen", config.window_size().0, config.window_size().1);
            if config.fullscreen() {
                window_builder.fullscreen();
            }
            gfx_window_sdl::init::<gfx_types::RenderFormat, gfx_types::DepthFormat>(window_builder).expect("gfx_window_sdl::init failed!")
        };
        let encoder : gfx_types::Encoder = factory.create_command_buffer().into();
        Window {
            sdl_window,
            gl_context,
            device,
            encoder,
            factory,
            render_target_view,
            depth_view,
        }
    }
    pub fn update(&mut self) {
        // clear the frame
        self.encoder.clear(&self.render_target_view, CLEAR_COLOR);
        // do the drawing
        // flush
        self.encoder.flush(&mut self.device);
        self.sdl_window.gl_swap_window();
        self.device.cleanup();
    }
}