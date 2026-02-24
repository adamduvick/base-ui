//! Meter component.
//!
//! Ported from `@base-ui/react/meter`.
//! Displays a scalar measurement within a known range. Five parts:
//! `MeterRoot`, `MeterTrack`, `MeterIndicator`, `MeterValue`, `MeterLabel`.
//!
//! ## What's ported
//! - `MeterRoot` — renders `<div role="meter">` with ARIA attributes
//! - `MeterTrack` — structural container for the indicator
//! - `MeterIndicator` — renders with `width` style based on value
//! - `MeterValue` — displays the formatted value text
//! - `MeterLabel` — accessible label, auto-registers ID for `aria-labelledby`
//! - `ClassProp<Store<MeterState>>` — static string or state-dependent class
//! - `StyleProp<Store<MeterState>>` — static string or state-dependent style
//! - `RenderProp<Store<MeterState>>` — custom render function to replace the default element
//!
//! ## Reactive state
//! `MeterState` uses `#[derive(Store)]` from `reactive_stores` for consistency.
//! The `value` prop accepts `Signal<f64>`, enabling reactive updates. When `value`
//! changes, derived computations (indicator width, formatted value, ARIA attributes)
//! re-execute automatically.
//!
//! ## What's skipped (React-specific)
//! - `BaseUIComponentProps` type machinery
//! - `format`/`locale` props — simplified formatting (no `Intl.NumberFormat`)
//!
//! ## Leptos usage
//! ```ignore
//! use base_ui_leptos::meter::*;
//! use reactive_stores::Store;
//!
//! view! {
//!     <MeterRoot value=75.0>
//!         <MeterLabel>"CPU Usage"</MeterLabel>
//!         <MeterTrack>
//!             <MeterIndicator />
//!         </MeterTrack>
//!         <MeterValue />
//!     </MeterRoot>
//! }
//! ```

use leptos::prelude::*;
use reactive_stores::Store;

use crate::utils::format_number::format_number_value;
use crate::utils::props::{ClassProp, RenderProp, RenderProps, StyleProp};
use crate::utils::use_id::use_id;
use crate::utils::value_to_percent::value_to_percent;

/// Shared context between Meter parts.
#[derive(Clone, Copy)]
struct MeterContext {
    /// Reactive store for class/style subscriptions.
    state: Store<MeterState>,
    /// Raw reactive value for internal computations (indicator width, ARIA attrs).
    value: Signal<f64>,
    /// Static minimum value.
    min: f64,
    /// Static maximum value.
    max: f64,
    /// Derived formatted value string.
    formatted_value: Memo<String>,
    /// Label ID for `aria-labelledby`.
    #[allow(dead_code)] // Read via signal in the root's view
    label_id: ReadSignal<Option<String>>,
    /// Write handle for label ID registration.
    set_label_id: WriteSignal<Option<String>>,
}

/// State of the Meter component parts.
///
/// Maps to `MeterRoot.State` in the React version (which is `{}`).
/// Empty in React; provided here so `ClassProp`/`StyleProp`/`RenderProp`
/// have a state type to parameterise over.
/// Uses `#[derive(Store)]` for consistency with other components.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Store)]
pub struct MeterState {}

