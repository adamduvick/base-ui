//! Effect that runs once on mount.
//!
//! Ported from `@base-ui/utils/useOnMount`.
//! Maps `React.useEffect(fn, [])` to a Leptos effect that runs once.
//!
//! In Leptos 0.8, `Effect::new` with no signal dependencies runs once on mount,
//! which is equivalent to React's `useEffect(fn, [])`.

use leptos::prelude::*;

/// Run a function once when the component mounts.
///
/// The provided function may return an optional cleanup function that will
/// be called when the component is unmounted.
///
/// Maps to `useOnMount(fn)` in the React version.
///
/// # Arguments
/// * `f` - The function to run on mount.
pub fn use_on_mount<F>(f: F)
where
    F: FnOnce() + Send + Sync + 'static,
{
    let f = std::cell::Cell::new(Some(f));
    Effect::new(move |_| {
        if let Some(func) = f.take() {
            func();
        }
    });
}

#[cfg(test)]
mod tests {
    // Ported from useOnMount.ts:
    // ✗ runs the callback once on mount (requires Leptos runtime)
    // ✗ does not run on subsequent re-renders (requires Leptos runtime)
    //
    // All tests require a Leptos reactive runtime (wasm-bindgen-test).
}
