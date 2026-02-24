//! Prop types for headless component customization.
//!
//! Provides `ClassProp<S>`, `StyleProp<S>`, and `RenderProp<S>` — the Leptos
//! equivalents of Base UI React's `className`, `style`, and `render` props.
//!
//! Each type uses `From` impls so that consumers can pass a plain string
//! **or** a closure that receives the component's state:
//!
//! ```ignore
//! // Static class
//! <Separator class="my-separator" />
//!
//! // State-dependent class
//! <Separator class=|state: &SeparatorState| {
//!     if state.orientation == Orientation::Vertical { "v-sep" } else { "h-sep" }
//! }.to_string() />
//! ```

use leptos::prelude::*;

// ---------------------------------------------------------------------------
// ClassProp<S>
// ---------------------------------------------------------------------------

/// A CSS class prop that can be a static string or a closure over component state.
///
/// Maps to the React `className` prop which accepts `string | (state) => string`.
pub struct ClassProp<S: 'static>(Box<dyn Fn(&S) -> String + Send + Sync>);

impl<S: 'static> ClassProp<S> {
    /// Resolve the class string for the given state.
    pub fn resolve(&self, state: &S) -> String {
        (self.0)(state)
    }

    /// Like `resolve`, but returns `None` when the result is empty.
    /// Useful for the `class` attribute so an empty string isn't rendered.
    pub fn resolve_option(&self, state: &S) -> Option<String> {
        let s = self.resolve(state);
        if s.is_empty() { None } else { Some(s) }
    }
}

impl<S: 'static> Default for ClassProp<S> {
    fn default() -> Self {
        ClassProp(Box::new(|_| String::new()))
    }
}

impl<S: 'static> From<&str> for ClassProp<S> {
    fn from(s: &str) -> Self {
        let owned = s.to_owned();
        ClassProp(Box::new(move |_| owned.clone()))
    }
}

impl<S: 'static> From<String> for ClassProp<S> {
    fn from(s: String) -> Self {
        ClassProp(Box::new(move |_| s.clone()))
    }
}

impl<S: 'static, F: Fn(&S) -> String + Send + Sync + 'static> From<F> for ClassProp<S> {
    fn from(f: F) -> Self {
        ClassProp(Box::new(f))
    }
}

// ---------------------------------------------------------------------------
// StyleProp<S>
// ---------------------------------------------------------------------------

/// An inline style prop that can be a static string or a closure over component state.
///
/// Maps to the React `style` prop which accepts `object | (state) => object`.
/// In Leptos we use CSS strings rather than style objects.
pub struct StyleProp<S: 'static>(Box<dyn Fn(&S) -> String + Send + Sync>);

impl<S: 'static> StyleProp<S> {
    /// Resolve the style string for the given state.
    pub fn resolve(&self, state: &S) -> String {
        (self.0)(state)
    }

    /// Like `resolve`, but returns `None` when the result is empty.
    pub fn resolve_option(&self, state: &S) -> Option<String> {
        let s = self.resolve(state);
        if s.is_empty() { None } else { Some(s) }
    }
}

impl<S: 'static> Default for StyleProp<S> {
    fn default() -> Self {
        StyleProp(Box::new(|_| String::new()))
    }
}

impl<S: 'static> From<&str> for StyleProp<S> {
    fn from(s: &str) -> Self {
        let owned = s.to_owned();
        StyleProp(Box::new(move |_| owned.clone()))
    }
}

impl<S: 'static> From<String> for StyleProp<S> {
    fn from(s: String) -> Self {
        StyleProp(Box::new(move |_| s.clone()))
    }
}

impl<S: 'static, F: Fn(&S) -> String + Send + Sync + 'static> From<F> for StyleProp<S> {
    fn from(f: F) -> Self {
        StyleProp(Box::new(f))
    }
}

// ---------------------------------------------------------------------------
// RenderProps
// ---------------------------------------------------------------------------

/// Carries resolved attributes that a custom render function should apply to
/// the element it returns.
///
/// This is the Leptos equivalent of the props object passed to React's
/// `render` callback.
#[derive(Clone, Debug, Default)]
pub struct RenderProps {
    /// Resolved CSS class string.
    pub class: String,
    /// Resolved inline style string.
    pub style: String,
    /// Additional attributes to spread onto the element (`(name, value)` pairs).
    pub attrs: Vec<(&'static str, String)>,
}

impl RenderProps {
    /// Builder-style: add a single attribute.
    pub fn attr(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.attrs.push((name, value.into()));
        self
    }
}

// ---------------------------------------------------------------------------
// RenderProp<S>
// ---------------------------------------------------------------------------

/// A custom render prop that replaces the default element entirely.
///
/// Maps to the React `render` prop. When provided, the component delegates
/// rendering to this closure instead of emitting its default HTML element.
pub struct RenderProp<S: 'static>(
    Option<Box<dyn Fn(RenderProps, &S) -> AnyView + Send + Sync>>,
);

