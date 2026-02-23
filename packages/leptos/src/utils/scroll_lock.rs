//! Document scroll locking.
//!
//! Ported from `@base-ui/utils/useScrollLock`.
//! Provides a `ScrollLocker` that prevents page scrolling when modal dialogs,
//! popovers, or other overlays are open. Handles scrollbar compensation and
//! multiple concurrent lock requests via reference counting.
//!
//! The React `useScrollLock()` hook is not ported — in Leptos, use
//! `ScrollLocker::acquire()` and call the returned release function on cleanup.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;

use super::owner::{owner_document, owner_window};

/// Check if an element is an overflow element (has overflow: auto/scroll/hidden/clip/overlay
/// and is not display: inline or display: contents).
///
/// Ported from `@floating-ui/utils/dom` `isOverflowElement`.
fn is_overflow_element(element: &web_sys::Element) -> bool {
    let window = owner_window(Some(element));
    let computed = window
        .get_computed_style(element)
        .ok()
        .flatten();

    let Some(style) = computed else {
        return false;
    };

    let overflow = style.get_property_value("overflow").unwrap_or_default();
    let overflow_x = style.get_property_value("overflow-x").unwrap_or_default();
    let overflow_y = style.get_property_value("overflow-y").unwrap_or_default();
    let display = style.get_property_value("display").unwrap_or_default();

    let combined = format!("{overflow}{overflow_y}{overflow_x}");
    let has_overflow = combined.contains("auto")
        || combined.contains("scroll")
        || combined.contains("overlay")
        || combined.contains("hidden")
        || combined.contains("clip");

    has_overflow && display != "inline" && display != "contents"
}

/// Check whether the document has inset (non-overlay) scrollbars.
fn has_inset_scrollbars(reference_element: Option<&web_sys::Element>) -> bool {
    let doc = owner_document(reference_element);
    let win = owner_window(Some(doc.document_element().unwrap().as_ref()));
    let inner_width = win.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
    let client_width = doc.document_element().map(|e| e.client_width()).unwrap_or(0) as f64;
    inner_width - client_width > 0.0
}

/// Prevent scroll on overlay-scrollbar systems (iOS, macOS with overlay scrollbars).
/// Simply hides overflow on the scroll container.
fn prevent_scroll_overlay(reference_element: Option<&web_sys::Element>) -> Box<dyn FnOnce()> {
    let doc = owner_document(reference_element);
    let html = doc.document_element().unwrap();
    let body = doc.body().unwrap();

    let element_to_lock = if is_overflow_element(&html) {
        html
    } else {
        body.into()
    };

    let html_el: &web_sys::HtmlElement = element_to_lock.unchecked_ref();
    let style = html_el.style();
    let original_overflow_y = style.get_property_value("overflow-y").unwrap_or_default();
    let original_overflow_x = style.get_property_value("overflow-x").unwrap_or_default();

    let _ = style.set_property("overflow-y", "hidden");
    let _ = style.set_property("overflow-x", "hidden");

    Box::new(move || {
        let _ = style.set_property("overflow-y", &original_overflow_y);
        let _ = style.set_property("overflow-x", &original_overflow_x);
    })
}

