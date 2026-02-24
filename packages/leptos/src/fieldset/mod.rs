//! Fieldset component.
//!
//! Ported from `@base-ui/react/fieldset`.
//! Groups a shared legend with related controls. Two parts:
//! `FieldsetRoot` and `FieldsetLegend`.
//!
//! ## What's ported
//! - `FieldsetRoot` — renders `<fieldset>` with `aria-labelledby` linked to legend
//! - `FieldsetLegend` — renders `<div>` that auto-registers its ID into root context
//! - `FieldsetContext` — shared context between root and legend
//! - `ClassProp<Store<FieldsetState>>` — static string or state-dependent class
//! - `StyleProp<Store<FieldsetState>>` — static string or state-dependent style
//! - `RenderProp<Store<FieldsetState>>` — custom render function to replace the default element
//!
//! ## Reactive state
//! `FieldsetState` uses `#[derive(Store)]` from `reactive_stores`. The `disabled` prop
//! accepts `Signal<bool>`, enabling reactive updates. When `disabled` changes,
//! an `Effect` updates the store, causing subscribed closures (class/style/data-attrs)
//! to re-execute with fine-grained reactivity.
//!
//! ## What's skipped (React-specific)
//! - `BaseUIComponentProps` type machinery
//!
//! ## Leptos usage
//! ```ignore
//! use base_ui_leptos::fieldset::{FieldsetRoot, FieldsetLegend};
//!
//! view! {
//!     <FieldsetRoot>
//!         <FieldsetLegend>"Personal Info"</FieldsetLegend>
//!         <input type="text" />
//!     </FieldsetRoot>
//! }
//! ```

use leptos::prelude::*;
use reactive_stores::Store;

use crate::utils::props::{ClassProp, RenderProp, RenderProps, StyleProp};
use crate::utils::use_id::use_id;

/// Shared context between `FieldsetRoot` and `FieldsetLegend`.
#[derive(Clone, Copy)]
struct FieldsetContext {
    /// Reactive store for class/style/data-attribute subscriptions.
    state: Store<FieldsetState>,
    /// Label ID for `aria-labelledby`.
    #[allow(dead_code)] // Read via signal in the root's view
    legend_id: ReadSignal<Option<String>>,
    /// Write handle for legend ID registration.
    set_legend_id: WriteSignal<Option<String>>,
}

/// Groups a shared legend with related controls.
/// Renders a `<fieldset>` element with `aria-labelledby` automatically
/// linked to a `FieldsetLegend` child.
///
/// Maps to `<Fieldset.Root>` in the React version.
/// The `disabled` prop accepts `Signal<bool>` for reactive updates.
#[component]
pub fn FieldsetRoot(
    /// Whether the fieldset is disabled.
    /// Accepts a plain `bool` or a signal for reactive updates.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<FieldsetState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<FieldsetState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<FieldsetState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<FieldsetState>>,
    /// Custom render function. When provided, replaces the default `<fieldset>` element.
    /// Children are not rendered when a custom render function is used.
    #[prop(optional, into)]
    render: RenderProp<Store<FieldsetState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Fieldset>>,
    children: Children,
) -> impl IntoView {
    let (legend_id, set_legend_id) = signal(None::<String>);

    let state_store = Store::new(FieldsetState {
        disabled: disabled.get_untracked(),
    });

    // Sync reactive disabled prop into store
    Effect::new(move |_| {
        state_store.disabled().set(disabled.get());
    });

    provide_context(FieldsetContext {
        state: state_store,
        legend_id,
        set_legend_id,
    });

    if render.is_custom() {
        return (move || {
            let is_disabled = state_store.disabled().get();
            let mut props = RenderProps {
                class: class.resolve(&state_store),
                style: style.resolve(&state_store),
                attrs: vec![],
            };
            if is_disabled {
                props = props.attr("data-disabled", "");
            }
            render.call(props, &state_store).unwrap()
        })
        .into_any();
    }

    let children_view = children();

    view! {
        <fieldset
            aria-labelledby=move || legend_id.get()
            class=move || class.resolve_option(&state_store)
            style=move || style.resolve_option(&state_store)
            disabled=move || state_store.disabled().get()
            data-disabled=move || {
                if state_store.disabled().get() { Some("") } else { None }
            }
            node_ref=node_ref.unwrap_or_default()
        >
            {children_view}
        </fieldset>
    }
    .into_any()
}

/// An accessible label that is automatically associated with the fieldset.
/// Renders a `<div>` element. Its ID is auto-generated and registered into
/// the parent `FieldsetRoot` context so the fieldset gets `aria-labelledby`.
///
/// Maps to `<Fieldset.Legend>` in the React version.
#[component]
pub fn FieldsetLegend(
    /// Override the auto-generated ID.
    #[prop(optional, into)]
    id: Option<String>,
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<FieldsetState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<FieldsetState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<FieldsetState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<FieldsetState>>,
    /// Custom render function. When provided, replaces the default `<div>` element.
    /// Children are not rendered when a custom render function is used.
    #[prop(optional, into)]
    render: RenderProp<Store<FieldsetState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Div>>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<FieldsetContext>();
    let store = ctx.state;
    let generated_id = use_id(id.as_deref(), None);
    let id_value = id.unwrap_or(generated_id);

    // Register legend ID into root context
    ctx.set_legend_id.set(Some(id_value.clone()));

    // Cleanup on unmount
    let set_legend_id = ctx.set_legend_id;
    on_cleanup(move || {
        set_legend_id.set(None);
    });

    if render.is_custom() {
        let id_for_render = id_value.clone();
        return (move || {
            let is_disabled = store.disabled().get();
            let mut props = RenderProps {
                class: class.resolve(&store),
                style: style.resolve(&store),
                attrs: vec![],
            }
            .attr("id", id_for_render.clone());
            if is_disabled {
                props = props.attr("data-disabled", "");
            }
            render.call(props, &store).unwrap()
        })
        .into_any();
    }

    let children_view = children();

    view! {
        <div
            id=id_value
            class=move || class.resolve_option(&store)
            style=move || style.resolve_option(&store)
            data-disabled=move || {
                if store.disabled().get() { Some("") } else { None }
            }
            node_ref=node_ref.unwrap_or_default()
        >
            {children_view}
        </div>
    }
    .into_any()
}

/// State of the Fieldset component parts.
///
/// Maps to `FieldsetRoot.State` / `FieldsetLegend.State` in the React version.
/// Uses `#[derive(Store)]` for reactive field-level access via `Store<FieldsetState>`.
/// The generated `FieldsetStateStoreFields` trait provides `.disabled()` for fine-grained
/// subscriptions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Store)]
pub struct FieldsetState {
    pub disabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from FieldsetRoot.test.tsx / FieldsetLegend.test.tsx:
    // ✓ FieldsetState stores disabled flag
    // ✗ renders <fieldset> with aria-labelledby (requires browser DOM)
    // ✗ legend auto-registers ID (requires browser DOM)
    // ✗ custom id on legend (requires browser DOM)
    // ✗ describeConformance (React-specific test infrastructure)

    #[test]
    fn fieldset_state_default() {
        let state = FieldsetState { disabled: false };
        assert!(!state.disabled);
    }

    #[test]
    fn fieldset_state_disabled() {
        let state = FieldsetState { disabled: true };
        assert!(state.disabled);
    }
}
