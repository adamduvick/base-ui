//! Floating UI integration (positioning system).
//!
//! Ported from `@base-ui/react/floating-ui-react`.
//!
//! ## Status: Stub / Pending Full Implementation
//!
//! The React version wraps `@floating-ui/react` and `@floating-ui/react-dom`,
//! providing hooks and components for positioning floating elements (popovers,
//! tooltips, menus, etc.) relative to reference elements.
//!
//! A full Leptos port requires either:
//! - WASM bindings to the floating-ui JavaScript library, or
//! - A native Rust positioning implementation
//!
//! This module currently provides only type definitions and documentation.
//! Popup/overlay components that depend on floating-ui positioning will need
//! this module to be fully implemented before they can function.
//!
//! ## What's defined (types only)
//! - `Side` — placement side enum
//! - `Alignment` — placement alignment enum
//! - `Placement` — combined side + alignment
//!
//! ## What's not yet implemented
//! - `useFloating` hook — core positioning logic
//! - `useClick`, `useDismiss`, `useFocus`, `useHover` — interaction hooks
//! - `useListNavigation`, `useTypeahead` — list interaction hooks
//! - `FloatingPortal`, `FloatingFocusManager` — DOM management components
//! - `FloatingTree`, `FloatingNode` — nested floating element tree
//! - Middleware: `offset`, `flip`, `shift`, `size`, `arrow`, `hide`, `autoPlacement`
//! - `autoUpdate` — position update scheduling

/// Side of the reference element where the floating element is placed.
///
/// Maps to the `Side` type from `@floating-ui/utils`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Side {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Side::Top => "top",
            Side::Bottom => "bottom",
            Side::Left => "left",
            Side::Right => "right",
        }
    }

    /// Returns the opposite side.
    pub fn opposite(&self) -> Self {
        match self {
            Side::Top => Side::Bottom,
            Side::Bottom => Side::Top,
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

/// Alignment of the floating element along the side axis.
///
/// Maps to the `Alignment` type from `@floating-ui/utils`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Alignment {
    Start,
    Center,
    End,
}

impl Alignment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Alignment::Start => "start",
            Alignment::Center => "center",
            Alignment::End => "end",
        }
    }
}

/// Combined placement: side + optional alignment.
///
/// Maps to the `Placement` type from `@floating-ui/utils`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Placement {
    pub side: Side,
    pub alignment: Option<Alignment>,
}

impl Default for Placement {
    fn default() -> Self {
        Placement {
            side: Side::Bottom,
            alignment: None,
        }
    }
}

impl Placement {
    pub fn new(side: Side, alignment: Option<Alignment>) -> Self {
        Placement { side, alignment }
    }

    /// Returns the placement as a string (e.g., "top", "bottom-start").
    pub fn as_str(&self) -> String {
        match self.alignment {
            Some(align) => format!("{}-{}", self.side.as_str(), align.as_str()),
            None => self.side.as_str().to_string(),
        }
    }
}

impl std::fmt::Display for Placement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // This module is a stub. Tests cover only the type definitions.
    // Full positioning tests require the floating-ui implementation.

    #[test]
    fn side_as_str() {
        assert_eq!(Side::Top.as_str(), "top");
        assert_eq!(Side::Bottom.as_str(), "bottom");
        assert_eq!(Side::Left.as_str(), "left");
        assert_eq!(Side::Right.as_str(), "right");
    }

    #[test]
    fn side_opposite() {
        assert_eq!(Side::Top.opposite(), Side::Bottom);
        assert_eq!(Side::Left.opposite(), Side::Right);
    }

    #[test]
    fn placement_as_str() {
        let p = Placement::new(Side::Top, None);
        assert_eq!(p.as_str(), "top");

        let p = Placement::new(Side::Bottom, Some(Alignment::Start));
        assert_eq!(p.as_str(), "bottom-start");

        let p = Placement::new(Side::Left, Some(Alignment::End));
        assert_eq!(p.as_str(), "left-end");
    }

    #[test]
    fn placement_default_is_bottom() {
        let p = Placement::default();
        assert_eq!(p.side, Side::Bottom);
        assert_eq!(p.alignment, None);
    }

    #[test]
    fn alignment_as_str() {
        assert_eq!(Alignment::Start.as_str(), "start");
        assert_eq!(Alignment::Center.as_str(), "center");
        assert_eq!(Alignment::End.as_str(), "end");
    }
}
