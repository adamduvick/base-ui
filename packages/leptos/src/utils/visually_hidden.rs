//! CSS styles for visually hidden elements.
//!
//! Ported from `@base-ui/utils/visuallyHidden`.
//! Provides CSS property strings for hiding elements visually while keeping them
//! accessible to screen readers.

/// CSS style string for visually hidden elements (fixed positioning).
///
/// Maps to `visuallyHidden` in the React version.
pub const VISUALLY_HIDDEN: &str = "\
clip-path:inset(50%);\
overflow:hidden;\
white-space:nowrap;\
border:0;\
padding:0;\
width:1px;\
height:1px;\
margin:-1px;\
position:fixed;\
top:0;\
left:0";

/// CSS style string for visually hidden input elements (absolute positioning).
///
/// Maps to `visuallyHiddenInput` in the React version.
pub const VISUALLY_HIDDEN_INPUT: &str = "\
clip-path:inset(50%);\
overflow:hidden;\
white-space:nowrap;\
border:0;\
padding:0;\
width:1px;\
height:1px;\
margin:-1px;\
position:absolute";

/// Apply visually-hidden styles to an `HtmlElement`.
///
/// This is a convenience function that sets the `style` attribute directly.
pub fn apply_visually_hidden(element: &web_sys::HtmlElement) {
    let _ = element.style().set_property("clip-path", "inset(50%)");
    let _ = element.style().set_property("overflow", "hidden");
    let _ = element.style().set_property("white-space", "nowrap");
    let _ = element.style().set_property("border", "0");
    let _ = element.style().set_property("padding", "0");
    let _ = element.style().set_property("width", "1px");
    let _ = element.style().set_property("height", "1px");
    let _ = element.style().set_property("margin", "-1px");
    let _ = element.style().set_property("position", "fixed");
    let _ = element.style().set_property("top", "0");
    let _ = element.style().set_property("left", "0");
}

/// Apply visually-hidden input styles to an `HtmlElement`.
///
/// Uses absolute positioning instead of fixed, suitable for form inputs.
pub fn apply_visually_hidden_input(element: &web_sys::HtmlElement) {
    let _ = element.style().set_property("clip-path", "inset(50%)");
    let _ = element.style().set_property("overflow", "hidden");
    let _ = element.style().set_property("white-space", "nowrap");
    let _ = element.style().set_property("border", "0");
    let _ = element.style().set_property("padding", "0");
    let _ = element.style().set_property("width", "1px");
    let _ = element.style().set_property("height", "1px");
    let _ = element.style().set_property("margin", "-1px");
    let _ = element.style().set_property("position", "absolute");
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from visuallyHidden.ts:
    // ✓ VISUALLY_HIDDEN contains expected CSS properties
    // ✓ VISUALLY_HIDDEN_INPUT uses absolute positioning

    #[test]
    fn visually_hidden_contains_expected_properties() {
        assert!(VISUALLY_HIDDEN.contains("clip-path:inset(50%)"));
        assert!(VISUALLY_HIDDEN.contains("overflow:hidden"));
        assert!(VISUALLY_HIDDEN.contains("white-space:nowrap"));
        assert!(VISUALLY_HIDDEN.contains("width:1px"));
        assert!(VISUALLY_HIDDEN.contains("height:1px"));
        assert!(VISUALLY_HIDDEN.contains("margin:-1px"));
        assert!(VISUALLY_HIDDEN.contains("position:fixed"));
        assert!(VISUALLY_HIDDEN.contains("top:0"));
        assert!(VISUALLY_HIDDEN.contains("left:0"));
    }

    #[test]
    fn visually_hidden_input_uses_absolute_positioning() {
        assert!(VISUALLY_HIDDEN_INPUT.contains("position:absolute"));
        assert!(!VISUALLY_HIDDEN_INPUT.contains("position:fixed"));
        // Should not contain top/left (only the fixed variant has those)
        assert!(!VISUALLY_HIDDEN_INPUT.contains("top:0"));
    }
}
