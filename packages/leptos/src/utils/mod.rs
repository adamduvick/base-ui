//! Utility modules ported from `@base-ui/utils`.
//! Each submodule corresponds to a unit in `analyze_deps.py`.

pub mod animation_frame;
pub mod detect_browser;
pub mod empty;
pub mod error;
pub mod format_error_message;
pub mod generate_id;
pub mod interval;
pub mod is_element_disabled;
pub mod is_mouse_within_bounds;
pub mod owner;
pub mod scroll_lock;
pub mod store;
pub mod timeout;
pub mod use_controlled;
pub mod use_enhanced_click_handler;
pub mod use_id;
pub mod use_on_mount;
pub mod use_previous_value;
pub mod visually_hidden;
pub mod warn;

// Documentation-only module listing skipped React-specific utils.
mod skipped;
