use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to initialize {0}")]
    Initialization(#[from] InitializationError),
}

#[derive(Debug, Error)]
pub enum InitializationError {
    #[error("glfw")]
    Glfw,
    #[error("window")]
    Window,
    #[error("bgfx")]
    Bgfx,
}

pub type Result<T> = std::result::Result<T, Error>;
