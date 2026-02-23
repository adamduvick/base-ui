//! Text direction (LTR/RTL) provider.
//!
//! Ported from `@base-ui/react/direction-provider`.
//! Provides a text direction context for Base UI components to support
//! right-to-left (RTL) layouts.
//!
//! ## What's ported
//! - `TextDirection` — direction enum (`Ltr` / `Rtl`)
//! - `provide_direction` — provides direction context to children
//! - `use_direction` — consumes direction context from ancestor
//!
//! ## What's skipped (React-specific)
//! - `<DirectionProvider>` JSX component wrapper — use `provide_direction()` directly
//! - `DirectionProvider.Props` namespace types
//!
//! ## Leptos usage
//! ```ignore
//! // In a parent component:
//! provide_direction(TextDirection::Rtl);
//!
//! // In a child component:
//! let dir = use_direction();
//! // dir == TextDirection::Ltr (default) unless overridden
//! ```

use leptos::prelude::*;

/// Text direction for layout and reading order.
///
/// Maps to `TextDirection` (`'ltr' | 'rtl'`) in the React version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum TextDirection {
    /// Left-to-right (default).
    #[default]
    Ltr,
    /// Right-to-left.
    Rtl,
}

impl TextDirection {
    /// Returns the CSS/HTML `dir` attribute value.
    pub fn as_str(&self) -> &'static str {
        match self {
            TextDirection::Ltr => "ltr",
            TextDirection::Rtl => "rtl",
        }
    }

    /// Returns `true` if this is right-to-left.
    pub fn is_rtl(&self) -> bool {
        *self == TextDirection::Rtl
    }
}

/// Provide text direction context to descendant components.
///
/// Call this in a parent component's body to override the direction
/// for all children that call `use_direction()`.
///
/// Maps to `<DirectionProvider>` in the React version.
pub fn provide_direction(direction: TextDirection) {
    provide_context(direction);
}

/// Consume the text direction context from an ancestor `provide_direction()` call.
///
/// Returns `TextDirection::Ltr` if no provider exists in the tree.
///
/// Maps to `useDirection()` in the React version.
pub fn use_direction() -> TextDirection {
    use_context::<TextDirection>().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from DirectionProvider (no test file in React source):
    // ✓ TextDirection defaults to Ltr
    // ✓ TextDirection::as_str returns correct CSS values
    // ✓ TextDirection::is_rtl works correctly
    //
    // Context integration tests (provide_direction + use_direction)
    // require a Leptos runtime and are not tested here.

    #[test]
    fn default_is_ltr() {
        assert_eq!(TextDirection::default(), TextDirection::Ltr);
    }

    #[test]
    fn as_str_values() {
        assert_eq!(TextDirection::Ltr.as_str(), "ltr");
        assert_eq!(TextDirection::Rtl.as_str(), "rtl");
    }

    #[test]
    fn is_rtl() {
        assert!(!TextDirection::Ltr.is_rtl());
        assert!(TextDirection::Rtl.is_rtl());
    }

    #[test]
    fn equality() {
        assert_eq!(TextDirection::Ltr, TextDirection::Ltr);
        assert_eq!(TextDirection::Rtl, TextDirection::Rtl);
        assert_ne!(TextDirection::Ltr, TextDirection::Rtl);
    }
}
