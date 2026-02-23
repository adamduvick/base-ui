//! Deduplicated warning logging.
//!
//! Ported from `@base-ui/utils/warn`.
//! Logs each unique message at most once via `web_sys::console::warn_1`.
//! In debug builds only (controlled by `cfg(debug_assertions)`).

use std::collections::HashSet;
use std::sync::Mutex;

use std::sync::LazyLock;

static SEEN: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

/// Log a warning message prefixed with "Base UI: ", deduplicating by message content.
///
/// Only active in debug builds (`cfg(debug_assertions)`). In release builds this is a no-op.
///
/// Maps to `warn(...messages)` in the React version.
pub fn warn(messages: &[&str]) {
    if cfg!(debug_assertions) {
        let message_key: String = messages.join(" ");
        let mut set = SEEN.lock().unwrap();
        if !set.contains(&message_key) {
            set.insert(message_key.clone());
            web_sys::console::warn_1(&format!("Base UI: {message_key}").into());
        }
    }
}

/// Reset the deduplication set. Useful for testing.
pub fn reset() {
    if cfg!(debug_assertions) {
        let mut set = SEEN.lock().unwrap();
        set.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_clears_seen_messages() {
        reset();
        let set = SEEN.lock().unwrap();
        assert!(set.is_empty());
    }

    #[test]
    fn deduplication_logic() {
        reset();
        {
            let mut set = SEEN.lock().unwrap();
            assert!(!set.contains("test warning"));
            set.insert("test warning".to_string());
            assert!(set.contains("test warning"));
            set.insert("test warning".to_string());
            assert_eq!(set.len(), 1);
        }
        reset();
    }
}
