use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to initialize glfw")]
    GlfwInitialization,
    #[error("Failed to initialize window")]
    WindowInitialization,
    #[error("Failed to initialize bgfx")]
    BgfxInitialization,
}

pub type Result<T> = std::result::Result<T, Error>;
