use bgfx_rs::static_lib::ResetArgs;
use error::Result;
use glfw::{Action, Context, Key, WindowMode};
use window::{WindowHandle, WindowMetadata};

mod error;
mod window;

const TITLE: &str = "Test window";
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

fn main() -> Result<()> {
    let metadata = WindowMetadata::new(TITLE, WIDTH, HEIGHT, WindowMode::Windowed);
    let mut window = WindowHandle::try_new(metadata)?;

    window.init()?;
    window.run();
    window.shutdown();

    Ok(())
}