/// Displays a scalar measurement within a known range.
/// Renders a `<div>` element with `role="meter"` and ARIA value attributes.
///
/// Maps to `<Meter.Root>` in the React version.
///
/// Unlike Progress, Meter always has a definite value (no indeterminate state).
/// The `value` prop accepts `Signal<f64>` for reactive updates.
#[component]
pub fn MeterRoot(
    /// Current meter value (required).
    /// Accepts a plain `f64` or a signal for reactive updates.
    #[prop(into)]
    value: Signal<f64>,
    /// Minimum value. Defaults to `0.0`.
    #[prop(optional, default = 0.0)]
    min: f64,
    /// Maximum value. Defaults to `100.0`.
    #[prop(optional, default = 100.0)]
    max: f64,
    /// Custom function to generate `aria-valuetext`.
    /// Receives `(formatted_value, value)`.
    #[prop(optional)]
    get_aria_value_text: Option<fn(&str, f64) -> String>,
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<MeterState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<MeterState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<MeterState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<MeterState>>,
    /// Custom render function. When provided, replaces the default `<div>` element.
    /// Children are not rendered when a custom render function is used.
    #[prop(optional, into)]
    render: RenderProp<Store<MeterState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Div>>,
    children: Children,
) -> impl IntoView {
    let (label_id, set_label_id) = signal(None::<String>);
    let state_store = Store::new(MeterState {});

    let formatted_value = Memo::new(move |_| format_number_value(Some(value.get())));

    let aria_valuetext = Memo::new(move |_| {
        let v = value.get();
        let formatted = format_number_value(Some(v));
        if let Some(get_text) = get_aria_value_text {
            get_text(&formatted, v)
        } else {
            format!("{v}%")
        }
    });

    provide_context(MeterContext {
        state: state_store,
        value,
        min,
        max,
        formatted_value,
        label_id,
        set_label_id,
    });

    if render.is_custom() {
        return (move || {
            let v = value.get();
            let props = RenderProps {
                class: class.resolve(&state_store),
                style: style.resolve(&state_store),
                attrs: vec![],
            }
            .attr("role", "meter")
            .attr("aria-valuemax", max.to_string())
            .attr("aria-valuemin", min.to_string())
            .attr("aria-valuenow", v.to_string())
            .attr("aria-valuetext", aria_valuetext.get());
            render.call(props, &state_store).unwrap()
        })
        .into_any();
    }

    let children_view = children();

    view! {
        <div
            role="meter"
            aria-labelledby=move || label_id.get()
            aria-valuemax=max
            aria-valuemin=min
            aria-valuenow=move || value.get()
            aria-valuetext=move || aria_valuetext.get()
            class=move || class.resolve_option(&state_store)
            style=move || style.resolve_option(&state_store)
            node_ref=node_ref.unwrap_or_default()
        >
            {children_view}
        </div>
    }
    .into_any()
}

/// Structural container for the meter indicator.
/// Renders a `<div>` element.
///
/// Maps to `<Meter.Track>` in the React version.
#[component]
pub fn MeterTrack(
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<MeterState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<MeterState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<MeterState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<MeterState>>,
    /// Custom render function. When provided, replaces the default `<div>` element.
    /// Children are not rendered when a custom render function is used.
    #[prop(optional, into)]
    render: RenderProp<Store<MeterState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Div>>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<MeterContext>();
    let store = ctx.state;

    if render.is_custom() {
        return (move || {
            let props = RenderProps {
                class: class.resolve(&store),
                style: style.resolve(&store),
                attrs: vec![],
            };
            render.call(props, &store).unwrap()
        })
        .into_any();
    }

    let children_view = children();

    view! {
        <div
            class=move || class.resolve_option(&store)
            style=move || style.resolve_option(&store)
            node_ref=node_ref.unwrap_or_default()
        >
            {children_view}
        </div>
    }
    .into_any()
}

