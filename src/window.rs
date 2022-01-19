use crate::error::InitializationError;
use bgfx_rs::static_lib::{
    ClearFlags, DbgTextClearArgs, DebugFlags, Init, PlatformData, RendererType, ResetArgs,
    ResetFlags, SetViewClearArgs,
};
use glfw::{Action, Context, Glfw, Key, Window, WindowEvent, WindowMode};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::sync::mpsc::Receiver;

pub type EventStream = Receiver<(f64, WindowEvent)>;

pub struct WindowMetadata<'a> {
    title: &'a str,
    width: u32,
    height: u32,
    mode: WindowMode<'a>,
    debug_flags: DebugFlags,
}

impl<'a> WindowMetadata<'a> {
    pub fn new(
        title: &'a str,
        width: u32,
        height: u32,
        mode: WindowMode<'a>,
        debug_flags: DebugFlags,
    ) -> Self {
        Self {
            title,
            width,
            height,
            mode,
            debug_flags,
        }
    }
}

/// Wrapper around a glfw window and EventStream for providing initialization abstractions
pub struct WindowHandle {
    glfw: Glfw,
    window: glfw::Window,
    size: (i32, i32),
    event_stream: EventStream,
    debug_flags: DebugFlags,
}

impl WindowHandle {
    fn new(
        glfw: Glfw,
        window: Window,
        size: (i32, i32),
        event_stream: EventStream,
        debug_flags: DebugFlags,
    ) -> Self {
        Self {
            glfw,
            window,
            size,
            event_stream,
            debug_flags,
        }
    }

    pub fn try_new(metadata: WindowMetadata<'_>) -> Result<Self, InitializationError> {
        let glfw = match glfw::init(glfw::FAIL_ON_ERRORS) {
            Ok(glfw) => glfw,
            Err(_) => return Err(InitializationError::Glfw),
        };

        match glfw.create_window(
            metadata.width,
            metadata.height,
            metadata.title,
            metadata.mode,
        ) {
            Some((window, event_stream)) => Ok(Self::new(
                glfw,
                window,
                (metadata.width as i32, metadata.height as i32),
                event_stream,
                metadata.debug_flags,
            )),
            None => Err(InitializationError::Window),
        }
    }

    pub fn init(&mut self) -> Result<(), InitializationError> {
        let mut init = Init::new();
        init.type_r = self.get_render_type();
        init.resolution.height = 0;
        init.resolution.width = 0;
        init.resolution.reset = ResetFlags::VSYNC.bits(); // makes window recreation smooth
        init.platform_data = self.get_platform_data();

        if !bgfx_rs::static_lib::init(&init) {
            return Err(InitializationError::Bgfx);
        };

        bgfx_rs::static_lib::set_debug(self.debug_flags.bits());
        bgfx_rs::static_lib::set_view_clear(
            0,
            ClearFlags::COLOR.bits() | ClearFlags::DEPTH.bits(),
            SetViewClearArgs {
                rgba: 0x103030ff,
                ..Default::default()
            },
        );

        Ok(())
    }

    /// Base event loop
    pub fn run(&mut self) {
        self.window.make_current();
        self.window.set_key_polling(true);

        while !self.window.should_close() {
            // Swap front and back buffers
            // self.window.swap_buffers();

            // Poll for and process events
            self.handle_events();

            let size = self.window.get_framebuffer_size();

            if self.size != size {
                bgfx_rs::static_lib::reset(size.0 as _, size.1 as _, ResetArgs::default());
                self.size = size;
            }

            bgfx_rs::static_lib::set_view_rect(0, 0, 0, size.0 as _, size.1 as _);
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
    }

    pub fn shutdown(&self) {
        bgfx_rs::static_lib::shutdown();
    }

    fn handle_events(&mut self) {
        self.glfw.poll_events();
        glfw::flush_messages(&self.event_stream).for_each(|(_, event)| {
            println!("{:?}", event);
            if let glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
                self.window.set_should_close(true);
            }
        });
    }

    fn get_render_type(&self) -> RendererType {
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        return RendererType::Vulkan;
        #[cfg(target_os = "macos")]
        return RendererType::Metal;
    }

    fn get_platform_data(&self) -> PlatformData {
        let mut pd = PlatformData::new();

        match self.window.raw_window_handle() {
            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            RawWindowHandle::Xlib(data) => {
                use std::ffi::c_void;
                pd.nwh = data.window as *mut c_void;
                pd.ndt = data.display as *mut c_void;
            }
            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            RawWindowHandle::Wayland(data) => {
                pd.ndt = data.surface; // same as window, on wayland there ins't a concept of windows
                pd.nwh = data.display;
            }

            #[cfg(target_os = "macos")]
            RawWindowHandle::MacOS(data) => {
                pd.nwh = data.ns_window;
            }
            #[cfg(target_os = "windows")]
            RawWindowHandle::Win32(data) => {
                pd.nwh = data.hwnd;
            }
            _ => panic!("Unsupported Window Manager"),
        }

        pd
    }
}
