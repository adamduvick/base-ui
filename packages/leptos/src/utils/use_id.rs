//! Unique ID generation for components.
//!
//! Ported from `@base-ui/utils/useId`.
//! Provides a function to generate unique IDs for DOM elements, with optional
//! prefix and override support.
//!
//! The React version wraps `React.useId()` with fallbacks for older React versions.
//! In Leptos CSR mode, we use a simple atomic counter (no SSR hydration concerns).

use std::sync::atomic::{AtomicU64, Ordering};

static GLOBAL_ID: AtomicU64 = AtomicU64::new(0);

/// Generate a unique ID, or return the override if provided.
///
/// If `id_override` is `Some`, returns the override value unchanged.
/// Otherwise, generates a unique ID using a global atomic counter with an
/// optional prefix (defaults to `"base-ui"`).
///
/// Maps to `useId(idOverride?, prefix?)` in the React version.
///
/// # Arguments
/// * `id_override` - An optional explicit ID to use instead of generating one.
/// * `prefix` - An optional prefix for generated IDs (defaults to `"base-ui"`).
///
/// # Returns
/// The ID string.
pub fn use_id(id_override: Option<&str>, prefix: Option<&str>) -> String {
    if let Some(id) = id_override {
        return id.to_string();
    }

    let prefix = prefix.unwrap_or("base-ui");
    let id = GLOBAL_ID.fetch_add(1, Ordering::Relaxed) + 1;
    format!("{prefix}-{id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from useId.test.tsx:
    // ✓ returns the provided ID
    // ✓ generates an ID if one isn't provided
    // ✓ can be suffixed
    // ✓ can be prefixed
    // ✗ provides an ID on server in React 18 (N/A — CSR only)

    #[test]
    fn returns_provided_id() {
        let id = use_id(Some("custom-id"), None);
        assert_eq!(id, "custom-id");
    }

    #[test]
    fn generates_id_if_not_provided() {
        let id = use_id(None, None);
        assert!(!id.is_empty());
        assert!(id.starts_with("base-ui-"));
    }

    #[test]
    fn generated_ids_are_unique() {
        let id1 = use_id(None, None);
        let id2 = use_id(None, None);
        assert_ne!(id1, id2);
    }

    #[test]
    fn can_be_suffixed() {
        let id = use_id(None, None);
        let label_id = format!("{id}-label");
        assert!(label_id.ends_with("-label"));
        assert!(label_id.starts_with("base-ui-"));
    }

    #[test]
    fn can_be_prefixed() {
        let id = use_id(None, Some("my-prefix"));
        assert!(id.starts_with("my-prefix-"));
    }

    #[test]
    fn override_takes_precedence_over_prefix() {
        let id = use_id(Some("explicit"), Some("prefix"));
        assert_eq!(id, "explicit");
    }
}
