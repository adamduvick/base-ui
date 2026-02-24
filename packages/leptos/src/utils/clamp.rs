//! Numeric clamping utility.
//!
//! Ported from `packages/react/src/utils/clamp.ts`.

/// Clamps `val` between `min` and `max` (inclusive).
///
/// Maps to `clamp(val, min, max)` in the React version.
pub fn clamp(val: f64, min: f64, max: f64) -> f64 {
    val.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_to_min() {
        assert_eq!(clamp(-5.0, 0.0, 100.0), 0.0);
    }

    #[test]
    fn clamps_to_max() {
        assert_eq!(clamp(200.0, 0.0, 100.0), 100.0);
    }

    #[test]
    fn passes_through_in_range() {
        assert_eq!(clamp(50.0, 0.0, 100.0), 50.0);
    }

    #[test]
    fn handles_equal_min_max() {
        assert_eq!(clamp(50.0, 10.0, 10.0), 10.0);
    }

    #[test]
    fn handles_boundary_values() {
        assert_eq!(clamp(0.0, 0.0, 100.0), 0.0);
        assert_eq!(clamp(100.0, 0.0, 100.0), 100.0);
    }
}
