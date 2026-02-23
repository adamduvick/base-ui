//! Mouse bounds detection.
//!
//! Ported from `@base-ui/utils/isMouseWithinBounds`.
//! Checks whether a mouse event's coordinates fall within an element's bounding rect,
//! with a 1px inset to work around Safari's incorrect `mouseleave` events.

use web_sys::MouseEvent;

/// Check whether the mouse position from a `MouseEvent` is within the
/// bounds of the event's current target element.
///
/// Uses a 1px inset to work around Safari incorrectly firing `mouseleave`
/// when the cursor is at the very edge of an element.
/// See: <https://github.com/mui/base-ui/issues/869>
///
/// Maps to `isMouseWithinBounds(event)` in the React version.
///
/// # Arguments
/// * `event` - The mouse event to check
/// * `target` - The element to check bounds against (typically `event.currentTarget`)
pub fn is_mouse_within_bounds(event: &MouseEvent, target: &web_sys::Element) -> bool {
    let rect = target.get_bounding_client_rect();
    let client_x = event.client_x() as f64;
    let client_y = event.client_y() as f64;

    rect.top() + 1.0 <= client_y
        && client_y <= rect.bottom() - 1.0
        && rect.left() + 1.0 <= client_x
        && client_x <= rect.right() - 1.0
}

// Note: Tests require wasm-bindgen-test with a real DOM and mouse events.
#[cfg(test)]
mod tests {
    // Ported from isMouseWithinBounds.ts:
    // ✗ returns true when mouse is within bounds (requires DOM)
    // ✗ returns false when mouse is outside bounds (requires DOM)
    // ✗ handles 1px inset correctly (requires DOM)
    //
    // All tests require a real browser environment with layout.
}
