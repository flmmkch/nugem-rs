use thiserror::Error;

pub mod surface;

pub mod sprites;

mod state;
pub use self::state::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Empty texture atlas")]
    EmptyAtlas,
    #[error(transparent)]
    WgpuError(#[from] wgpu::Error),
    #[error(transparent)]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
    #[error("No GPU adapter found")]
    NoGpuAdapter,
    #[error("No graphical surface format found")]
    NoGraphicalSurfaceFormat,
}
