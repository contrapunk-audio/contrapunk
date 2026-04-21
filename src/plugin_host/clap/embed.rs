//! Embedded plugin view — inserts the plugin's native view into
//! Contrapunk's own window instead of a separate OS window.
//!
//! The caller supplies:
//! * A pointer to Contrapunk's main `NSWindow` (obtained from Tauri via
//!   `WebviewWindow::ns_window()`).
//! * A rect (client coordinates from the rack's `getBoundingClientRect`)
//!   where the plugin should live.
//!
//! We create a bare `NSView` at that frame, add it as a subview of the
//! window's `contentView`, and expose its pointer so the CLAP plugin
//! can `set_parent` into it. The webview sits BELOW this subview in
//! z-order, so the plugin paints on top of the DOM at the given rect.

#[cfg(target_os = "macos")]
mod cocoa {
    use std::ffi::c_void;

    use objc2::rc::Retained;
    use objc2::{MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{NSView, NSWindow};
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    /// Container subview that claims top-left origin coordinates so
    /// we can position it directly in DOM-client-rect space without
    /// flipping the Y axis. `isFlipped` only affects child layout;
    /// the frame we set on THIS view is still interpreted in the
    /// parent's (contentView's) coordinate space.
    ///
    /// We still flip Y to place the frame correctly relative to the
    /// window — see `client_rect_to_ns_frame`.
    pub struct EmbeddedPluginView {
        /// Our subview where the plugin attaches. Kept alive by its
        /// superview's retain; we don't drop it manually on teardown
        /// to avoid an overrelease fight with AppKit's autorelease
        /// pool (caused a use-after-free crash when re-opening).
        subview: Retained<NSView>,
        /// Weak reference to the contentView so we can re-measure on
        /// every frame update (window may have resized since attach).
        content_view: Retained<NSView>,
    }

    /// Convert a DOM `getBoundingClientRect` (top-left origin, pixels
    /// from top of the webview) into an AppKit frame rect for
    /// positioning inside the window's (non-flipped) contentView.
    fn client_rect_to_ns_frame(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        content_view_height: f64,
    ) -> NSRect {
        // No clamp: if the rack is scrolled below the visible viewport
        // the subview gets a negative y and naturally goes off-screen
        // (which is what the user expects — the plugin should track
        // the rack).
        let ns_y = content_view_height - y - height;
        NSRect {
            origin: NSPoint { x, y: ns_y },
            size: NSSize { width, height },
        }
    }

    impl EmbeddedPluginView {
        pub unsafe fn new(ns_window_ptr: *mut c_void, frame: NSRect) -> Option<Self> {
            let mtm = MainThreadMarker::new()?;
            // SAFETY: caller guarantees pointer validity on main thread.
            let ns_window: &NSWindow = unsafe { &*(ns_window_ptr as *const NSWindow) };
            let content_view = ns_window.contentView()?;
            let content_view_height = content_view.frame().size.height;
            let is_flipped = content_view.isFlipped();
            eprintln!(
                "[clap/embed] contentView: size={}x{} isFlipped={is_flipped}",
                content_view.frame().size.width,
                content_view_height
            );

            // If contentView is flipped (origin top-left), DOM coords
            // map directly; otherwise flip Y.
            let appkit_frame = if is_flipped {
                NSRect {
                    origin: NSPoint {
                        x: frame.origin.x,
                        y: frame.origin.y,
                    },
                    size: frame.size,
                }
            } else {
                client_rect_to_ns_frame(
                    frame.origin.x,
                    frame.origin.y,
                    frame.size.width,
                    frame.size.height,
                    content_view_height,
                )
            };
            eprintln!(
                "[clap/embed] client rect: ({},{}) {}x{} -> appkit frame: ({},{}) {}x{}",
                frame.origin.x,
                frame.origin.y,
                frame.size.width,
                frame.size.height,
                appkit_frame.origin.x,
                appkit_frame.origin.y,
                appkit_frame.size.width,
                appkit_frame.size.height
            );

            let subview = {
                let alloc = NSView::alloc(mtm);
                unsafe { NSView::initWithFrame(alloc, appkit_frame) }
            };
            subview.setWantsLayer(true);

            // Use `positioned:NSWindowAbove relativeTo:nil` so we're
            // added on TOP of the z-order, above the WKWebView.
            unsafe {
                use objc2_app_kit::NSWindowOrderingMode;
                content_view.addSubview_positioned_relativeTo(
                    &subview,
                    NSWindowOrderingMode::Above,
                    None,
                );
            }
            Some(Self {
                subview,
                content_view,
            })
        }

        pub fn ns_view_ptr(&self) -> *mut c_void {
            Retained::as_ptr(&self.subview) as *mut c_void
        }

        pub fn set_frame(&self, x: f64, y: f64, width: f64, height: f64) {
            let h = self.content_view.frame().size.height;
            let appkit_frame = client_rect_to_ns_frame(x, y, width, height, h);
            self.subview.setFrame(appkit_frame);
        }
    }

    // Intentionally NO `Drop` impl. AppKit's `removeFromSuperview`
    // autoreleases the view — calling it from Drop while we still
    // hold a `Retained<NSView>` produces a double-release when the
    // autorelease pool drains (crash: `objc_release` in
    // `AutoreleasePoolPage::releaseUntil`). The subview is retained
    // by its superview; letting Retained drop simply decrements one
    // of the two retain counts, and the superview keeps the view
    // alive for the lifetime of the window. That leaks a bare NSView
    // per plugin until the window closes, which is acceptable.
}

#[cfg(target_os = "macos")]
pub use cocoa::EmbeddedPluginView;

#[cfg(not(target_os = "macos"))]
pub struct EmbeddedPluginView;

#[cfg(not(target_os = "macos"))]
impl EmbeddedPluginView {
    pub unsafe fn new(_ns_window_ptr: *mut std::ffi::c_void, _frame: ()) -> Option<Self> {
        None
    }
    pub fn ns_view_ptr(&self) -> *mut std::ffi::c_void {
        std::ptr::null_mut()
    }
    pub fn set_frame(&self, _x: f64, _y: f64, _w: f64, _h: f64) {}
}
