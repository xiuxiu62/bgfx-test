use application::{Application, WindowMetadata};
use bgfx_rs::static_lib::{ClearFlags, DbgTextClearArgs, DebugFlags, ResetArgs, SetViewClearArgs};
use error::Result;
use glfw::{Context, WindowMode};

mod application;
mod error;
mod tile;

const TITLE: &str = "Test window";
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

fn main() -> Result<()> {
    let metadata =
        WindowMetadata::new(TITLE, WIDTH, HEIGHT, WindowMode::Windowed, DebugFlags::TEXT);
    let mut application = Application::try_new(metadata)?;

    application.init()?;
    application.run(&executor)
}

fn executor(app: &mut Application) -> crate::error::Result<()> {
    app.window.make_current();
    app.window.set_key_polling(true);

    bgfx_rs::static_lib::set_debug(app.debug_flags.bits());
    bgfx_rs::static_lib::set_view_clear(
        0,
        ClearFlags::COLOR.bits() | ClearFlags::DEPTH.bits(),
        SetViewClearArgs {
            // rgba: 0x103030ff,
            rgba: 0x443355FF,
            ..Default::default()
        },
    );

    loop {
        match app.window.should_close() {
            true => break,
            false => tick(app),
        }
    }

    bgfx_rs::static_lib::shutdown();
    Ok(())
}

fn tick(app: &mut Application) {
    // Swap front and back buffers
    // self.window.swap_buffers();

    // Poll for and process events
    app.handle_events();

    let size = app.window.get_framebuffer_size();
    let size = (size.0 as u32, size.1 as u32);

    if app.size != size {
        bgfx_rs::static_lib::reset(size.0, size.1, ResetArgs::default());
        app.size = size;
    }

    bgfx_rs::static_lib::set_view_rect(0, 0, 0, size.0 as u16, size.1 as u16);
    bgfx_rs::static_lib::touch(0);

    bgfx_rs::static_lib::dbg_text_clear(DbgTextClearArgs::default());

    bgfx_rs::static_lib::dbg_text(0, 1, 0x0f, "Color can be changed with ANSI \x1b[9;me\x1b[10;ms\x1b[11;mc\x1b[12;ma\x1b[13;mp\x1b[14;me\x1b[0m code too.");
    bgfx_rs::static_lib::dbg_text(80, 1, 0x0f, "\x1b[;0m    \x1b[;1m    \x1b[; 2m    \x1b[; 3m    \x1b[; 4m    \x1b[; 5m    \x1b[; 6m    \x1b[; 7m    \x1b[0m");
    bgfx_rs::static_lib::dbg_text(80, 2, 0x0f, "\x1b[;8m    \x1b[;9m    \x1b[;10m    \x1b[;11m    \x1b[;12m    \x1b[;13m    \x1b[;14m    \x1b[;15m    \x1b[0m");
    bgfx_rs::static_lib::dbg_text(
        0,
        4,
        0x3f,
        "Description: Initialization and debug text with bgfx-rs Rust API.",
    );

    bgfx_rs::static_lib::frame(false);
}
