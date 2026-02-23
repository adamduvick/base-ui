//! Previous value tracking.
//!
//! Ported from `@base-ui/utils/usePreviousValue`.
//! Maps the React `usePreviousValue` hook to a Leptos signal-based approach.
//!
//! In React, this uses `useState` to track current/previous pairs. In Leptos,
//! we use a memo that tracks the previous value of a signal.

use leptos::prelude::*;

/// Track the previous value of a signal.
///
/// Returns a read signal that holds the previous value. On the first read
/// (before any changes), returns `None`.
///
/// Maps to `usePreviousValue(value)` in the React version.
///
/// # Arguments
/// * `value` - The signal whose previous value should be tracked.
///
/// # Returns
/// A `Signal<Option<T>>` that holds the previous value, or `None` if
/// the value hasn't changed yet.
pub fn use_previous_value<T>(value: Signal<T>) -> Signal<Option<T>>
where
    T: Send + Sync + Clone + PartialEq + 'static,
{
    let (previous, set_previous) = signal::<Option<T>>(None);
    let (current, set_current) = signal::<Option<T>>(None);

    // Create an effect that updates previous/current when value changes
    Effect::new(move |_| {
        let new_value = value.get();
        let cur = current.get_untracked();
        match cur {
            Some(ref cur_val) if *cur_val == new_value => {
                // Value hasn't changed, don't update
            }
            _ => {
                set_previous.set(cur);
                set_current.set(Some(new_value));
            }
        }
    });

    previous.into()
}

#[cfg(test)]
mod tests {
    // Ported from usePreviousValue.test.tsx:
    // ✗ should return null on the first render (requires Leptos runtime)
    // ✗ should return the previous value on subsequent renders (requires Leptos runtime)
    // ✗ should work with primitive values (requires Leptos runtime)
    // ✗ should ignore renders where the value does not change (requires Leptos runtime)
    // ✗ should work with object values (requires Leptos runtime)
    // ✗ should handle undefined and null values (requires Leptos runtime)
    // ✗ should handle rapid value changes (requires Leptos runtime)
    // ✗ should maintain type safety (enforced by Rust's type system at compile time)
    //
    // All behavioral tests require a Leptos reactive runtime (wasm-bindgen-test).
    // Type safety is enforced at compile time by Rust's type system.
}
