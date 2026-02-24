//! Avatar component.
//!
//! Ported from `@base-ui/react/avatar`.
//! Displays a user's profile picture, initials, or fallback icon. Three parts:
//! `AvatarRoot`, `AvatarImage`, `AvatarFallback`.
//!
//! ## What's ported
//! - `AvatarRoot` — manages image loading status via reactive store context
//! - `AvatarImage` — preloads image via `HtmlImageElement`, renders `<img>` when loaded
//! - `AvatarFallback` — renders fallback content when image is not loaded
//! - `ImageLoadingStatus` — status enum (idle/loading/loaded/error)
//! - `ClassProp<Store<AvatarState>>` — static string or state-dependent class
//! - `StyleProp<Store<AvatarState>>` — static string or state-dependent style
//! - `RenderProp<Store<AvatarState>>` — custom render function to replace the default element
//!
//! ## Reactive state
//! `AvatarState` uses `#[derive(Store)]` from `reactive_stores`. The store is
//! provided via context and children subscribe to `store.image_loading_status().get()`
//! for fine-grained reactivity when the image loading status changes.
//!
//! ## What's skipped (React-specific)
//! - `useTransitionStatus` / `useOpenChangeComplete` — CSS animation lifecycle
//!   (can be added later with Leptos animation primitives)
//! - `data-starting-style` / `data-ending-style` transition attributes
//!
//! ## Leptos usage
//! ```ignore
//! use base_ui_leptos::avatar::*;
//! use reactive_stores::Store;
//!
//! view! {
//!     <AvatarRoot>
//!         <AvatarImage src="https://example.com/avatar.jpg" />
//!         <AvatarFallback>"AB"</AvatarFallback>
//!     </AvatarRoot>
//! }
//! ```

use leptos::prelude::*;
use reactive_stores::Store;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::utils::props::{ClassProp, RenderProp, RenderProps, StyleProp};

/// Image loading status.
///
/// Maps to `ImageLoadingStatus` in the React version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ImageLoadingStatus {
    /// No image source provided or not yet started.
    #[default]
    Idle,
    /// Image is currently loading.
    Loading,
    /// Image loaded successfully.
    Loaded,
    /// Image failed to load.
    Error,
}

impl ImageLoadingStatus {
    /// Returns the status as a lowercase string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageLoadingStatus::Idle => "idle",
            ImageLoadingStatus::Loading => "loading",
            ImageLoadingStatus::Loaded => "loaded",
            ImageLoadingStatus::Error => "error",
        }
    }
}

/// State of the Avatar component parts.
///
/// Maps to `AvatarRoot.State` / `AvatarImage.State` / `AvatarFallback.State`
/// in the React version.
/// Uses `#[derive(Store)]` for reactive field-level access via `Store<AvatarState>`.
/// The generated `AvatarStateStoreFields` trait provides `.image_loading_status()`
/// for fine-grained subscriptions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Store)]
pub struct AvatarState {
    pub image_loading_status: ImageLoadingStatus,
}

