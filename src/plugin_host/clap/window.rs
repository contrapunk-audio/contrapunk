//! Native window container used to host a CLAP plugin's embedded GUI.
//!
//! Most pro plugins (FabFilter, Arturia, Diva) only support embedded
//! mode — they expect the host to provide a parent `NSView` on macOS,
//! `HWND` on Windows, or X11 `Window` on Linux. This module owns one
//! dedicated native window per plugin GUI; the plugin attaches its
//! native view into our window's content area.
//!
//! macOS only for now. Windows + Linux are follow-up work that
//! structures the same way: an OS-native window owned on the main
//! thread, its native handle passed to `gui.set_parent`.

#[cfg(target_os = "macos")]
mod cocoa {
    use std::ffi::c_void;

    use objc2::rc::Retained;
    use objc2::{MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{NSBackingStoreType, NSWindow, NSWindowStyleMask};
    use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

    /// Dedicated native window hosting one plugin's embedded GUI.
    pub struct PluginWindow {
        window: Retained<NSWindow>,
    }

    impl PluginWindow {
        /// Create and show a bare window suitable for embedding a
        /// plugin GUI. The content rect is a placeholder — the
        /// plugin will usually request its own size via the GUI
        /// extension right after `set_parent`.
        ///
        /// Must be called on the AppKit main thread (NSWindow has
        /// strict thread affinity).
        pub fn new(title: &str, width: f64, height: f64) -> Self {
            let mtm = MainThreadMarker::new()
                .expect("PluginWindow::new must run on the macOS main thread");
            let rect = NSRect {
                origin: NSPoint { x: 100.0, y: 100.0 },
                size: NSSize { width, height },
            };
            let style = NSWindowStyleMask::Titled
                | NSWindowStyleMask::Closable
                | NSWindowStyleMask::Resizable
                | NSWindowStyleMask::Miniaturizable;
            let window: Retained<NSWindow> = {
                let alloc = NSWindow::alloc(mtm);
                unsafe {
                    NSWindow::initWithContentRect_styleMask_backing_defer(
                        alloc,
                        rect,
                        style,
                        NSBackingStoreType::Buffered,
                        false,
                    )
                }
            };
            let ns_title = NSString::from_str(title);
            unsafe {
                window.setTitle(&ns_title);
                window.setReleasedWhenClosed(false);
            }
            unsafe { window.makeKeyAndOrderFront(None) };
            Self { window }
        }

        /// Pointer to the window's `NSView` content view. Handed to
        /// CLAP's `Window::from_cocoa_nsview` so the plugin can make
        /// its own view a subview.
        pub fn ns_view_ptr(&self) -> *mut c_void {
            let view =
                unsafe { self.window.contentView() }.expect("NSWindow should have a contentView");
            // Use the Retained as a stable pointer; the window owns
            // the view, so this address is valid while `self.window`
            // is alive.
            Retained::as_ptr(&view) as *mut c_void
        }

        /// Resize the window's content area to `width` × `height`.
        pub fn set_content_size(&self, width: f64, height: f64) {
            let size = NSSize { width, height };
            unsafe { self.window.setContentSize(size) };
        }

        /// Close and destroy the window. Called implicitly on drop.
        pub fn close(&self) {
            unsafe { self.window.close() };
        }
    }

    // Intentionally NO `Drop` impl. `NSWindow.close()` from a Drop
    // fired during registry removal triggers an Obj-C overrelease
    // when it races with Retained<NSWindow> dropping — same failure
    // pattern as EmbeddedPluginView. Letting Retained's release
    // naturally dealloc the window is safe because
    // `setReleasedWhenClosed(false)` is set.

    // NSWindow is not thread-safe. The controller owning a
    // PluginWindow is stored in a main-thread `thread_local!`, so
    // this isn't crossed between threads at runtime.
}

#[cfg(target_os = "macos")]
pub use cocoa::PluginWindow;

#[cfg(not(target_os = "macos"))]
mod stub {
    use std::ffi::c_void;

    /// Placeholder for non-macOS builds. Returns a dummy pointer;
    /// `open_gui` will reject embedded mode on these platforms until
    /// Win32 / X11 support is added.
    pub struct PluginWindow;

    impl PluginWindow {
        pub fn new(_title: &str, _w: f64, _h: f64) -> Self {
            Self
        }
        pub fn ns_view_ptr(&self) -> *mut c_void {
            std::ptr::null_mut()
        }
        pub fn set_content_size(&self, _w: f64, _h: f64) {}
        pub fn close(&self) {}
    }
}

#[cfg(not(target_os = "macos"))]
pub use stub::PluginWindow;
