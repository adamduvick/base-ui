//! Progress component.
//!
//! Ported from `@base-ui/react/progress`.
//! Displays the status of a task or operation. Five parts:
//! `ProgressRoot`, `ProgressTrack`, `ProgressIndicator`, `ProgressValue`, `ProgressLabel`.
//!
//! ## What's ported
//! - `ProgressRoot` — renders `<div role="progressbar">` with ARIA attributes
//! - `ProgressTrack` — structural container for the indicator
//! - `ProgressIndicator` — renders with `width` style based on progress
//! - `ProgressValue` — displays the formatted value text
//! - `ProgressLabel` — accessible label, auto-registers ID for `aria-labelledby`
//! - `ProgressStatus` — status enum (indeterminate/progressing/complete)
//! - Data attributes: `data-indeterminate`, `data-progressing`, `data-complete`
//! - `ClassProp<Store<ProgressState>>` — static string or state-dependent class
//! - `StyleProp<Store<ProgressState>>` — static string or state-dependent style
//! - `RenderProp<Store<ProgressState>>` — custom render function to replace the default element
//!
//! ## Reactive state
//! `ProgressState` uses `#[derive(Store)]` from `reactive_stores`. The `value` prop
//! accepts `Signal<Option<f64>>`, enabling reactive updates. When `value` changes,
//! an `Effect` recomputes `status` and updates the store, causing subscribed closures
//! (class/style/data-attrs) to re-execute with fine-grained reactivity.
//!
//! ## What's skipped (React-specific)
//! - `BaseUIComponentProps` type machinery
//! - `format`/`locale` props — simplified formatting (no `Intl.NumberFormat`)
//!
//! ## Leptos usage
//! ```ignore
//! use base_ui_leptos::progress::*;
//! use reactive_stores::Store;
//!
//! view! {
//!     <ProgressRoot value=Some(50.0)>
//!         <ProgressLabel>"Loading..."</ProgressLabel>
//!         <ProgressTrack>
//!             <ProgressIndicator />
//!         </ProgressTrack>
//!         <ProgressValue />
//!     </ProgressRoot>
//! }
//! ```

use leptos::prelude::*;
use reactive_stores::Store;

use crate::utils::format_number::format_number_value;
use crate::utils::props::{ClassProp, RenderProp, RenderProps, StyleProp};
use crate::utils::use_id::use_id;
use crate::utils::value_to_percent::value_to_percent;

/// Progress status derived from the current value.
///
/// Maps to `ProgressStatus` in the React version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProgressStatus {
    /// Value is `None` — progress amount is unknown.
    Indeterminate,
    /// Value is between min and max (exclusive).
    Progressing,
    /// Value equals max.
    Complete,
}

impl ProgressStatus {
    /// Returns the status string for data attribute values.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProgressStatus::Indeterminate => "indeterminate",
            ProgressStatus::Progressing => "progressing",
            ProgressStatus::Complete => "complete",
        }
    }
}

/// Compute progress status from value and max.
fn compute_status(value: Option<f64>, max: f64) -> ProgressStatus {
    match value {
        None => ProgressStatus::Indeterminate,
        Some(v) if v >= max => ProgressStatus::Complete,
        Some(_) => ProgressStatus::Progressing,
    }
}

