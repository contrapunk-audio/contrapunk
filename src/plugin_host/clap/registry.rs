//! Main-thread registry for live plugin controllers.
//!
//! Plugin instances are `!Send`: they must never leave the thread
//! that created them. We store them in a `thread_local!` keyed by a
//! monotonically increasing `PluginId` so that callers dispatching
//! onto the main thread (via `AppHandle::run_on_main_thread`) can
//! look up and operate on individual plugins.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use super::controller::ClapPluginController;

/// Opaque id assigned at load time. Handed back to the UI so it can
/// reference the plugin without exposing internals.
pub type PluginId = u64;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Allocate a fresh plugin id. Safe to call from any thread.
pub fn next_id() -> PluginId {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

thread_local! {
    /// Live plugin controllers keyed by id. `RefCell` because we
    /// never have simultaneous borrows — every access is synchronous
    /// from the main thread.
    static PLUGINS: RefCell<HashMap<PluginId, ClapPluginController>> = RefCell::new(HashMap::new());
}

/// Run `f` with mutable access to the plugin map on the current
/// thread. Must be called on the main thread (where controllers
/// live); caller guarantees this via `AppHandle::run_on_main_thread`.
pub fn with_plugins<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<PluginId, ClapPluginController>) -> R,
{
    PLUGINS.with(|p| f(&mut p.borrow_mut()))
}

/// Insert a controller, returning its assigned id.
pub fn insert(controller: ClapPluginController) -> PluginId {
    let id = next_id();
    with_plugins(|m| {
        m.insert(id, controller);
    });
    id
}

/// Remove and drop the controller for `id`, if it exists. Dropping
/// the controller automatically closes any open GUI and drops the
/// plugin instance + entry.
pub fn remove(id: PluginId) {
    with_plugins(|m| {
        m.remove(&id);
    });
}
