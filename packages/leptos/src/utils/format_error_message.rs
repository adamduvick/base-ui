//! Production error message formatter.
//!
//! Ported from `@base-ui/utils/formatErrorMessage`.
//! Generates a compact error message with a URL pointing to the full error description.

/// Format a production error message with a code and arguments.
///
/// Returns a string like:
/// `"Base UI error #1; visit https://base-ui.com/production-error?code=1&args[]=foo for the full message."`
///
/// Maps to `formatErrorMessage(code, ...args)` in the React version.
pub fn format_error_message(code: u32, args: &[&str]) -> String {
    let mut url = format!("https://base-ui.com/production-error?code={code}");
    for arg in args {
        url.push_str(&format!(
            "&args[]={}",
            percent_encode(arg)
        ));
    }
    format!("Base UI error #{code}; visit {url} for the full message.")
}

/// Minimal percent-encoding for URL query parameter values.
fn percent_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from formatErrorMessage.ts:
    // ✓ formats error with code only
    // ✓ formats error with code and arguments
    // ✓ percent-encodes special characters in arguments

    #[test]
    fn format_with_code_only() {
        let result = format_error_message(1, &[]);
        assert!(result.starts_with("Base UI error #1; visit "));
        assert!(result.contains("code=1"));
        assert!(result.ends_with(" for the full message."));
        assert!(!result.contains("args[]"));
    }

    #[test]
    fn format_with_code_and_args() {
        let result = format_error_message(2, &["foo", "bar"]);
        assert!(result.contains("code=2"));
        assert!(result.contains("args[]=foo"));
        assert!(result.contains("args[]=bar"));
    }

    #[test]
    fn percent_encodes_special_characters() {
        let result = format_error_message(3, &["hello world"]);
        assert!(result.contains("args[]=hello%20world"));
    }

    #[test]
    fn percent_encode_preserves_unreserved_chars() {
        assert_eq!(percent_encode("abc123"), "abc123");
        assert_eq!(percent_encode("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn percent_encode_encodes_reserved_chars() {
        assert_eq!(percent_encode("a b"), "a%20b");
        assert_eq!(percent_encode("a&b=c"), "a%26b%3Dc");
    }
}
