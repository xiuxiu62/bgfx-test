use crate::error::{Error, Result};
use bgfx_rs::static_lib::{Init, PlatformData, RendererType, ResetFlags};
use glfw::{Action, Context, Glfw, Key, Window, WindowEvent, WindowMode};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::sync::mpsc::Receiver;

pub type EventStream = Receiver<(f64, WindowEvent)>;

pub struct WindowMetadata<'a> {
    title: &'a str,
    width: u32,
    height: u32,
    mode: WindowMode<'a>,
}

impl<'a> WindowMetadata<'a> {
    pub fn new(title: &'a str, width: u32, height: u32, mode: WindowMode<'a>) -> Self {
        Self {
            title,
            width,
            height,
            mode,
        }
    }
}

/// Wrapper around a glfw window and EventStream for providing initialization abstractions
pub struct WindowHandle {
    glfw: Glfw,
    window: glfw::Window,
    event_stream: EventStream,
}

impl WindowHandle {
    fn new(glfw: Glfw, window: Window, event_stream: EventStream) -> Self {
        Self {
            glfw,
            window,
            event_stream,
        }
    }

    pub fn try_new(metadata: WindowMetadata<'_>) -> Result<Self> {
        let glfw = match glfw::init(glfw::FAIL_ON_ERRORS) {
            Ok(glfw) => glfw,
            Err(_) => return Err(Error::GlfwInitialization),
        };

        match glfw.create_window(
            metadata.width,
            metadata.height,
            metadata.title,
            metadata.mode,
        ) {
            Some((window, event_stream)) => Ok(Self::new(glfw, window, event_stream)),
            None => return Err(Error::WindowInitialization),
        }
    }

    pub fn init(&mut self) -> Result<()> {
        let mut init = Init::new();
        init.type_r = self.get_render_type();
        init.resolution.height = 0;
        init.resolution.width = 0;
        init.resolution.reset = ResetFlags::VSYNC.bits(); // makes window recreation smooth
        init.platform_data = self.get_platform_data();

        if !bgfx_rs::static_lib::init(&init) {
            return Err(Error::BgfxInitialization);
        };

        // Make the window's context current
        self.window.make_current();
        self.window.set_key_polling(true);

        Ok(())
    }

    /// Base event loop
    pub fn run(&mut self) {
        while !self.window.should_close() {
            // Swap front and back buffers
            self.window.swap_buffers();

            // Poll for and process events
            self.glfw.poll_events();
            for (_, event) in glfw::flush_messages(&self.event_stream) {
                println!("{:?}", event);
                match event {
                    glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                        self.window.set_should_close(true)
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn shutdown(&self) {
        bgfx_rs::static_lib::shutdown();
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
