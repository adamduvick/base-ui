//! Layout orientation type.
//!
//! Ported from `packages/react/src/utils/types.ts` (`Orientation` type).

/// Layout orientation for components like separators, sliders, toolbars.
///
/// Maps to `Orientation` (`'horizontal' | 'vertical'`) in the React version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

impl Orientation {
    /// Returns the CSS/ARIA attribute value.
    pub fn as_str(&self) -> &'static str {
        match self {
            Orientation::Horizontal => "horizontal",
            Orientation::Vertical => "vertical",
        }
    }
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_horizontal() {
        assert_eq!(Orientation::default(), Orientation::Horizontal);
    }

    #[test]
    fn as_str_values() {
        assert_eq!(Orientation::Horizontal.as_str(), "horizontal");
        assert_eq!(Orientation::Vertical.as_str(), "vertical");
    }

    #[test]
    fn display_matches_as_str() {
        assert_eq!(format!("{}", Orientation::Horizontal), "horizontal");
        assert_eq!(format!("{}", Orientation::Vertical), "vertical");
    }
}
