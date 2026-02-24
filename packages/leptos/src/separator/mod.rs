//! Separator component.
//!
//! Ported from `@base-ui/react/separator`.
//! A visual or semantic separator element accessible to screen readers.
//!
//! ## What's ported
//! - `Separator` component — renders `<div role="separator">` with orientation
//! - `ClassProp<Store<SeparatorState>>` — static string or state-dependent class
//! - `StyleProp<Store<SeparatorState>>` — static string or state-dependent style
//! - `RenderProp<Store<SeparatorState>>` — custom render function to replace the default element
//!
//! ## Reactive state
//! `SeparatorState` uses `#[derive(Store)]` from `reactive_stores` for consistency
//! with other components. Since Separator has no dynamic state (orientation is set
//! once at mount), the store is created but never updated. Callbacks receive
//! `&Store<SeparatorState>` and can read fields via `.orientation().get()`.
//!
//! ## What's skipped (React-specific)
//! - `BaseUIComponentProps` type — use idiomatic Leptos props
//!
//! ## Leptos usage
//! ```ignore
//! use base_ui_leptos::separator::{Separator, SeparatorState, SeparatorStateStoreFields};
//! use base_ui_leptos::utils::orientation::Orientation;
//! use reactive_stores::Store;
//!
//! // Static class
//! view! { <Separator class="my-sep" /> }
//!
//! // State-dependent class (closure)
//! view! {
//!     <Separator class=|store: &Store<SeparatorState>| {
//!         format!("sep sep--{}", store.orientation().get().as_str())
//!     } />
//! }
//! ```

use leptos::prelude::*;
use reactive_stores::Store;

use crate::utils::orientation::Orientation;
use crate::utils::props::{ClassProp, RenderProp, RenderProps, StyleProp};

/// A separator element accessible to screen readers.
/// Renders a `<div>` element with `role="separator"`.
///
/// Maps to `<Separator>` in the React version.
#[component]
pub fn Separator(
    /// The orientation of the separator.
    /// Defaults to `Orientation::Horizontal`.
    #[prop(optional)]
    orientation: Option<Orientation>,
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<SeparatorState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<SeparatorState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<SeparatorState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<SeparatorState>>,
    /// Custom render function. When provided, replaces the default `<div>` element.
    #[prop(optional, into)]
    render: RenderProp<Store<SeparatorState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Div>>,
) -> impl IntoView {
    let orientation = orientation.unwrap_or(Orientation::Horizontal);
    let store = Store::new(SeparatorState { orientation });

    if render.is_custom() {
        let props = RenderProps {
            class: class.resolve(&store),
            style: style.resolve(&store),
            attrs: vec![],
        }
        .attr("role", "separator")
        .attr("aria-orientation", orientation.as_str())
        .attr("data-orientation", orientation.as_str());
        return render.call(props, &store).unwrap().into_any();
    }

    let class_val = class.resolve_option(&store);
    let style_val = style.resolve_option(&store);

    view! {
        <div
            role="separator"
            aria-orientation=orientation.as_str()
            data-orientation=orientation.as_str()
            class=class_val
            style=style_val
            node_ref=node_ref.unwrap_or_default()
        />
    }
    .into_any()
}

/// State of the Separator component.
///
/// Maps to `Separator.State` in the React version.
/// Uses `#[derive(Store)]` for reactive field-level access via `Store<SeparatorState>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Store)]
pub struct SeparatorState {
    pub orientation: Orientation,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from Separator.test.tsx:
    // ✓ SeparatorState stores orientation
    // ✓ default orientation is horizontal
    // ✗ renders a div with the `separator` role (requires browser DOM)
    // ✗ aria-orientation horizontal (requires browser DOM)
    // ✗ aria-orientation vertical (requires browser DOM)
    // ✗ describeConformance (React-specific test infrastructure)

    #[test]
    fn default_orientation_is_horizontal() {
        let state = SeparatorState {
            orientation: Orientation::default(),
        };
        assert_eq!(state.orientation, Orientation::Horizontal);
    }

    #[test]
    fn state_stores_orientation() {
        let state = SeparatorState {
            orientation: Orientation::Vertical,
        };
        assert_eq!(state.orientation, Orientation::Vertical);
        assert_eq!(state.orientation.as_str(), "vertical");
    }
}