impl<S: 'static> RenderProp<S> {
    /// Returns `true` when a custom render function was provided.
    pub fn is_custom(&self) -> bool {
        self.0.is_some()
    }

    /// Call the render function. Returns `None` when using the default element.
    pub fn call(&self, props: RenderProps, state: &S) -> Option<AnyView> {
        self.0.as_ref().map(|f| f(props, state))
    }
}

impl<S: 'static> Default for RenderProp<S> {
    fn default() -> Self {
        RenderProp(None)
    }
}

impl<S: 'static, F: Fn(RenderProps, &S) -> AnyView + Send + Sync + 'static> From<F>
    for RenderProp<S>
{
    fn from(f: F) -> Self {
        RenderProp(Some(Box::new(f)))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(dead_code)] // TestWidget component is compiled but not called in native tests
mod tests {
    use super::*;

    // -- Test component -----------------------------------------------------
    //
    // Defines a component that uses all three prop types in its API.
    // The `#[component]` definition proves the types compile with Leptos's
    // macro and `#[prop(optional, into)]`. The `resolve_widget` helper
    // mirrors the component's resolution logic so we can test every usage
    // variant in native Rust tests (no browser DOM needed).

    /// State type for the test widget.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct WidgetState {
        active: bool,
    }

    /// A minimal component that exercises `ClassProp`, `StyleProp`, and
    /// `RenderProp` exactly as the real components do.
    /// Not called in tests — exists to prove the types compile with `#[component]`.
    #[component]
    fn TestWidget(
        #[prop(optional)]
        active: Option<bool>,
        #[prop(optional, into)]
        class: ClassProp<WidgetState>,
        #[prop(optional, into)]
        style: StyleProp<WidgetState>,
        #[prop(optional, into)]
        render: RenderProp<WidgetState>,
    ) -> impl IntoView {
        let state = WidgetState {
            active: active.unwrap_or(false),
        };

        if render.is_custom() {
            let props = RenderProps {
                class: class.resolve(&state),
                style: style.resolve(&state),
                attrs: vec![],
            }
            .attr("role", "widget")
            .attr("data-active", if state.active { "" } else { "false" });
            return render.call(props, &state).unwrap().into_any();
        }

        let class_val = class.resolve_option(&state);
        let style_val = style.resolve_option(&state);

        view! {
            <div
                role="widget"
                class=class_val
                style=style_val
                data-active=if state.active { Some("") } else { None }
            />
        }
        .into_any()
    }

    // -- Resolution helper --------------------------------------------------

    /// What the component would resolve for a given set of inputs.
    #[derive(Debug)]
    struct Resolved {
        class: Option<String>,
        style: Option<String>,
        is_custom_render: bool,
        /// Present only when `render.is_custom()`.
        render_props: Option<RenderProps>,
    }

    /// Mirrors `TestWidget`'s resolution logic without needing the Leptos
    /// runtime, so every code path can be exercised in `#[test]`.
    fn resolve_widget(
        active: bool,
        class: ClassProp<WidgetState>,
        style: StyleProp<WidgetState>,
        render: RenderProp<WidgetState>,
    ) -> Resolved {
        let state = WidgetState { active };

        if render.is_custom() {
            let props = RenderProps {
                class: class.resolve(&state),
                style: style.resolve(&state),
                attrs: vec![],
            }
            .attr("role", "widget")
            .attr("data-active", if state.active { "" } else { "false" });
            Resolved {
                class: Some(props.class.clone()),
                style: Some(props.style.clone()),
                is_custom_render: true,
                render_props: Some(props),
            }
        } else {
            Resolved {
                class: class.resolve_option(&state),
                style: style.resolve_option(&state),
                is_custom_render: false,
                render_props: None,
            }
        }
    }

    // -- Default (no props) -------------------------------------------------

    #[test]
    fn widget_defaults_produce_no_class_or_style() {
        let r = resolve_widget(
            false,
            ClassProp::default(),
            StyleProp::default(),
            RenderProp::default(),
        );
        assert!(r.class.is_none());
        assert!(r.style.is_none());
        assert!(!r.is_custom_render);
        assert!(r.render_props.is_none());
    }

    // -- ClassProp variants -------------------------------------------------

    #[test]
    fn widget_class_from_str_literal() {
        let r = resolve_widget(
            false,
            ClassProp::from("btn"),
            StyleProp::default(),
            RenderProp::default(),
        );
        assert_eq!(r.class, Some("btn".to_string()));
    }

    #[test]
    fn widget_class_from_owned_string() {
        let r = resolve_widget(
            false,
            ClassProp::from(String::from("card")),
            StyleProp::default(),
            RenderProp::default(),
        );
        assert_eq!(r.class, Some("card".to_string()));
    }

    #[test]
    fn widget_class_from_state_closure_inactive() {
        let class: ClassProp<WidgetState> = ClassProp(Box::new(|s| {
            if s.active {
                "widget active".into()
            } else {
                "widget".into()
            }
        }));
        let r = resolve_widget(false, class, StyleProp::default(), RenderProp::default());
        assert_eq!(r.class, Some("widget".to_string()));
    }

    #[test]
    fn widget_class_from_state_closure_active() {
        let class: ClassProp<WidgetState> = ClassProp(Box::new(|s| {
            if s.active {
                "widget active".into()
            } else {
                "widget".into()
            }
        }));
        let r = resolve_widget(true, class, StyleProp::default(), RenderProp::default());
        assert_eq!(r.class, Some("widget active".to_string()));
    }

    #[test]
    fn widget_class_closure_returning_empty_yields_none() {
        let class: ClassProp<WidgetState> =
            ClassProp(Box::new(|_| String::new()));
        let r = resolve_widget(false, class, StyleProp::default(), RenderProp::default());
        assert!(r.class.is_none());
    }

    // -- StyleProp variants -------------------------------------------------

    #[test]
    fn widget_style_from_str_literal() {
        let r = resolve_widget(
            false,
            ClassProp::default(),
            StyleProp::from("color: red;"),
            RenderProp::default(),
        );
        assert_eq!(r.style, Some("color: red;".to_string()));
    }

    #[test]
    fn widget_style_from_owned_string() {
        let r = resolve_widget(
            false,
            ClassProp::default(),
            StyleProp::from(String::from("display: flex;")),
            RenderProp::default(),
        );
        assert_eq!(r.style, Some("display: flex;".to_string()));
    }

    #[test]
    fn widget_style_from_state_closure() {
        let style: StyleProp<WidgetState> = StyleProp(Box::new(|s| {
            if s.active {
                "opacity: 1;".into()
            } else {
                "opacity: 0.5;".into()
            }
        }));
        let r = resolve_widget(true, ClassProp::default(), style, RenderProp::default());
        assert_eq!(r.style, Some("opacity: 1;".to_string()));
    }

    #[test]
    fn widget_style_closure_returning_empty_yields_none() {
        let style: StyleProp<WidgetState> =
            StyleProp(Box::new(|_| String::new()));
        let r = resolve_widget(false, ClassProp::default(), style, RenderProp::default());
        assert!(r.style.is_none());
    }

    // -- Class + Style combined ---------------------------------------------

    #[test]
    fn widget_class_and_style_both_present() {
        let r = resolve_widget(
            false,
            ClassProp::from("card"),
            StyleProp::from("padding: 8px;"),
            RenderProp::default(),
        );
        assert_eq!(r.class, Some("card".to_string()));
        assert_eq!(r.style, Some("padding: 8px;".to_string()));
        assert!(!r.is_custom_render);
    }

    // -- RenderProp variants ------------------------------------------------

    #[test]
    fn widget_render_default_is_not_custom() {
        let r = resolve_widget(
            false,
            ClassProp::default(),
            StyleProp::default(),
            RenderProp::default(),
        );
        assert!(!r.is_custom_render);
    }

    #[test]
    fn widget_render_custom_receives_resolved_class_and_style() {
        let class = ClassProp::from("rendered");
        let style = StyleProp::from("color: blue;");
        // RenderProp needs a real closure; we just need is_custom to be true
        // and to inspect the RenderProps that would be passed.
        let render: RenderProp<WidgetState> = RenderProp(Some(Box::new(
            |_props: RenderProps, _state: &WidgetState| {
                // In a real component this returns a view; here we just need
                // the resolve_widget helper to take the custom path.
                ().into_any()
            },
        )));
        let r = resolve_widget(false, class, style, render);
        assert!(r.is_custom_render);
        let rp = r.render_props.unwrap();
        assert_eq!(rp.class, "rendered");
        assert_eq!(rp.style, "color: blue;");
    }

    #[test]
    fn widget_render_custom_includes_data_attrs() {
        let render: RenderProp<WidgetState> = RenderProp(Some(Box::new(
            |_props: RenderProps, _state: &WidgetState| ().into_any(),
        )));
        let r = resolve_widget(true, ClassProp::default(), StyleProp::default(), render);
        let rp = r.render_props.unwrap();
        // Should have role + data-active attrs
        assert_eq!(rp.attrs.len(), 2);
        assert_eq!(rp.attrs[0], ("role", "widget".to_string()));
        // active=true → data-active=""
        assert_eq!(rp.attrs[1], ("data-active", "".to_string()));
    }

    #[test]
    fn widget_render_custom_inactive_data_attr() {
        let render: RenderProp<WidgetState> = RenderProp(Some(Box::new(
            |_props: RenderProps, _state: &WidgetState| ().into_any(),
        )));
        let r = resolve_widget(false, ClassProp::default(), StyleProp::default(), render);
        let rp = r.render_props.unwrap();
        // active=false → data-active="false"
        assert_eq!(rp.attrs[1], ("data-active", "false".to_string()));
    }

    #[test]
    fn widget_render_custom_with_state_dependent_class() {
        let class: ClassProp<WidgetState> = ClassProp(Box::new(|s| {
            if s.active { "on".into() } else { "off".into() }
        }));
        let render: RenderProp<WidgetState> = RenderProp(Some(Box::new(
            |_props: RenderProps, _state: &WidgetState| ().into_any(),
        )));
        // active = true → class should resolve to "on"
        let r = resolve_widget(true, class, StyleProp::default(), render);
        let rp = r.render_props.unwrap();
        assert_eq!(rp.class, "on");
    }

    #[test]
    fn widget_render_custom_with_state_dependent_style() {
        let style: StyleProp<WidgetState> = StyleProp(Box::new(|s| {
            if s.active {
                "font-weight: bold;".into()
            } else {
                String::new()
            }
        }));
        let render: RenderProp<WidgetState> = RenderProp(Some(Box::new(
            |_props: RenderProps, _state: &WidgetState| ().into_any(),
        )));
        // active = false → style should resolve to ""
        let r = resolve_widget(false, ClassProp::default(), style, render);
        let rp = r.render_props.unwrap();
        assert!(rp.style.is_empty());
    }

    // -- Full combination ---------------------------------------------------

    #[test]
    fn widget_all_props_combined() {
        let class: ClassProp<WidgetState> = ClassProp(Box::new(|s| {
            format!("widget{}", if s.active { " active" } else { "" })
        }));
        let style: StyleProp<WidgetState> = StyleProp(Box::new(|s| {
            if s.active {
                "outline: 2px solid blue;".into()
            } else {
                "outline: none;".into()
            }
        }));
        let render: RenderProp<WidgetState> = RenderProp(Some(Box::new(
            |_props: RenderProps, _state: &WidgetState| ().into_any(),
        )));

        let r = resolve_widget(true, class, style, render);
        assert!(r.is_custom_render);
        let rp = r.render_props.unwrap();
        assert_eq!(rp.class, "widget active");
        assert_eq!(rp.style, "outline: 2px solid blue;");
        assert_eq!(rp.attrs[0], ("role", "widget".to_string()));
        assert_eq!(rp.attrs[1], ("data-active", "".to_string()));
    }

    // -- State changes produce different results ----------------------------

    #[test]
    fn widget_same_props_different_state_produces_different_output() {
        let make_class = || -> ClassProp<WidgetState> {
            ClassProp(Box::new(|s| {
                if s.active { "active".into() } else { "inactive".into() }
            }))
        };
        let make_style = || -> StyleProp<WidgetState> {
            StyleProp(Box::new(|s| {
                if s.active {
                    "color: green;".into()
                } else {
                    "color: gray;".into()
                }
            }))
        };

        let r_off = resolve_widget(false, make_class(), make_style(), RenderProp::default());
        let r_on = resolve_widget(true, make_class(), make_style(), RenderProp::default());

        assert_eq!(r_off.class, Some("inactive".to_string()));
        assert_eq!(r_off.style, Some("color: gray;".to_string()));
        assert_eq!(r_on.class, Some("active".to_string()));
        assert_eq!(r_on.style, Some("color: green;".to_string()));
    }
}
