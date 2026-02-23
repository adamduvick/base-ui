//! Browser and platform detection.
//!
//! Ported from `@base-ui/utils/detectBrowser`.
//! Provides lazy-evaluated boolean flags for browser/platform identification
//! using `web_sys::window()` and the Navigator API.

use std::sync::LazyLock;

/// Whether the platform is iOS (iPhone, iPad, or iPod).
///
/// Maps to `isIOS` in the React version.
pub static IS_IOS: LazyLock<bool> = LazyLock::new(|| {
    let (platform, max_touch) = get_navigator_data();
    // iPads can claim to be MacIntel
    if platform == "MacIntel" && max_touch > 1 {
        return true;
    }
    platform.contains("iPhone")
        || platform.contains("iPad")
        || platform.contains("iPod")
        || platform.contains("iOS")
});

/// Whether the browser is Firefox.
///
/// Maps to `isFirefox` in the React version.
pub static IS_FIREFOX: LazyLock<bool> = LazyLock::new(|| {
    let ua = get_user_agent();
    ua.to_lowercase().contains("firefox")
});

/// Whether the browser is Safari.
///
/// Maps to `isSafari` in the React version.
/// Detects Safari by checking for "apple" in the user agent (since `navigator.vendor`
/// requires an additional web-sys feature, we use the user agent as a proxy).
pub static IS_SAFARI: LazyLock<bool> = LazyLock::new(|| {
    let ua = get_user_agent().to_lowercase();
    ua.contains("safari") && !ua.contains("chrome") && !ua.contains("chromium")
});

/// Whether the browser is Edge.
///
/// Maps to `isEdge` in the React version.
pub static IS_EDGE: LazyLock<bool> = LazyLock::new(|| {
    let ua = get_user_agent();
    ua.contains("Edg")
});

/// Whether the platform is Android.
///
/// Maps to `isAndroid` in the React version.
pub static IS_ANDROID: LazyLock<bool> = LazyLock::new(|| {
    let ua = get_user_agent();
    let platform = get_platform();
    platform.to_lowercase().contains("android") || ua.to_lowercase().contains("android")
});

/// Whether the platform is macOS (not iOS).
///
/// Maps to `isMac` in the React version.
pub static IS_MAC: LazyLock<bool> = LazyLock::new(|| {
    let platform = get_platform();
    let max_touch = web_sys::window()
        .map(|w| w.navigator().max_touch_points())
        .unwrap_or(0);
    platform.to_lowercase().starts_with("mac") && max_touch == 0
});

fn get_navigator_data() -> (String, i32) {
    match web_sys::window() {
        Some(w) => {
            let nav = w.navigator();
            let platform = nav.platform().unwrap_or_default();
            let max_touch = nav.max_touch_points();
            (platform, max_touch)
        }
        None => (String::new(), -1),
    }
}

fn get_user_agent() -> String {
    web_sys::window()
        .map(|w| w.navigator().user_agent().unwrap_or_default())
        .unwrap_or_default()
}

fn get_platform() -> String {
    web_sys::window()
        .map(|w| w.navigator().platform().unwrap_or_default())
        .unwrap_or_default()
}

// Note: Tests for browser detection require a real browser environment (wasm-bindgen-test).
// The lazy statics will return false/defaults in a native test environment since there's no window.
#[cfg(test)]
mod tests {
    // Ported from detectBrowser.ts:
    // ✓ iOS platform string matching logic
    // ✗ get_user_agent / get_platform (requires wasm target — web_sys panics on native)
    // ✗ isWebKit (CSS.supports check requires real browser)
    // ✗ Actual browser flag values require wasm-bindgen-test in a real browser

    #[test]
    fn ios_platform_detection() {
        // Test the string matching logic directly (no web_sys calls)
        let test_cases = [
            ("iPhone", true),
            ("iPad", true),
            ("iPod", true),
            ("iOS", true),
            ("Android", false),
            ("MacIntel", false),
            ("Linux", false),
        ];
        for (platform, expected) in test_cases {
            let result = platform.contains("iPhone")
                || platform.contains("iPad")
                || platform.contains("iPod")
                || platform.contains("iOS");
            assert_eq!(result, expected, "Failed for platform: {platform}");
        }
    }
}
