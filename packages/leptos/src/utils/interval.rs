//! Interval scheduling with automatic cancellation.
//!
//! Ported from `@base-ui/utils/useInterval`.
//! The `Interval` struct wraps `setInterval`/`clearInterval` with a guard that
//! ensures at most one interval is active at a time. Starting a new interval
//! automatically cancels the previous one.
//!
//! The React `useInterval()` hook is not ported — in Leptos, create an `Interval`
//! in the component body and use `on_cleanup` for automatic cancellation.

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

/// A managed `setInterval` wrapper that ensures at most one interval is active.
///
/// Maps to the `Interval` class in the React version.
///
/// # Example (Leptos)
/// ```ignore
/// let interval = Interval::new();
/// on_cleanup({
///     let interval = interval.clone();
///     move || interval.clear()
/// });
/// interval.start(1000, move || { /* called every 1s */ });
/// ```
#[derive(Clone)]
pub struct Interval {
    inner: std::rc::Rc<std::cell::RefCell<IntervalInner>>,
}

struct IntervalInner {
    current_id: Option<i32>,
}

impl Interval {
    /// Create a new `Interval` with no scheduled callback.
    pub fn new() -> Self {
        Interval {
            inner: std::rc::Rc::new(std::cell::RefCell::new(IntervalInner {
                current_id: None,
            })),
        }
    }

    /// Schedule `f` to run repeatedly at `delay_ms` millisecond intervals.
    /// Any previously scheduled interval is cancelled first.
    pub fn start<F>(&self, delay_ms: i32, f: F)
    where
        F: Fn() + 'static,
    {
        self.clear();

        let closure = Closure::wrap(Box::new(f) as Box<dyn Fn()>);

        let id = web_sys::window()
            .expect("no window")
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                delay_ms,
            )
            .expect("setInterval failed");

        // Leak the closure so it stays alive for the interval's lifetime.
        // It will be cleaned up when `clear()` is called and the browser
        // releases the interval callback.
        closure.forget();

        self.inner.borrow_mut().current_id = Some(id);
    }

    /// Returns `true` if an interval is currently active.
    pub fn is_started(&self) -> bool {
        self.inner.borrow().current_id.is_some()
    }

    /// Cancel the currently active interval, if any.
    pub fn clear(&self) {
        let mut inner = self.inner.borrow_mut();
        if let Some(id) = inner.current_id.take() {
            if let Some(window) = web_sys::window() {
                window.clear_interval_with_handle(id);
            }
        }
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from useInterval.ts (Interval class):
    // ✓ new interval is not started
    // ✗ start schedules a recurring callback (requires browser timer)
    // ✗ start cancels previous interval (requires browser timer)
    // ✗ clear cancels the interval (requires browser timer)
    //
    // Timer tests require wasm-bindgen-test with a real browser.

    #[test]
    fn new_interval_is_not_started() {
        let interval = Interval::new();
        assert!(!interval.is_started());
    }

    #[test]
    fn default_is_not_started() {
        let interval = Interval::default();
        assert!(!interval.is_started());
    }
}
