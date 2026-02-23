//! Empty/no-op constants.
//!
//! Ported from `@base-ui/utils/empty`.
//! In Rust, frozen empty collections aren't needed (immutability is the default),
//! but a no-op function is still useful as a default callback.

/// A no-op function. Useful as a default callback.
///
/// Maps to `NOOP` in the React version.
pub fn noop() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_does_nothing() {
        noop(); // should not panic
    }
}
