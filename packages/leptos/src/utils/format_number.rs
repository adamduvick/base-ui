//! Number formatting for progress/meter display.
//!
//! Ported from `packages/react/src/utils/formatNumber.ts`.
//!
//! The React version uses `Intl.NumberFormat` for locale-aware formatting.
//! This simplified Rust version provides basic percentage formatting without
//! locale support. Full `Intl.NumberFormat` integration via `js_sys` can be
//! added later if locale-aware formatting is needed.

/// Formats a value as a display string for progress/meter components.
///
/// When no format is specified, the value is treated as a percentage
/// (e.g., `50.0` → `"50%"`). Returns an empty string for `None`.
///
/// Maps to `formatNumberValue(value, locale, format)` in the React version.
/// The `locale` and `format` parameters are omitted in this simplified port.
pub fn format_number_value(value: Option<f64>) -> String {
    match value {
        None => String::new(),
        Some(v) => {
            // The React version divides by 100 then formats with { style: 'percent' },
            // which effectively just appends "%". We skip the Intl step.
            let rounded = v.round();
            if (rounded - v).abs() < f64::EPSILON {
                format!("{}%", rounded as i64)
            } else {
                format!("{v}%")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_integer_percentage() {
        assert_eq!(format_number_value(Some(50.0)), "50%");
    }

    #[test]
    fn formats_zero() {
        assert_eq!(format_number_value(Some(0.0)), "0%");
    }

    #[test]
    fn formats_hundred() {
        assert_eq!(format_number_value(Some(100.0)), "100%");
    }

    #[test]
    fn formats_fractional() {
        assert_eq!(format_number_value(Some(33.5)), "33.5%");
    }

    #[test]
    fn returns_empty_for_none() {
        assert_eq!(format_number_value(None), "");
    }
}
