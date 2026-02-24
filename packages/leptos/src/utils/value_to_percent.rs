//! Value-to-percentage conversion.
//!
//! Ported from `packages/react/src/utils/valueToPercent.ts`.

/// Converts a value within a range to a percentage (0–100).
///
/// Maps to `valueToPercent(value, min, max)` in the React version.
pub fn value_to_percent(value: f64, min: f64, max: f64) -> f64 {
    ((value - min) * 100.0) / (max - min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_midpoint() {
        assert_eq!(value_to_percent(50.0, 0.0, 100.0), 50.0);
    }

    #[test]
    fn converts_min() {
        assert_eq!(value_to_percent(0.0, 0.0, 100.0), 0.0);
    }

    #[test]
    fn converts_max() {
        assert_eq!(value_to_percent(100.0, 0.0, 100.0), 100.0);
    }

    #[test]
    fn handles_custom_range() {
        assert_eq!(value_to_percent(5.0, 0.0, 10.0), 50.0);
    }

    #[test]
    fn handles_offset_range() {
        assert_eq!(value_to_percent(75.0, 50.0, 100.0), 50.0);
    }
}