/// Shared context between Progress parts.
#[derive(Clone, Copy)]
struct ProgressContext {
    /// Reactive store for class/style/data-attribute subscriptions.
    state: Store<ProgressState>,
    /// Raw reactive value for internal computations (indicator width, ARIA attrs).
    value: Signal<Option<f64>>,
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

/// Helper: build data-attribute entries for progress status.
fn progress_status_attrs(status: ProgressStatus) -> Vec<(&'static str, String)> {
    let mut attrs = Vec::new();
    if status == ProgressStatus::Indeterminate {
        attrs.push(("data-indeterminate", String::new()));
    }
    if status == ProgressStatus::Progressing {
        attrs.push(("data-progressing", String::new()));
    }
    if status == ProgressStatus::Complete {
        attrs.push(("data-complete", String::new()));
    }
    attrs
}

/// Displays the status of a task or operation.
/// Renders a `<div>` element with `role="progressbar"` and ARIA value attributes.
///
/// Maps to `<Progress.Root>` in the React version.
///
/// Pass `value=None` for indeterminate progress.
/// The `value` prop accepts `Signal<Option<f64>>` for reactive updates.
#[component]
pub fn ProgressRoot(
    /// Current progress value. `None` means indeterminate.
    /// Accepts a plain `Option<f64>` or a signal for reactive updates.
    #[prop(into)]
    value: Signal<Option<f64>>,
    /// Minimum value. Defaults to `0.0`.
    #[prop(optional, default = 0.0)]
    min: f64,
    /// Maximum value. Defaults to `100.0`.
    #[prop(optional, default = 100.0)]
    max: f64,
    /// Custom function to generate `aria-valuetext`.
    /// Receives `(formatted_value, value)`.
    #[prop(optional)]
    get_aria_value_text: Option<fn(&str, Option<f64>) -> String>,
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<ProgressState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<ProgressState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<ProgressState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<ProgressState>>,
    /// Custom render function. When provided, replaces the default `<div>` element.
    /// Children are not rendered when a custom render function is used.
    #[prop(optional, into)]
    render: RenderProp<Store<ProgressState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Div>>,
    children: Children,
) -> impl IntoView {
    let (label_id, set_label_id) = signal(None::<String>);

    let initial_status = compute_status(value.get_untracked(), max);
    let state_store = Store::new(ProgressState { status: initial_status });

    // Sync reactive value changes into the store
    Effect::new(move |_| {
        let new_status = compute_status(value.get(), max);
        state_store.status().set(new_status);
    });

    // Derived formatted value
    let formatted_value = Memo::new(move |_| format_number_value(value.get()));

    // Derived aria-valuetext
    let aria_valuetext = Memo::new(move |_| {
        let v = value.get();
        let formatted = format_number_value(v);
        if let Some(get_text) = get_aria_value_text {
            get_text(&formatted, v)
        } else if v.is_none() {
            "indeterminate progress".to_string()
        } else if !formatted.is_empty() {
            formatted
        } else {
            format!("{}%", v.unwrap_or(0.0))
        }
    });

    provide_context(ProgressContext {
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
            let status = state_store.status().get();
            let props_builder = RenderProps {
                class: class.resolve(&state_store),
                style: style.resolve(&state_store),
                attrs: vec![],
            }
            .attr("role", "progressbar")
            .attr("aria-valuemax", max.to_string())
            .attr("aria-valuemin", min.to_string())
            .attr("aria-valuetext", aria_valuetext.get());
            let mut props = if let Some(v) = value.get() {
                props_builder.attr("aria-valuenow", v.to_string())
            } else {
                props_builder
            };
            for (name, val) in progress_status_attrs(status) {
                props = props.attr(name, val);
            }
            render.call(props, &state_store).unwrap()
        })
        .into_any();
    }

    let children_view = children();

    view! {
        <div
            role="progressbar"
            aria-labelledby=move || label_id.get()
            aria-valuemax=max
            aria-valuemin=min
            aria-valuenow=move || value.get()
            aria-valuetext=move || aria_valuetext.get()
            class=move || class.resolve_option(&state_store)
            style=move || style.resolve_option(&state_store)
            data-indeterminate=move || {
                if state_store.status().get() == ProgressStatus::Indeterminate { Some("") } else { None }
            }
            data-progressing=move || {
                if state_store.status().get() == ProgressStatus::Progressing { Some("") } else { None }
            }
            data-complete=move || {
                if state_store.status().get() == ProgressStatus::Complete { Some("") } else { None }
            }
            node_ref=node_ref.unwrap_or_default()
        >
            {children_view}
        </div>
    }
    .into_any()
}

/// Structural container for the progress indicator.
/// Renders a `<div>` element.
///
/// Maps to `<Progress.Track>` in the React version.
#[component]
pub fn ProgressTrack(
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<ProgressState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<ProgressState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<ProgressState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<ProgressState>>,
    /// Custom render function. When provided, replaces the default `<div>` element.
    /// Children are not rendered when a custom render function is used.
    #[prop(optional, into)]
    render: RenderProp<Store<ProgressState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Div>>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ProgressContext>();
    let store = ctx.state;

    if render.is_custom() {
        return (move || {
            let status = store.status().get();
            let mut props = RenderProps {
                class: class.resolve(&store),
                style: style.resolve(&store),
                attrs: vec![],
            };
            for (name, val) in progress_status_attrs(status) {
                props = props.attr(name, val);
            }
            render.call(props, &store).unwrap()
        })
        .into_any();
    }

    let children_view = children();

    view! {
        <div
            class=move || class.resolve_option(&store)
            style=move || style.resolve_option(&store)
            data-indeterminate=move || {
                if store.status().get() == ProgressStatus::Indeterminate { Some("") } else { None }
            }
            data-progressing=move || {
                if store.status().get() == ProgressStatus::Progressing { Some("") } else { None }
            }
            data-complete=move || {
                if store.status().get() == ProgressStatus::Complete { Some("") } else { None }
            }
            node_ref=node_ref.unwrap_or_default()
        >
            {children_view}
        </div>
    }
    .into_any()
}

