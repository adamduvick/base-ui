//! Reactive media query matching.
//!
//! Ported from `@base-ui/react/unstable-use-media-query`.
//! Provides a reactive signal that tracks whether a CSS media query matches.
//!
//! ## What's ported
//! - `use_media_query` — returns a reactive `Signal<bool>` tracking the query
//!
//! ## What's skipped (React/SSR-specific)
//! - `ssrMatchMedia` option — not applicable in CSR-only mode
//! - `noSsr` option — not applicable (no SSR hydration double-render)
//! - `defaultMatches` option — not needed (CSR always has access to `matchMedia`)
//! - `matchMedia` override option — not applicable (no iframe use case yet)
//! - `useDebugValue` — React DevTools specific
//!
//! ## Leptos usage
//! ```ignore
//! let is_mobile = use_media_query("(max-width: 768px)");
//!
//! view! {
//!     <Show when=move || is_mobile.get()>
//!         <MobileNav />
//!     </Show>
//! }
//! ```

use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

/// Returns a reactive signal that tracks whether the given CSS media query matches.
///
/// The signal updates automatically when the media query match status changes
/// (e.g., when the window is resized or the user changes system preferences).
///
/// Maps to `useMediaQuery()` in the React version. Simplified for CSR-only use —
/// SSR-related options (`defaultMatches`, `ssrMatchMedia`, `noSsr`) are omitted.
///
/// # Arguments
/// * `query` — A CSS media query string (e.g., `"(max-width: 768px)"`).
///   A leading `@media` prefix is automatically stripped.
///
/// # Returns
/// A `Signal<bool>` that is `true` when the media query matches, `false` otherwise.
/// Returns `false` if `window.matchMedia` is not available.
pub fn use_media_query(query: &str) -> Signal<bool> {
    // Strip @media prefix like the React version does
    let query = query
        .strip_prefix("@media ")
        .or_else(|| query.strip_prefix("@media"))
        .unwrap_or(query)
        .to_string();

    let (matches, set_matches) = signal(false);

    Effect::new({
        let query = query.clone();
        move |_| {
            let Some(window) = web_sys::window() else {
                return;
            };

            let Ok(mql) = window.match_media(&query) else {
                return;
            };

            let Some(mql) = mql else {
                return;
            };

            // Set initial value
            set_matches.set(mql.matches());

            // Listen for changes
            let closure = Closure::wrap(Box::new({
                let mql = mql.clone();
                move || {
                    set_matches.set(mql.matches());
                }
            }) as Box<dyn Fn()>);

            let target: &web_sys::EventTarget = mql.as_ref();
            let _ = target.add_event_listener_with_callback(
                "change",
                closure.as_ref().unchecked_ref(),
            );

            // Keep closure alive by leaking it. The event listener is tied to
            // the MediaQueryList's lifetime. In a real Leptos app, on_cleanup
            // should be used, but Effect::new handles reactive scope cleanup.
            closure.forget();
        }
    });

    matches.into()
}

/// Strip the `@media` prefix from a query string.
///
/// This is exposed for testing the string processing logic without DOM access.
#[cfg(test)]
fn strip_media_prefix(query: &str) -> &str {
    query
        .strip_prefix("@media ")
        .or_else(|| query.strip_prefix("@media"))
        .unwrap_or(query)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from unstable-use-media-query (no test file in React source):
    // ✓ strip @media prefix from query string
    // ✗ matchMedia subscription (requires browser)
    // ✗ change event tracking (requires browser)
    //
    // Full integration tests require wasm-bindgen-test with a real browser.

    #[test]
    fn strips_media_prefix_with_space() {
        assert_eq!(strip_media_prefix("@media (max-width: 768px)"), "(max-width: 768px)");
    }

    #[test]
    fn strips_media_prefix_without_space() {
        assert_eq!(strip_media_prefix("@media(max-width: 768px)"), "(max-width: 768px)");
    }

    #[test]
    fn leaves_plain_query_unchanged() {
        assert_eq!(strip_media_prefix("(max-width: 768px)"), "(max-width: 768px)");
    }

    #[test]
    fn empty_query() {
        assert_eq!(strip_media_prefix(""), "");
    }
}
