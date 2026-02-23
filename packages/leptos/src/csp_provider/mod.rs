//! Content Security Policy provider.
//!
//! Ported from `@base-ui/react/csp-provider`.
//! Provides a CSP configuration context for Base UI components that
//! require inline `<style>` or `<script>` tags.
//!
//! ## What's ported
//! - `CspContext` — context value struct
//! - `provide_csp` — provides CSP context to children
//! - `use_csp` — consumes CSP context from ancestor
//!
//! ## What's skipped (React-specific)
//! - `<CSPProvider>` JSX component wrapper — use `provide_csp()` directly
//! - `CSPProvider.Props` / `CSPProvider.State` namespace types
//!
//! ## Leptos usage
//! ```ignore
//! // In a parent component:
//! provide_csp(CspContext { nonce: Some("abc123".into()), ..Default::default() });
//!
//! // In a child component:
//! let csp = use_csp();
//! if !csp.disable_style_elements {
//!     // render inline <style> with csp.nonce
//! }
//! ```

use leptos::prelude::*;

/// CSP configuration passed down through context.
///
/// Maps to `CSPContextValue` in the React version.
#[derive(Clone, Debug, PartialEq)]
pub struct CspContext {
    /// Nonce value to apply to inline `<style>` and `<script>` tags.
    pub nonce: Option<String>,
    /// Whether inline `<style>` elements created by Base UI components should
    /// not be rendered. When `true`, components must specify CSS styles via
    /// custom class names or other methods.
    pub disable_style_elements: bool,
}

impl Default for CspContext {
    fn default() -> Self {
        CspContext {
            nonce: None,
            disable_style_elements: false,
        }
    }
}

/// Provide CSP context to descendant components.
///
/// Call this in a parent component's body to make CSP configuration
/// available to all children via `use_csp()`.
///
/// Maps to `<CSPProvider>` in the React version.
pub fn provide_csp(context: CspContext) {
    provide_context(context);
}

/// Consume the CSP context from an ancestor `provide_csp()` call.
///
/// Returns the default `CspContext` if no provider exists in the tree.
///
/// Maps to `useCSPContext()` in the React version.
pub fn use_csp() -> CspContext {
    use_context::<CspContext>().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from CSPProvider.test.tsx:
    // ✗ does not render inline style tags when disableStyleElements is true
    //   (requires ScrollArea component — not yet ported)
    // ✗ does not render Select inline style tags when disableStyleElements is true
    //   (requires Select component — not yet ported)
    // ✗ applies nonce to inline style tags
    //   (requires ScrollArea component — not yet ported)
    // ✗ renders inline style tags by default
    //   (requires ScrollArea component — not yet ported)
    //
    // All React tests are integration tests requiring ScrollArea/Select.
    // Unit tests below verify the context struct and defaults.

    #[test]
    fn default_context() {
        let ctx = CspContext::default();
        assert_eq!(ctx.nonce, None);
        assert!(!ctx.disable_style_elements);
    }

    #[test]
    fn context_with_nonce() {
        let ctx = CspContext {
            nonce: Some("test-nonce".into()),
            disable_style_elements: false,
        };
        assert_eq!(ctx.nonce.as_deref(), Some("test-nonce"));
    }

    #[test]
    fn context_with_disabled_styles() {
        let ctx = CspContext {
            nonce: None,
            disable_style_elements: true,
        };
        assert!(ctx.disable_style_elements);
    }

    #[test]
    fn context_equality() {
        let a = CspContext::default();
        let b = CspContext::default();
        assert_eq!(a, b);

        let c = CspContext {
            nonce: Some("x".into()),
            disable_style_elements: false,
        };
        assert_ne!(a, c);
    }
}