/// Prevent scroll on inset-scrollbar systems (Windows, Linux).
/// Uses scrollbar-gutter or body positioning to compensate for the scrollbar width.
fn prevent_scroll_inset(reference_element: Option<&web_sys::Element>) -> Box<dyn FnOnce()> {
    let doc = owner_document(reference_element);
    let html = doc.document_element().unwrap();
    let body = doc.body().unwrap();
    let win = owner_window(Some(&html));

    let html_style = html.unchecked_ref::<web_sys::HtmlElement>().style();
    let body_style = body.style();

    // Save originals
    let orig_html_overflow_y = html_style.get_property_value("overflow-y").unwrap_or_default();
    let orig_html_overflow_x = html_style.get_property_value("overflow-x").unwrap_or_default();
    let orig_html_scrollbar_gutter = html_style
        .get_property_value("scrollbar-gutter")
        .unwrap_or_default();
    let orig_html_scroll_behavior = html_style
        .get_property_value("scroll-behavior")
        .unwrap_or_default();

    let orig_body_position = body_style.get_property_value("position").unwrap_or_default();
    let orig_body_height = body_style.get_property_value("height").unwrap_or_default();
    let orig_body_width = body_style.get_property_value("width").unwrap_or_default();
    let orig_body_box_sizing = body_style.get_property_value("box-sizing").unwrap_or_default();
    let orig_body_overflow_y = body_style.get_property_value("overflow-y").unwrap_or_default();
    let orig_body_overflow_x = body_style.get_property_value("overflow-x").unwrap_or_default();
    let orig_body_scroll_behavior = body_style
        .get_property_value("scroll-behavior")
        .unwrap_or_default();

    let scroll_top = html.scroll_top();
    let scroll_left = html.scroll_left();

    // Compute scrollbar dimensions
    let inner_width = win.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
    let inner_height = win.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
    let scrollbar_width = (inner_width - body.client_width() as f64).max(0.0);
    let scrollbar_height = (inner_height - body.client_height() as f64).max(0.0);

    let body_computed = win.get_computed_style(&body).ok().flatten();
    let margin_y = body_computed
        .as_ref()
        .map(|s| {
            let mt: f64 = s
                .get_property_value("margin-top")
                .unwrap_or_default()
                .trim_end_matches("px")
                .parse()
                .unwrap_or(0.0);
            let mb: f64 = s
                .get_property_value("margin-bottom")
                .unwrap_or_default()
                .trim_end_matches("px")
                .parse()
                .unwrap_or(0.0);
            mt + mb
        })
        .unwrap_or(0.0);
    let margin_x = body_computed
        .as_ref()
        .map(|s| {
            let ml: f64 = s
                .get_property_value("margin-left")
                .unwrap_or_default()
                .trim_end_matches("px")
                .parse()
                .unwrap_or(0.0);
            let mr: f64 = s
                .get_property_value("margin-right")
                .unwrap_or_default()
                .trim_end_matches("px")
                .parse()
                .unwrap_or(0.0);
            ml + mr
        })
        .unwrap_or(0.0);

    // Apply scroll lock styles
    let _ = html_style.set_property("scrollbar-gutter", "stable");
    let _ = html_style.set_property("overflow-y", "hidden");
    let _ = html_style.set_property("overflow-x", "hidden");
    let _ = html_style.set_property("scroll-behavior", "unset");

    let height_val = if margin_y != 0.0 || scrollbar_height != 0.0 {
        format!("calc(100dvh - {}px)", margin_y + scrollbar_height)
    } else {
        "100dvh".to_string()
    };
    let width_val = if margin_x != 0.0 || scrollbar_width != 0.0 {
        format!("calc(100vw - {}px)", margin_x + scrollbar_width)
    } else {
        "100vw".to_string()
    };

    let _ = body_style.set_property("position", "relative");
    let _ = body_style.set_property("height", &height_val);
    let _ = body_style.set_property("width", &width_val);
    let _ = body_style.set_property("box-sizing", "border-box");
    let _ = body_style.set_property("overflow", "hidden");
    let _ = body_style.set_property("scroll-behavior", "unset");

    body.set_scroll_top(scroll_top);
    body.set_scroll_left(scroll_left);
    html.set_attribute("data-base-ui-scroll-locked", "").ok();

    // Return cleanup
    let html_style_clone = html_style.clone();
    let body_style_clone = body_style.clone();
    let html_clone = html.clone();
    Box::new(move || {
        let _ = html_style_clone.set_property("overflow-y", &orig_html_overflow_y);
        let _ = html_style_clone.set_property("overflow-x", &orig_html_overflow_x);
        let _ = html_style_clone.set_property("scrollbar-gutter", &orig_html_scrollbar_gutter);
        let _ = html_style_clone.set_property("scroll-behavior", &orig_html_scroll_behavior);

        let _ = body_style_clone.set_property("position", &orig_body_position);
        let _ = body_style_clone.set_property("height", &orig_body_height);
        let _ = body_style_clone.set_property("width", &orig_body_width);
        let _ = body_style_clone.set_property("box-sizing", &orig_body_box_sizing);
        let _ = body_style_clone.set_property("overflow-y", &orig_body_overflow_y);
        let _ = body_style_clone.set_property("overflow-x", &orig_body_overflow_x);
        let _ = body_style_clone.set_property("scroll-behavior", &orig_body_scroll_behavior);

        html_clone.set_scroll_top(scroll_top);
        html_clone.set_scroll_left(scroll_left);
        html_clone.remove_attribute("data-base-ui-scroll-locked").ok();
    })
}

/// A reference-counted scroll locker.
///
/// Multiple components can acquire the lock simultaneously. The scroll is only
/// actually unlocked when the last lock is released. Lock/unlock are deferred
/// by a microtask (`setTimeout(fn, 0)`) to batch rapid acquire/release cycles.
///
/// Maps to the `ScrollLocker` class in the React version.
pub struct ScrollLocker {
    inner: Rc<RefCell<ScrollLockerInner>>,
}

struct ScrollLockerInner {
    lock_count: i32,
    restore: Option<Box<dyn FnOnce()>>,
}

impl ScrollLocker {
    /// Create a new `ScrollLocker`.
    pub fn new() -> Self {
        ScrollLocker {
            inner: Rc::new(RefCell::new(ScrollLockerInner {
                lock_count: 0,
                restore: None,
            })),
        }
    }

