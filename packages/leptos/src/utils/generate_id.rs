//! Random ID generation.
//!
//! Ported from `@base-ui/utils/generateId`.
//! Generates unique IDs with a prefix, a random component, and a monotonic counter.

use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique ID string with the given prefix.
///
/// Returns a string like `"prefix-a1b2-1"` where the middle segment is random
/// and the suffix is a monotonically increasing counter.
///
/// Maps to `generateId(prefix)` in the React version.
pub fn generate_id(prefix: &str) -> String {
    let count = COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    let random = pseudo_random_base36();
    format!("{prefix}-{random}-{count}")
}

/// Generate a short pseudo-random base-36 string.
/// Uses a simple xorshift PRNG seeded from a global counter to avoid
/// requiring `getrandom` (which needs special WASM configuration).
fn pseudo_random_base36() -> String {
    use std::sync::atomic::AtomicU64;
    static SEED: AtomicU64 = AtomicU64::new(0x12345678_9ABCDEF0);

    let mut s = SEED.load(Ordering::Relaxed);
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    SEED.store(s, Ordering::Relaxed);

    // Take 4 characters of base-36 representation (matching the JS `.slice(2, 6)`)
    let base36 = format_base36(s);
    base36[..4.min(base36.len())].to_string()
}

fn format_base36(mut n: u64) -> String {
    if n == 0 {
        return "0000".to_string();
    }
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut result = Vec::new();
    while n > 0 {
        result.push(CHARS[(n % 36) as usize]);
        n /= 36;
    }
    result.reverse();
    String::from_utf8(result).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ported from generateId.ts:
    // ✓ generates ID with correct prefix
    // ✓ generates unique IDs
    // ✓ counter increments

    #[test]
    fn generates_id_with_prefix() {
        let id = generate_id("test");
        assert!(id.starts_with("test-"));
    }

    #[test]
    fn generates_unique_ids() {
        let id1 = generate_id("a");
        let id2 = generate_id("a");
        assert_ne!(id1, id2);
    }

    #[test]
    fn counter_increments() {
        let id1 = generate_id("b");
        let id2 = generate_id("b");
        // Extract the counter (last segment after final '-')
        let c1: u64 = id1.rsplit('-').next().unwrap().parse().unwrap();
        let c2: u64 = id2.rsplit('-').next().unwrap().parse().unwrap();
        assert_eq!(c2, c1 + 1);
    }

    #[test]
    fn format_base36_works() {
        assert_eq!(format_base36(0), "0000");
        assert_eq!(format_base36(35), "z");
        assert_eq!(format_base36(36), "10");
    }
}
