/// Canonical-order helpers for editor-style tab strips.
///
/// These helpers are intentionally "headless": they operate on caller-provided slices and pure
/// closures so different tab models (workspace, docking, etc.) can share the same invariants.

/// Resolves the canonical insert index for an "end-drop" within the dragged tab's group.
///
/// This is the canonical-order equivalent of "drop at end of the strip", but restricted to the
/// dragged tab's group (e.g. pinned vs unpinned).
///
/// Returns `None` when the dragged tab is the only member of its group (excluding itself), so
/// callers can treat it as a no-op.
///
/// The returned `insert_index` is in the range `[1, canonical.len()]` (never `0`), and refers to
/// the canonical list after excluding the dragged element from candidates.
pub fn resolve_end_drop_insert_index_in_canonical_order<T, G>(
    canonical: &[T],
    mut is_dragged: impl FnMut(&T) -> bool,
    mut group_key: impl FnMut(&T) -> G,
) -> Option<usize>
where
    G: Copy + PartialEq,
{
    let dragged = canonical.iter().find(|t| is_dragged(t))?;
    let dragged_group = group_key(dragged);

    let mut best: Option<usize> = None;
    for (ix, t) in canonical.iter().enumerate() {
        if is_dragged(t) {
            continue;
        }
        if group_key(t) != dragged_group {
            continue;
        }
        best = Some(ix);
    }

    // Convert "last index in group" to "insert after it".
    best.map(|ix| ix + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_drop_insert_index_respects_group_key() {
        let canonical = ["a", "b", "c", "d"];
        let pinned = |id: &&str| matches!(*id, "a" | "b");

        // Dragging pinned should end-drop after last pinned => insert_index == 2.
        assert_eq!(
            resolve_end_drop_insert_index_in_canonical_order(&canonical, |id| *id == "a", pinned),
            Some(2)
        );

        // Dragging unpinned should end-drop after last unpinned => insert_index == 4.
        assert_eq!(
            resolve_end_drop_insert_index_in_canonical_order(&canonical, |id| *id == "c", pinned),
            Some(4)
        );
    }

    #[test]
    fn end_drop_insert_index_returns_none_when_dragged_is_only_member_of_group() {
        let canonical = ["only", "other"];
        let pinned = |id: &&str| *id == "only";

        assert_eq!(
            resolve_end_drop_insert_index_in_canonical_order(
                &canonical,
                |id| *id == "only",
                pinned
            ),
            None
        );
    }
}