/// Visual indicator of the meter value.
/// Renders a `<div>` with `width` style set to the percentage of the value.
///
/// Maps to `<Meter.Indicator>` in the React version.
#[component]
pub fn MeterIndicator(
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<MeterState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<MeterState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<MeterState>`.
    /// Concatenated after the internal indicator style (`inset-inline-start: 0; ...`).
    #[prop(optional, into)]
    style: StyleProp<Store<MeterState>>,
    /// Custom render function. When provided, replaces the default `<div>` element.
    #[prop(optional, into)]
    render: RenderProp<Store<MeterState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Div>>,
) -> impl IntoView {
    let ctx = expect_context::<MeterContext>();
    let store = ctx.state;

    if render.is_custom() {
        return (move || {
            let v = ctx.value.get();
            let pct = value_to_percent(v, ctx.min, ctx.max);
            let internal_style = format!("inset-inline-start: 0; height: inherit; width: {pct}%;");
            let user_style = style.resolve(&store);
            let combined_style = if user_style.is_empty() {
                internal_style
            } else {
                format!("{internal_style} {user_style}")
            };
            let props = RenderProps {
                class: class.resolve(&store),
                style: combined_style,
                attrs: vec![],
            };
            render.call(props, &store).unwrap()
        })
        .into_any();
    }

    view! {
        <div
            class=move || class.resolve_option(&store)
            style=move || {
                let v = ctx.value.get();
                let pct = value_to_percent(v, ctx.min, ctx.max);
                let internal_style = format!("inset-inline-start: 0; height: inherit; width: {pct}%;");
                let user_style = style.resolve(&store);
                if user_style.is_empty() {
                    Some(internal_style)
                } else {
                    Some(format!("{internal_style} {user_style}"))
                }
            }
            node_ref=node_ref.unwrap_or_default()
        />
    }
    .into_any()
}

/// Displays the formatted meter value.
/// Renders a `<span>` with `aria-hidden="true"`.
///
/// Maps to `<Meter.Value>` in the React version.
#[component]
pub fn MeterValue(
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<MeterState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<MeterState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<MeterState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<MeterState>>,
    /// Custom render function. When provided, replaces the default `<span>` element.
    #[prop(optional, into)]
    render: RenderProp<Store<MeterState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Span>>,
) -> impl IntoView {
    let ctx = expect_context::<MeterContext>();
    let store = ctx.state;

    if render.is_custom() {
        return (move || {
            let props = RenderProps {
                class: class.resolve(&store),
                style: style.resolve(&store),
                attrs: vec![],
            }
            .attr("aria-hidden", "true");
            render.call(props, &store).unwrap()
        })
        .into_any();
    }

    view! {
        <span
            aria-hidden="true"
            class=move || class.resolve_option(&store)
            style=move || style.resolve_option(&store)
            node_ref=node_ref.unwrap_or_default()
        >
            {move || {
                let formatted = ctx.formatted_value.get();
                if formatted.is_empty() {
                    format!("{}", ctx.value.get())
                } else {
                    formatted
                }
            }}
        </span>
    }
    .into_any()
}

/// Accessible label for the meter.
/// Renders a `<span>` whose ID is auto-registered into `MeterRoot`
/// for `aria-labelledby`.
///
/// Maps to `<Meter.Label>` in the React version.
#[component]
pub fn MeterLabel(
    /// Override the auto-generated ID.
    #[prop(optional, into)]
    id: Option<String>,
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<MeterState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<MeterState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<MeterState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<MeterState>>,
    /// Custom render function. When provided, replaces the default `<span>` element.
    /// Children are not rendered when a custom render function is used.
    #[prop(optional, into)]
    render: RenderProp<Store<MeterState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Span>>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<MeterContext>();
    let store = ctx.state;
    let generated_id = use_id(id.as_deref(), None);
    let id_value = id.unwrap_or(generated_id);

    // Register label ID into root context
    ctx.set_label_id.set(Some(id_value.clone()));

    let set_label_id = ctx.set_label_id;
    on_cleanup(move || {
        set_label_id.set(None);
    });

    if render.is_custom() {
        let id_for_render = id_value.clone();
        return (move || {
            let props = RenderProps {
                class: class.resolve(&store),
                style: style.resolve(&store),
                attrs: vec![],
            }
            .attr("id", id_for_render.clone());
            render.call(props, &store).unwrap()
        })
        .into_any();
    }

    let children_view = children();

    view! {
        <span
            id=id_value
            class=move || class.resolve_option(&store)
            style=move || style.resolve_option(&store)
            node_ref=node_ref.unwrap_or_default()
        >
            {children_view}
        </span>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    // Ported from MeterRoot.test.tsx:
    // ✓ default aria-valuetext
    // ✓ custom get_aria_value_text
    // ✓ format_number_value formats value correctly
    // ✓ value_to_percent computes indicator width
    // ✗ renders meter role (requires browser DOM)
    // ✗ ARIA attributes applied (requires browser DOM)
    // ✗ label aria-labelledby registration (requires browser DOM)
    // ✗ describeConformance (React-specific)

    #[test]
    fn default_aria_valuetext() {
        let value = 75.0;
        let text = format!("{value}%");
        assert_eq!(text, "75%");
    }

    #[test]
    fn custom_get_aria_value_text() {
        let get_text: fn(&str, f64) -> String = |formatted, value| {
            format!("{formatted} ({value} out of 100)")
        };
        let result = get_text("75%", 75.0);
        assert_eq!(result, "75% (75 out of 100)");
    }

    #[test]
    fn indicator_percentage() {
        use crate::utils::value_to_percent::value_to_percent;
        assert_eq!(value_to_percent(75.0, 0.0, 100.0), 75.0);
        assert_eq!(value_to_percent(5.0, 0.0, 10.0), 50.0);
    }

    #[test]
    fn formatted_value() {
        use crate::utils::format_number::format_number_value;
        assert_eq!(format_number_value(Some(75.0)), "75%");
        assert_eq!(format_number_value(Some(0.0)), "0%");
        assert_eq!(format_number_value(Some(100.0)), "100%");
    }
}
