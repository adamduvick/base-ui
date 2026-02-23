//! Timeout scheduling with automatic cancellation.
//!
//! Ported from `@base-ui/utils/useTimeout`.
//! The `Timeout` struct wraps `setTimeout`/`clearTimeout` with a guard that
//! ensures at most one timeout is active at a time. Starting a new timeout
//! automatically cancels the previous one.
//!
//! The React `useTimeout()` hook is not ported — in Leptos, create a `Timeout`
//! in the component body and use `on_cleanup` for automatic cancellation.

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

/// A managed `setTimeout` wrapper that ensures at most one timeout is active.
///
/// Maps to the `Timeout` class in the React version.
///
/// # Example (Leptos)
/// ```ignore
/// let timeout = Timeout::new();
/// on_cleanup({
///     let timeout = timeout.clone();
///     move || timeout.clear()
/// });
/// timeout.start(500, move || { /* ... */ });
/// ```
#[derive(Clone)]
pub struct Timeout {
    inner: std::rc::Rc<std::cell::RefCell<TimeoutInner>>,
}

struct TimeoutInner {
    current_id: Option<i32>,
}

impl Timeout {
    /// Create a new `Timeout` with no scheduled callback.
    pub fn new() -> Self {
        Timeout {
            inner: std::rc::Rc::new(std::cell::RefCell::new(TimeoutInner {
                current_id: None,
            })),
        }
    }

    /// Schedule `f` to run after `delay_ms` milliseconds.
    /// Any previously scheduled callback is cancelled first.
    pub fn start<F>(&self, delay_ms: i32, f: F)
    where
        F: FnOnce() + 'static,
    {
        self.clear();

        let inner = self.inner.clone();
        let closure = Closure::once_into_js(move || {
            inner.borrow_mut().current_id = None;
            f();
        });

        let id = web_sys::window()
            .expect("no window")
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                delay_ms,
            )
            .expect("setTimeout failed");

        self.inner.borrow_mut().current_id = Some(id);
    }

    /// Returns `true` if a timeout is currently scheduled.
    pub fn is_started(&self) -> bool {
        self.inner.borrow().current_id.is_some()
    }

    /// Cancel the currently scheduled timeout, if any.
    pub fn clear(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(id) = inner.current_id.take() {
            if let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(id);
            }
        }
    }
}

impl Default for Timeout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from useTimeout.ts (Timeout class):
    // ✓ new timeout is not started
    // ✗ start schedules a callback (requires browser timer)
    // ✗ start cancels previous timeout (requires browser timer)
    // ✗ clear cancels the timeout (requires browser timer)
    // ✗ disposeEffect returns clear (React-specific)
    //
    // Timer tests require wasm-bindgen-test with a real browser.

    #[test]
    fn new_timeout_is_not_started() {
        let timeout = Timeout::new();
        assert!(!timeout.is_started());
    }

    #[test]
    fn default_is_not_started() {
        let timeout = Timeout::default();
        assert!(!timeout.is_started());
    }
}
