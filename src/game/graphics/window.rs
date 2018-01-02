use ::game::Config;
use sdl2;
use gfx_window_sdl;
use gfx_core::Device;
use super::gfx_types;

pub struct Window {
    sdl_window: sdl2::video::Window,
    #[allow(dead_code)] // needed by RAII
    gl_context: sdl2::video::GLContext,
    device: gfx_types::Device,
    encoder: gfx_types::Encoder,
    factory: gfx_types::Factory,
    render_target_view: gfx_types::RenderTargetView,
    #[allow(dead_code)] // needed by RAII
    depth_view: gfx_types::DepthView,
}

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

impl Window {
    pub fn new(config: &Config, sdl_video: &sdl2::VideoSubsystem) -> Window {
        let (sdl_window, gl_context, device, mut factory, render_target_view, depth_view) = {
            let mut window_builder = sdl_video.window("Rugen", config.window_size().0, config.window_size().1);
            if config.fullscreen() {
                window_builder.fullscreen();
            }
            gfx_window_sdl::init::<gfx_types::RenderFormat, gfx_types::DepthFormat>(sdl_video, window_builder).expect("gfx_window_sdl::init failed!")
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
    pub fn clear(&mut self) {
        self.encoder.clear(&self.render_target_view, CLEAR_COLOR);
    }
    pub fn update(&mut self) {
        self.encoder.flush(&mut self.device);
        self.sdl_window.gl_swap_window();
        self.device.cleanup();
    }
    pub fn factory(&mut self) -> &mut gfx_types::Factory {
        &mut self.factory
    }
    pub fn gfx_data(&mut self) -> (&mut gfx_types::Factory, &mut gfx_types::Encoder, gfx_types::RenderTargetView) {
        (&mut self.factory, &mut self.encoder, self.render_target_view.clone())
    }
}