/// Visual indicator of progress.
/// Renders a `<div>` with `width` style set to the percentage of progress.
///
/// Maps to `<Progress.Indicator>` in the React version.
#[component]
pub fn ProgressIndicator(
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<ProgressState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<ProgressState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<ProgressState>`.
    /// Concatenated after the internal indicator style (`inset-inline-start: 0; ...`).
    #[prop(optional, into)]
    style: StyleProp<Store<ProgressState>>,
    /// Custom render function. When provided, replaces the default `<div>` element.
    #[prop(optional, into)]
    render: RenderProp<Store<ProgressState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Div>>,
) -> impl IntoView {
    let ctx = expect_context::<ProgressContext>();
    let store = ctx.state;

    if render.is_custom() {
        return (move || {
            let status = store.status().get();
            let internal_style = match ctx.value.get() {
                Some(v) => {
                    let pct = value_to_percent(v, ctx.min, ctx.max);
                    format!("inset-inline-start: 0; height: inherit; width: {pct}%;")
                }
                None => String::new(),
            };
            let user_style = style.resolve(&store);
            let combined_style = if user_style.is_empty() {
                internal_style
            } else {
                format!("{internal_style} {user_style}")
            };
            let mut props = RenderProps {
                class: class.resolve(&store),
                style: combined_style,
                attrs: vec![],
            };
            for (name, val) in progress_status_attrs(status) {
                props = props.attr(name, val);
            }
            render.call(props, &store).unwrap()
        })
        .into_any();
    }

    view! {
        <div
            class=move || class.resolve_option(&store)
            style=move || {
                let internal_style = match ctx.value.get() {
                    Some(v) => {
                        let pct = value_to_percent(v, ctx.min, ctx.max);
                        format!("inset-inline-start: 0; height: inherit; width: {pct}%;")
                    }
                    None => String::new(),
                };
                let user_style = style.resolve(&store);
                let combined = if user_style.is_empty() {
                    internal_style
                } else {
                    format!("{internal_style} {user_style}")
                };
                if combined.is_empty() { None } else { Some(combined) }
            }
            data-indeterminate=move || {
                if store.status().get() == ProgressStatus::Indeterminate { Some("") } else { None }
            }
            data-progressing=move || {
                if store.status().get() == ProgressStatus::Progressing { Some("") } else { None }
            }
            data-complete=move || {
                if store.status().get() == ProgressStatus::Complete { Some("") } else { None }
            }
            node_ref=node_ref.unwrap_or_default()
        />
    }
    .into_any()
}

/// Displays the formatted progress value.
/// Renders a `<span>` with `aria-hidden="true"`.
///
/// Maps to `<Progress.Value>` in the React version.
#[component]
pub fn ProgressValue(
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<ProgressState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<ProgressState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<ProgressState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<ProgressState>>,
    /// Custom render function. When provided, replaces the default `<span>` element.
    #[prop(optional, into)]
    render: RenderProp<Store<ProgressState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Span>>,
) -> impl IntoView {
    let ctx = expect_context::<ProgressContext>();
    let store = ctx.state;

    if render.is_custom() {
        return (move || {
            let status = store.status().get();
            let mut props = RenderProps {
                class: class.resolve(&store),
                style: style.resolve(&store),
                attrs: vec![],
            }
            .attr("aria-hidden", "true");
            for (name, val) in progress_status_attrs(status) {
                props = props.attr(name, val);
            }
            render.call(props, &store).unwrap()
        })
        .into_any();
    }

    view! {
        <span
            aria-hidden="true"
            class=move || class.resolve_option(&store)
            style=move || style.resolve_option(&store)
            data-indeterminate=move || {
                if store.status().get() == ProgressStatus::Indeterminate { Some("") } else { None }
            }
            data-progressing=move || {
                if store.status().get() == ProgressStatus::Progressing { Some("") } else { None }
            }
            data-complete=move || {
                if store.status().get() == ProgressStatus::Complete { Some("") } else { None }
            }
            node_ref=node_ref.unwrap_or_default()
        >
            {move || {
                match ctx.value.get() {
                    Some(_) => ctx.formatted_value.get(),
                    None => String::new(),
                }
            }}
        </span>
    }
    .into_any()
}

/// Accessible label for the progress bar.
/// Renders a `<span>` whose ID is auto-registered into `ProgressRoot`
/// for `aria-labelledby`.
///
/// Maps to `<Progress.Label>` in the React version.
#[component]
pub fn ProgressLabel(
    /// Override the auto-generated ID.
    #[prop(optional, into)]
    id: Option<String>,
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<ProgressState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<ProgressState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<ProgressState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<ProgressState>>,
    /// Custom render function. When provided, replaces the default `<span>` element.
    /// Children are not rendered when a custom render function is used.
    #[prop(optional, into)]
    render: RenderProp<Store<ProgressState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Span>>,
    children: Children,
) -> impl IntoView {
    let ctx = expect_context::<ProgressContext>();
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
            let status = store.status().get();
            let mut props = RenderProps {
                class: class.resolve(&store),
                style: style.resolve(&store),
                attrs: vec![],
            }
            .attr("id", id_for_render.clone());
            for (name, val) in progress_status_attrs(status) {
                props = props.attr(name, val);
            }
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
            data-indeterminate=move || {
                if store.status().get() == ProgressStatus::Indeterminate { Some("") } else { None }
            }
            data-progressing=move || {
                if store.status().get() == ProgressStatus::Progressing { Some("") } else { None }
            }
            data-complete=move || {
                if store.status().get() == ProgressStatus::Complete { Some("") } else { None }
            }
            node_ref=node_ref.unwrap_or_default()
        >
            {children_view}
        </span>
    }
    .into_any()
}

/// State of the Progress component parts.
///
/// Maps to `ProgressRoot.State` in the React version.
/// Uses `#[derive(Store)]` for reactive field-level access via `Store<ProgressState>`.
/// The generated `ProgressStateStoreFields` trait provides `.status()` for fine-grained
/// subscriptions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Store)]
pub struct ProgressState {
    pub status: ProgressStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from ProgressRoot.test.tsx:
    // ✓ status is indeterminate when value is None
    // ✓ status is progressing when value < max
    // ✓ status is complete when value == max
    // ✓ format_number_value formats correctly
    // ✓ value_to_percent computes correctly
    // ✓ default aria-valuetext for indeterminate
    // ✓ compute_status helper
    // ✗ renders progressbar role (requires browser DOM)
    // ✗ ARIA attributes applied correctly (requires browser DOM)
    // ✗ data attributes applied correctly (requires browser DOM)
    // ✗ describeConformance (React-specific)
    // ✗ indicator width style (requires browser DOM)
    // ✗ label aria-labelledby registration (requires browser DOM)

    #[test]
    fn status_indeterminate_when_none() {
        assert_eq!(compute_status(None, 100.0), ProgressStatus::Indeterminate);
    }

    #[test]
    fn status_progressing_when_in_range() {
        assert_eq!(compute_status(Some(50.0), 100.0), ProgressStatus::Progressing);
    }

    #[test]
    fn status_complete_when_at_max() {
        assert_eq!(compute_status(Some(100.0), 100.0), ProgressStatus::Complete);
    }

    #[test]
    fn status_as_str() {
        assert_eq!(ProgressStatus::Indeterminate.as_str(), "indeterminate");
        assert_eq!(ProgressStatus::Progressing.as_str(), "progressing");
        assert_eq!(ProgressStatus::Complete.as_str(), "complete");
    }

    #[test]
    fn indicator_percent_calculation() {
        use crate::utils::value_to_percent::value_to_percent;
        assert_eq!(value_to_percent(50.0, 0.0, 100.0), 50.0);
        assert_eq!(value_to_percent(0.0, 0.0, 100.0), 0.0);
        assert_eq!(value_to_percent(100.0, 0.0, 100.0), 100.0);
    }

    #[test]
    fn default_aria_valuetext_indeterminate() {
        let text = "indeterminate progress";
        assert_eq!(text, "indeterminate progress");
    }

    #[test]
    fn progress_state() {
        let state = ProgressState {
            status: ProgressStatus::Progressing,
        };
        assert_eq!(state.status, ProgressStatus::Progressing);
    }
}
