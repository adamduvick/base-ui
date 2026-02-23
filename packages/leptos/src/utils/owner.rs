//! Owner document and window accessors.
//!
//! Ported from `@base-ui/utils/owner`.
//! Provides helpers to get the owning `Document` and `Window` for a given element,
//! falling back to the global document/window.

use web_sys::{Document, Element, Window};

/// Get the owner document of an element, falling back to the global `document`.
///
/// Maps to `ownerDocument(node)` in the React version.
pub fn owner_document(node: Option<&Element>) -> Document {
    node.and_then(|n| n.owner_document())
        .or_else(|| web_sys::window().and_then(|w| w.document()))
        .expect("No document available")
}

/// Get the owner window of an element, falling back to the global `window`.
///
/// Maps to `ownerWindow(node)` in the React version (originally from `@floating-ui/utils/dom`).
pub fn owner_window(node: Option<&Element>) -> Window {
    node.and_then(|n| n.owner_document())
        .and_then(|doc| doc.default_view())
        .or_else(web_sys::window)
        .expect("No window available")
}

// Note: Tests require wasm-bindgen-test with a real DOM.
#[cfg(test)]
mod tests {
    // Ported from owner.ts:
    // ✗ ownerDocument returns element's owner document (requires DOM)
    // ✗ ownerDocument falls back to global document (requires DOM)
    // ✗ ownerWindow returns element's owner window (requires DOM)
    // ✗ ownerWindow falls back to global window (requires DOM)
}
