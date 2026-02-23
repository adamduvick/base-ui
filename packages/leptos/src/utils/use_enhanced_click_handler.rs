//! Enhanced click handler with pointer type detection.
//!
//! Ported from `@base-ui/utils/useEnhancedClickHandler`.
//! Provides a way to determine whether a click was triggered by mouse, touch,
//! pen, or keyboard.
//!
//! In Leptos, this is implemented as a struct that generates event handler
//! closures rather than a React hook.

use std::cell::Cell;
use std::rc::Rc;

/// The type of interaction that triggered a click.
///
/// Maps to `InteractionType` in the React version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionType {
    Mouse,
    Touch,
    Pen,
    Keyboard,
    Unknown,
}

impl InteractionType {
    /// Parse from a `PointerEvent.pointerType` string.
    pub fn from_pointer_type(s: &str) -> Self {
        match s {
            "mouse" => Self::Mouse,
            "touch" => Self::Touch,
            "pen" => Self::Pen,
            _ => Self::Unknown,
        }
    }
}

/// Create enhanced click handler closures that detect the interaction type.
///
/// Returns a pair of `(on_pointer_down, on_click)` closures. Both closures
/// call the provided `handler` with the event and the detected `InteractionType`.
///
/// Maps to `useEnhancedClickHandler(handler)` in the React version.
///
/// # Arguments
/// * `handler` - A callback receiving the event and the interaction type.
///
/// # Returns
/// A tuple of `(on_pointer_down_handler, on_click_handler)` to be attached
/// to the element's `pointerdown` and `click` events respectively.
pub fn create_enhanced_click_handlers<F>(
    handler: F,
) -> (
    impl Fn(web_sys::PointerEvent) + Clone,
    impl Fn(web_sys::MouseEvent) + Clone,
)
where
    F: Fn(InteractionType) + Clone + 'static,
{
    let last_interaction: Rc<Cell<InteractionType>> = Rc::new(Cell::new(InteractionType::Unknown));

    let last_interaction_pd = last_interaction.clone();
    let handler_pd = handler.clone();
    let on_pointer_down = move |event: web_sys::PointerEvent| {
        let event_ref: &web_sys::Event = event.as_ref();
        if event_ref.default_prevented() {
            return;
        }
        let interaction = InteractionType::from_pointer_type(&event.pointer_type());
        last_interaction_pd.set(interaction);
        handler_pd(interaction);
    };

    let on_click = move |event: web_sys::MouseEvent| {
        // event.detail() == 0 means it was triggered by the keyboard
        if event.detail() == 0 {
            handler(InteractionType::Keyboard);
            return;
        }

        // Check if this is actually a PointerEvent (Chrome/Edge use PointerEvent for clicks)
        use wasm_bindgen::JsCast;
        if let Some(pointer_event) = event.dyn_ref::<web_sys::PointerEvent>() {
            let interaction = InteractionType::from_pointer_type(&pointer_event.pointer_type());
            handler(interaction);
        } else {
            handler(last_interaction.get());
        }
        last_interaction.set(InteractionType::Unknown);
    };

    (on_pointer_down, on_click)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from useEnhancedClickHandler.ts:
    // ✓ InteractionType::from_pointer_type parses correctly
    // ✗ on_pointer_down sets interaction type (requires DOM events)
    // ✗ on_click detects keyboard interaction (requires DOM events)
    // ✗ on_click uses PointerEvent.pointerType in Chrome/Edge (requires DOM events)
    // ✗ on_click falls back to last pointer type in Safari/Firefox (requires DOM events)

    #[test]
    fn interaction_type_from_pointer_type() {
        assert_eq!(InteractionType::from_pointer_type("mouse"), InteractionType::Mouse);
        assert_eq!(InteractionType::from_pointer_type("touch"), InteractionType::Touch);
        assert_eq!(InteractionType::from_pointer_type("pen"), InteractionType::Pen);
        assert_eq!(InteractionType::from_pointer_type(""), InteractionType::Unknown);
        assert_eq!(InteractionType::from_pointer_type("other"), InteractionType::Unknown);
    }
}