/// Displays a user's profile picture, initials, or fallback icon.
/// Renders a `<span>` element. Manages image loading status shared
/// with `AvatarImage` and `AvatarFallback` children via a reactive store.
///
/// Maps to `<Avatar.Root>` in the React version.
#[component]
pub fn AvatarRoot(
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<AvatarState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<AvatarState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<AvatarState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<AvatarState>>,
    /// Custom render function. When provided, replaces the default `<span>` element.
    /// Children are not rendered when a custom render function is used.
    #[prop(optional, into)]
    render: RenderProp<Store<AvatarState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Span>>,
    children: Children,
) -> impl IntoView {
    let store = Store::new(AvatarState {
        image_loading_status: ImageLoadingStatus::Idle,
    });

    provide_context(store);

    if render.is_custom() {
        return (move || {
            // Subscribe to status so the render closure re-runs
            let _ = store.image_loading_status().get();
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
        <span
            class=move || {
                let _ = store.image_loading_status().get();
                class.resolve_option(&store)
            }
            style=move || {
                let _ = store.image_loading_status().get();
                style.resolve_option(&store)
            }
            node_ref=node_ref.unwrap_or_default()
        >
            {children_view}
        </span>
    }
    .into_any()
}

/// The image to be displayed in the avatar.
/// Renders an `<img>` element when the image has loaded successfully.
/// The image is preloaded via an off-screen `HtmlImageElement` to avoid
/// layout shifts.
///
/// Maps to `<Avatar.Image>` in the React version.
///
/// Transition animation support (`data-starting-style` / `data-ending-style`)
/// is not yet implemented — the image appears/disappears immediately.
#[component]
pub fn AvatarImage(
    /// Image source URL.
    #[prop(optional, into)]
    src: Option<String>,
    /// Image alt text.
    #[prop(optional, into)]
    alt: Option<String>,
    /// CORS cross-origin attribute.
    #[prop(optional, into)]
    cross_origin: Option<String>,
    /// Referrer policy for the image request.
    #[prop(optional, into)]
    referrer_policy: Option<String>,
    /// Callback fired when the loading status changes.
    #[prop(optional)]
    on_loading_status_change: Option<fn(ImageLoadingStatus)>,
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<AvatarState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<AvatarState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<AvatarState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<AvatarState>>,
    /// Custom render function. When provided, replaces the default `<img>` element.
    #[prop(optional, into)]
    render: RenderProp<Store<AvatarState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Img>>,
) -> impl IntoView {
    let store = expect_context::<Store<AvatarState>>();

    // Preload the image to detect load/error status before rendering
    Effect::new({
        let src = src.clone();
        let cross_origin = cross_origin.clone();
        let referrer_policy = referrer_policy.clone();
        move |_| {
            let Some(src_url) = src.as_deref() else {
                store.image_loading_status().set(ImageLoadingStatus::Error);
                if let Some(cb) = on_loading_status_change {
                    cb(ImageLoadingStatus::Error);
                }
                return;
            };

            store
                .image_loading_status()
                .set(ImageLoadingStatus::Loading);
            if let Some(cb) = on_loading_status_change {
                cb(ImageLoadingStatus::Loading);
            }

            let img = web_sys::HtmlImageElement::new().expect("failed to create Image");

            // Set up onload handler
            let onload = Closure::wrap(Box::new(move || {
                store.image_loading_status().set(ImageLoadingStatus::Loaded);
                if let Some(cb) = on_loading_status_change {
                    cb(ImageLoadingStatus::Loaded);
                }
            }) as Box<dyn Fn()>);
            img.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();

            // Set up onerror handler
            let onerror = Closure::wrap(Box::new(move || {
                store.image_loading_status().set(ImageLoadingStatus::Error);
                if let Some(cb) = on_loading_status_change {
                    cb(ImageLoadingStatus::Error);
                }
            }) as Box<dyn Fn()>);
            img.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            onerror.forget();

            // Set CORS and referrer policy
            if let Some(ref co) = cross_origin {
                img.set_cross_origin(Some(co));
            }
            if let Some(ref rp) = referrer_policy {
                img.set_referrer_policy(rp);
            }

            // Start loading
            img.set_src(src_url);
        }
    });

    let is_loaded = move || store.image_loading_status().get() == ImageLoadingStatus::Loaded;

    if render.is_custom() {
        return (move || {
            if !is_loaded() {
                return ().into_any();
            }
            let props = RenderProps {
                class: class.resolve(&store),
                style: style.resolve(&store),
                attrs: vec![],
            };
            render.call(props, &store).unwrap()
        })
        .into_any();
    }

    let src_for_view = src.clone();
    let alt_for_view = alt.clone();

    view! {
        <Show when=is_loaded>
            {
                let _ = store.image_loading_status().get();
                let class_val = class.resolve_option(&store);
                let style_val = style.resolve_option(&store);
                view! {
                    <img
                        src=src_for_view.clone()
                        alt=alt_for_view.clone().unwrap_or_default()
                        class=class_val
                        style=style_val
                        node_ref=node_ref.unwrap_or_default()
                    />
                }
            }
        </Show>
    }
    .into_any()
}

/// Rendered when the image fails to load or when no image is provided.
/// Renders a `<span>` element.
///
/// Maps to `<Avatar.Fallback>` in the React version.
///
/// Unlike the React version, this uses CSS `display: none` for conditional
/// visibility instead of conditional rendering, since Leptos `Show` requires
/// `Fn` children but `Children` is `FnOnce`.
#[component]
pub fn AvatarFallback(
    /// How long to wait before showing the fallback, in milliseconds.
    /// If not set, the fallback is shown immediately when needed.
    #[prop(optional)]
    delay: Option<i32>,
    /// CSS class name(s). Accepts a static string or a closure receiving `&Store<AvatarState>`.
    #[prop(optional, into)]
    class: ClassProp<Store<AvatarState>>,
    /// Inline styles. Accepts a static string or a closure receiving `&Store<AvatarState>`.
    #[prop(optional, into)]
    style: StyleProp<Store<AvatarState>>,
    /// Custom render function. When provided, replaces the default `<span>` element.
    /// Children are not rendered when a custom render function is used.
    #[prop(optional, into)]
    render: RenderProp<Store<AvatarState>>,
    /// Node ref for direct DOM access.
    #[prop(optional)]
    node_ref: Option<NodeRef<leptos::html::Span>>,
    children: Children,
) -> impl IntoView {
    let store = expect_context::<Store<AvatarState>>();
    let (delay_passed, set_delay_passed) = signal(delay.is_none());

    // Handle delay timer using web_sys directly (Timeout uses Rc which isn't Send+Sync)
    if let Some(delay_ms) = delay {
        let timeout_handle: RwSignal<Option<i32>> = RwSignal::new(None);

        let cb = Closure::wrap(Box::new(move || {
            set_delay_passed.set(true);
        }) as Box<dyn Fn()>);

        if let Some(window) = web_sys::window() {
            if let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                delay_ms,
            ) {
                timeout_handle.set(Some(id));
            }
        }
        cb.forget();

        on_cleanup(move || {
            if let Some(id) = timeout_handle.get() {
                if let Some(window) = web_sys::window() {
                    window.clear_timeout_with_handle(id);
                }
            }
        });
    }

    let should_show = move || {
        store.image_loading_status().get() != ImageLoadingStatus::Loaded && delay_passed.get()
    };

    if render.is_custom() {
        return (move || {
            if !should_show() {
                return ().into_any();
            }
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
        <span
            class=move || {
                let _ = store.image_loading_status().get();
                class.resolve_option(&store)
            }
            style=move || {
                let _ = store.image_loading_status().get();
                style.resolve_option(&store)
            }
            node_ref=node_ref.unwrap_or_default()
            style:display=move || if should_show() { "" } else { "none" }
        >
            {children_view}
        </span>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from AvatarRoot.test.tsx / AvatarImage.test.tsx / AvatarFallback.test.tsx:
    // ✓ ImageLoadingStatus enum values
    // ✓ ImageLoadingStatus default is Idle
    // ✓ ImageLoadingStatus as_str
    // ✓ AvatarState stores image_loading_status
    // ✗ hides fallback when image loads (requires browser DOM)
    // ✗ shows fallback on error (requires browser DOM)
    // ✗ delay prop defers fallback (requires browser DOM)
    // ✗ transition data attributes (skipped — transition status not ported)
    // ✗ describeConformance (React-specific)

    #[test]
    fn loading_status_default() {
        assert_eq!(ImageLoadingStatus::default(), ImageLoadingStatus::Idle);
    }

    #[test]
    fn loading_status_as_str() {
        assert_eq!(ImageLoadingStatus::Idle.as_str(), "idle");
        assert_eq!(ImageLoadingStatus::Loading.as_str(), "loading");
        assert_eq!(ImageLoadingStatus::Loaded.as_str(), "loaded");
        assert_eq!(ImageLoadingStatus::Error.as_str(), "error");
    }

    #[test]
    fn loading_status_equality() {
        assert_eq!(ImageLoadingStatus::Loaded, ImageLoadingStatus::Loaded);
        assert_ne!(ImageLoadingStatus::Loaded, ImageLoadingStatus::Error);
    }

    #[test]
    fn avatar_state_stores_status() {
        let state = AvatarState {
            image_loading_status: ImageLoadingStatus::Loaded,
        };
        assert_eq!(state.image_loading_status, ImageLoadingStatus::Loaded);
    }
}