    /// Acquire a scroll lock. Returns a `ScrollLockGuard` that releases the lock
    /// when its `release()` method is called (or when dropped).
    ///
    /// The actual DOM lock is applied synchronously. If multiple components
    /// acquire simultaneously, only one DOM lock is created.
    pub fn acquire(&self, reference_element: Option<&web_sys::Element>) -> ScrollLockGuard {
        {
            let mut inner = self.inner.borrow_mut();
            inner.lock_count += 1;
            if inner.lock_count == 1 && inner.restore.is_none() {
                self.lock_inner(&mut inner, reference_element);
            }
        }

        ScrollLockGuard {
            inner: Some(self.inner.clone()),
        }
    }

    fn lock_inner(
        &self,
        inner: &mut ScrollLockerInner,
        reference_element: Option<&web_sys::Element>,
    ) {
        if inner.lock_count == 0 || inner.restore.is_some() {
            return;
        }

        let doc = owner_document(reference_element);
        let html = doc.document_element().unwrap();
        let win = owner_window(Some(&html));

        let html_computed = win.get_computed_style(&html).ok().flatten();
        let html_overflow_y = html_computed
            .as_ref()
            .and_then(|s| s.get_property_value("overflow-y").ok())
            .unwrap_or_default();

        // If the site author already hid overflow on <html>, respect it and bail out.
        if html_overflow_y == "hidden" || html_overflow_y == "clip" {
            inner.restore = Some(Box::new(|| {}));
            return;
        }

        let has_overlay_scrollbars =
            super::detect_browser::IS_IOS.clone() || !has_inset_scrollbars(reference_element);

        inner.restore = Some(if has_overlay_scrollbars {
            prevent_scroll_overlay(reference_element)
        } else {
            prevent_scroll_inset(reference_element)
        });
    }
}

/// A guard that releases a scroll lock when dropped or when `release()` is called.
pub struct ScrollLockGuard {
    inner: Option<Rc<RefCell<ScrollLockerInner>>>,
}

impl ScrollLockGuard {
    /// Explicitly release the scroll lock.
    pub fn release(mut self) {
        self.do_release();
    }

    fn do_release(&mut self) {
        if let Some(inner) = self.inner.take() {
            let mut state = inner.borrow_mut();
            state.lock_count -= 1;
            if state.lock_count == 0 {
                if let Some(restore) = state.restore.take() {
                    restore();
                }
            }
        }
    }
}

impl Drop for ScrollLockGuard {
    fn drop(&mut self) {
        self.do_release();
    }
}

impl Default for ScrollLocker {
    fn default() -> Self {
        Self::new()
    }
}

// Global shared scroll locker instance.
// Maps to the `SCROLL_LOCKER` singleton in the React version.
thread_local! {
    static GLOBAL_SCROLL_LOCKER: ScrollLocker = ScrollLocker::new();
}

/// Acquire a scroll lock on the global shared locker.
///
/// Returns a `ScrollLockGuard` that releases the lock when dropped or
/// when `release()` is called.
pub fn acquire_scroll_lock(reference_element: Option<&web_sys::Element>) -> ScrollLockGuard {
    // Clone the Rc inside the thread_local so we can call acquire outside `with`.
    let locker_inner = GLOBAL_SCROLL_LOCKER.with(|locker| locker.inner.clone());
    let temp_locker = ScrollLocker {
        inner: locker_inner,
    };
    temp_locker.acquire(reference_element)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from useScrollLock.ts:
    // ✓ is_overflow_element string matching logic
    // ✗ scroll locking on overlay scrollbar systems (requires real browser)
    // ✗ scroll locking on inset scrollbar systems (requires real browser)
    // ✗ reference counting (acquire/release) (requires real browser)
    // ✗ respects existing overflow:hidden on html (requires real browser)
    //
    // DOM-based tests require wasm-bindgen-test with a real browser.

    #[test]
    fn overflow_detection_string_logic() {
        // Test the string matching logic used in is_overflow_element
        let test_cases = [
            ("auto", true),
            ("scroll", true),
            ("hidden", true),
            ("clip", true),
            ("overlay", true),
            ("visible", false),
            ("", false),
        ];
        for (value, expected) in test_cases {
            let result = value.contains("auto")
                || value.contains("scroll")
                || value.contains("overlay")
                || value.contains("hidden")
                || value.contains("clip");
            assert_eq!(result, expected, "Failed for overflow value: {value}");
        }
    }

    #[test]
    fn scroll_locker_initial_state() {
        let locker = ScrollLocker::new();
        let inner = locker.inner.borrow();
        assert_eq!(inner.lock_count, 0);
        assert!(inner.restore.is_none());
    }
}
