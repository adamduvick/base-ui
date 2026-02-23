//! Controlled/uncontrolled state management.
//!
//! Ported from `@base-ui/utils/useControlled`.
//! Maps the React controlled/uncontrolled pattern to Leptos signals.
//!
//! In React, `useControlled` returns `[value, setValue]` where `setValue` is a no-op
//! when controlled. In Leptos, we use `ReadSignal`/`WriteSignal` pairs with the
//! same semantics: when a controlled value is provided, the write signal updates
//! are ignored and the controlled value is always used.

use leptos::prelude::*;

/// Create a controlled-or-uncontrolled value signal pair.
///
/// If `controlled` is `Some(signal)`, the returned read signal always reflects
/// the controlled value and the write callback is a no-op.
///
/// If `controlled` is `None`, the returned signals behave as a normal
/// `signal(default_value)` pair.
///
/// Maps to `useControlled({ controlled, default, name })` in the React version.
///
/// # Arguments
/// * `controlled` - An optional signal providing the controlled value.
/// * `default_value` - The initial value when uncontrolled.
///
/// # Returns
/// A tuple of `(read_signal, write_callback)` where the write callback
/// only updates the internal state when uncontrolled.
pub fn use_controlled<T>(
    controlled: Option<Signal<T>>,
    default_value: T,
) -> (Signal<T>, WriteSignal<T>)
where
    T: Send + Sync + Clone + 'static,
{
    let (internal, set_internal) = signal(default_value);

    match controlled {
        Some(controlled_signal) => {
            // When controlled, always use the controlled signal for reads.
            // Writes go to the internal signal but are effectively ignored
            // since reads come from the controlled signal.
            (controlled_signal, set_internal)
        }
        None => {
            // When uncontrolled, use the internal signal for both reads and writes.
            (internal.into(), set_internal)
        }
    }
}

// Note: The React version includes dev-mode warnings for switching between
// controlled/uncontrolled and changing defaultValue. In Leptos, the controlled
// vs uncontrolled decision is made at component creation time and cannot change
// (it's determined by whether `controlled` is Some or None), so these warnings
// are unnecessary.

#[cfg(test)]
mod tests {
    // Ported from useControlled.test.tsx:
    // ✗ works correctly when is not controlled (requires Leptos runtime)
    // ✗ works correctly when is controlled (requires Leptos runtime)
    // ✗ warns when switching from uncontrolled to controlled (N/A — type system prevents this)
    // ✗ warns when switching from controlled to uncontrolled (N/A — type system prevents this)
    // ✗ warns when defaultValue changes (N/A — default_value is a plain value, not reactive)
    //
    // The React warnings about switching controlled/uncontrolled are not applicable
    // because in Leptos the decision is encoded in the type system at creation time
    // (Option<Signal<T>> is either Some or None, fixed at call site).
}
