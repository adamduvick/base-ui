//! Element disabled-state detection.
//!
//! Ported from `@base-ui/utils/isElementDisabled`.
//! Checks whether an HTML element is disabled via the `disabled` attribute
//! or `aria-disabled="true"`.

use web_sys::HtmlElement;

/// Check whether an element is disabled.
///
/// Returns `true` if:
/// - The element is `None`
/// - The element has a `disabled` attribute
/// - The element has `aria-disabled="true"`
///
/// Maps to `isElementDisabled(element)` in the React version.
pub fn is_element_disabled(element: Option<&HtmlElement>) -> bool {
    match element {
        None => true,
        Some(el) => {
            let el_as_element: &web_sys::Element = el.as_ref();
            el_as_element.has_attribute("disabled")
                || el_as_element.get_attribute("aria-disabled").as_deref() == Some("true")
        }
    }
}

// Note: Full integration tests require wasm-bindgen-test with a real DOM.
#[cfg(test)]
mod tests {
    use super::*;

    // Ported from isElementDisabled.ts:
    // ✓ returns true for None
    // ✗ returns true for element with disabled attribute (requires DOM)
    // ✗ returns true for element with aria-disabled="true" (requires DOM)
    // ✗ returns false for enabled element (requires DOM)

    #[test]
    fn returns_true_for_none() {
        assert!(is_element_disabled(None));
    }
}
