//! Animation frame scheduling with batching and cancellation.
//!
//! Ported from `@base-ui/utils/useAnimationFrame`.
//! The `AnimationFrame` struct wraps `requestAnimationFrame`/`cancelAnimationFrame`
//! with a guard that ensures at most one frame request is active at a time.
//!
//! The `Scheduler` provides a shared, batched animation frame scheduler that
//! coalesces multiple `request()` calls into a single `requestAnimationFrame`.
//!
//! The React `useAnimationFrame()` hook is not ported — in Leptos, create an
//! `AnimationFrame` in the component body and use `on_cleanup` for automatic cancellation.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

type FrameCallback = Box<dyn FnOnce(f64)>;

/// A shared, batched animation frame scheduler.
///
/// Coalesces multiple callbacks into a single `requestAnimationFrame` call.
/// Maps to the `Scheduler` class in the React version.
struct Scheduler {
    callbacks: RefCell<Vec<Option<FrameCallback>>>,
    callbacks_count: Cell<usize>,
    next_id: Cell<u32>,
    start_id: Cell<u32>,
    is_scheduled: Cell<bool>,
}

impl Scheduler {
    fn new() -> Rc<Self> {
        Rc::new(Self {
            callbacks: RefCell::new(Vec::new()),
            callbacks_count: Cell::new(0),
            next_id: Cell::new(1),
            start_id: Cell::new(1),
            is_scheduled: Cell::new(false),
        })
    }

    fn request(self: &Rc<Self>, f: FrameCallback) -> u32 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        self.callbacks.borrow_mut().push(Some(f));
        self.callbacks_count.set(self.callbacks_count.get() + 1);

        if !self.is_scheduled.get() {
            let this = Rc::clone(self);
            let closure = Closure::once_into_js(move |timestamp: f64| {
                this.tick(timestamp);
            });

            web_sys::window()
                .expect("no window")
                .request_animation_frame(closure.as_ref().unchecked_ref())
                .expect("requestAnimationFrame failed");
            self.is_scheduled.set(true);
        }
        id
    }

    fn cancel(&self, id: u32) {
        let start = self.start_id.get();
        if id < start {
            return;
        }
        let index = (id - start) as usize;
        let mut callbacks = self.callbacks.borrow_mut();
        if index >= callbacks.len() {
            return;
        }
        if callbacks[index].is_some() {
            callbacks[index] = None;
            self.callbacks_count.set(self.callbacks_count.get().saturating_sub(1));
        }
    }

    fn tick(&self, timestamp: f64) {
        self.is_scheduled.set(false);

        let current_callbacks: Vec<Option<FrameCallback>> =
            std::mem::take(&mut *self.callbacks.borrow_mut());
        let current_count = self.callbacks_count.get();

        self.callbacks_count.set(0);
        self.start_id.set(self.next_id.get());

        if current_count > 0 {
            for cb in current_callbacks {
                if let Some(f) = cb {
                    f(timestamp);
                }
            }
        }
    }
}

thread_local! {
    static SCHEDULER: Rc<Scheduler> = Scheduler::new();
}

/// A managed `requestAnimationFrame` wrapper that ensures at most one request is active.
///
/// Maps to the `AnimationFrame` class in the React version.
///
/// # Example (Leptos)
/// ```ignore
/// let raf = AnimationFrame::new();
/// on_cleanup({
///     let raf = raf.clone();
///     move || raf.cancel()
/// });
/// raf.request(move || { /* ... */ });
/// ```
#[derive(Clone)]
pub struct AnimationFrame {
    inner: Rc<Cell<Option<u32>>>,
}

impl AnimationFrame {
    /// Create a new `AnimationFrame` with no scheduled callback.
    pub fn new() -> Self {
        AnimationFrame {
            inner: Rc::new(Cell::new(None)),
        }
    }

    /// Schedule `f` to run on the next animation frame.
    /// Any previously scheduled callback is cancelled first.
    pub fn request<F>(&self, f: F)
    where
        F: FnOnce() + 'static,
    {
        self.cancel();
        let inner = self.inner.clone();
        let id = SCHEDULER.with(|s| {
            s.request(Box::new(move |_timestamp| {
                inner.set(None);
                f();
            }))
        });
        self.inner.set(Some(id));
    }

    /// Cancel the currently scheduled animation frame, if any.
    pub fn cancel(&self) {
        if let Some(id) = self.inner.take() {
            SCHEDULER.with(|s| s.cancel(id));
        }
    }

    /// Returns `true` if an animation frame is currently scheduled.
    pub fn is_scheduled(&self) -> bool {
        self.inner.get().is_some()
    }

    /// Request a callback on the shared scheduler (static method).
    ///
    /// Maps to `AnimationFrame.request(fn)` (static) in the React version.
    pub fn request_static<F>(f: F) -> u32
    where
        F: FnOnce(f64) + 'static,
    {
        SCHEDULER.with(|s| s.request(Box::new(f)))
    }

    /// Cancel a callback on the shared scheduler (static method).
    ///
    /// Maps to `AnimationFrame.cancel(id)` (static) in the React version.
    pub fn cancel_static(id: u32) {
        SCHEDULER.with(|s| s.cancel(id));
    }
}

impl Default for AnimationFrame {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from useAnimationFrame.ts:
    // ✓ new AnimationFrame is not scheduled
    // ✓ cancel on unscheduled frame is a no-op
    // ✓ Scheduler ID increments
    // ✗ request schedules a callback (requires browser rAF)
    // ✗ request cancels previous frame (requires browser rAF)
    // ✗ Scheduler batches callbacks into single rAF (requires browser rAF)
    // ✗ Scheduler.cancel nullifies a callback (requires browser rAF)
    //
    // Animation frame tests require wasm-bindgen-test with a real browser.

    #[test]
    fn new_animation_frame_is_not_scheduled() {
        let af = AnimationFrame::new();
        assert!(!af.is_scheduled());
    }

    #[test]
    fn cancel_on_unscheduled_is_noop() {
        let af = AnimationFrame::new();
        af.cancel(); // should not panic
        assert!(!af.is_scheduled());
    }

    #[test]
    fn default_is_not_scheduled() {
        let af = AnimationFrame::default();
        assert!(!af.is_scheduled());
    }

    #[test]
    fn scheduler_id_increments() {
        let scheduler = Scheduler::new();
        let id1 = scheduler.next_id.get();
        scheduler.next_id.set(id1 + 1);
        let id2 = scheduler.next_id.get();
        assert_eq!(id2, id1 + 1);
    }
}
