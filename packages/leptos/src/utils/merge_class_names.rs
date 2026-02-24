//! Class name merging utility.
//!
//! Ported from `packages/react/src/merge-props/mergeProps.ts` (`mergeClassNames`).

/// Merges two optional class name strings.
///
/// Returns a combined string with `theirs` first, then `ours`, separated by
/// a space. If only one is present, returns it directly.
///
/// Maps to `mergeClassNames(ourClassName, theirClassName)` in the React version.
pub fn merge_class_names(ours: Option<&str>, theirs: Option<&str>) -> Option<String> {
    match (theirs, ours) {
        (Some(t), Some(o)) => Some(format!("{t} {o}")),
        (Some(t), None) => Some(t.to_string()),
        (None, Some(o)) => Some(o.to_string()),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_both() {
        assert_eq!(
            merge_class_names(Some("our-class"), Some("their-class")),
            Some("their-class our-class".to_string())
        );
    }

    #[test]
    fn returns_theirs_only() {
        assert_eq!(
            merge_class_names(None, Some("their-class")),
            Some("their-class".to_string())
        );
    }

    #[test]
    fn returns_ours_only() {
        assert_eq!(
            merge_class_names(Some("our-class"), None),
            Some("our-class".to_string())
        );
    }

    #[test]
    fn returns_none_for_both_none() {
        assert_eq!(merge_class_names(None, None), None);
    }

    #[test]
    fn preserves_multiple_classes() {
        assert_eq!(
            merge_class_names(Some("a b"), Some("c d")),
            Some("c d a b".to_string())
        );
    }
}
