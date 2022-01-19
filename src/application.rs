use crate::error::InitializationError;
use bgfx_rs::static_lib::{DebugFlags, Init, PlatformData, RendererType, ResetFlags};
use glfw::{Action, Glfw, Key, Window, WindowEvent, WindowMode};
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
pub struct Application {
    glfw: Glfw,
    event_stream: EventStream,
    pub window: glfw::Window,
    pub size: (u32, u32),
    pub debug_flags: DebugFlags,
}

impl Application {
    fn new(
        glfw: Glfw,
        event_stream: EventStream,
        window: Window,
        size: (u32, u32),
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
                event_stream,
                window,
                (metadata.width, metadata.height),
                metadata.debug_flags,
            )),
            None => Err(InitializationError::Window),
        }
    }

    pub fn init(&mut self) -> Result<(), InitializationError> {
        let mut init = Init::new();
        init.type_r = self.get_render_type();
        init.resolution.height = self.size.0;
        init.resolution.width = self.size.1;
        init.resolution.reset = ResetFlags::VSYNC.bits(); // enable vsync
        init.platform_data = self.get_platform_data();

        if !bgfx_rs::static_lib::init(&init) {
            return Err(InitializationError::Bgfx);
        };

        Ok(())
    }

    /// Base event loop
    pub fn run(
        &mut self,
        // executor: Box<dyn FnOnce(&mut Application) -> crate::error::Result<()>>,
        executor: impl FnOnce(&mut Application) -> crate::error::Result<()>,
    ) -> crate::error::Result<()> {
        executor(self)
    }

    pub fn handle_events(&mut self) {
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

impl AsMut<Application> for Application {
    fn as_mut(&mut self) -> &mut Application {
        self
    }
}
