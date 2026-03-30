/// Helpers for deciding which tabs appear in an overflow dropdown/menu.
///
/// This is intentionally small and deterministic: adapters own layout/measurement and pass in
/// `overflowed` indices computed from their own geometry.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowMenuActivePolicy {
    /// Do not force-insert the active tab into the overflow dropdown.
    Exclude,
    /// Ensure the active tab is present (even if it is not overflowed), so it is always reachable.
    Include,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowMenuEmptyOverflowedPolicy {
    /// If `overflowed` is empty, return an empty menu.
    Empty,
    /// If `overflowed` is empty, fall back to listing all tabs.
    AllTabs,
}

/// Compute the overflow dropdown item indices in canonical order.
///
/// - `tab_count` is the total number of tabs in the strip.
/// - `overflowed` is a list of indices considered overflowed (caller-provided).
/// - `active` is the active tab index, if known.
///
/// The output is always in canonical order (0..tab_count) and deduplicated.
pub fn compute_overflow_menu_item_indices(
    tab_count: usize,
    overflowed: &[usize],
    active: Option<usize>,
    active_policy: OverflowMenuActivePolicy,
    empty_policy: OverflowMenuEmptyOverflowedPolicy,
) -> Vec<usize> {
    if tab_count == 0 {
        return Vec::new();
    }

    if overflowed.is_empty() {
        return match empty_policy {
            OverflowMenuEmptyOverflowedPolicy::Empty => Vec::new(),
            OverflowMenuEmptyOverflowedPolicy::AllTabs => (0..tab_count).collect(),
        };
    }

    let mut overflowed_set = vec![false; tab_count];
    for &ix in overflowed {
        if ix < tab_count {
            overflowed_set[ix] = true;
        }
    }

    let active_ix = match (active_policy, active) {
        (OverflowMenuActivePolicy::Include, Some(ix)) if ix < tab_count => Some(ix),
        _ => None,
    };

    let mut out = Vec::new();
    for (ix, is_overflowed) in overflowed_set.iter().copied().enumerate() {
        if is_overflowed || active_ix.is_some_and(|a| a == ix) {
            out.push(ix);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_overflowed_can_fallback_to_all_tabs() {
        let items = compute_overflow_menu_item_indices(
            3,
            &[],
            Some(0),
            OverflowMenuActivePolicy::Exclude,
            OverflowMenuEmptyOverflowedPolicy::AllTabs,
        );
        assert_eq!(items, vec![0, 1, 2]);
    }

    #[test]
    fn empty_overflowed_can_produce_empty_menu() {
        let items = compute_overflow_menu_item_indices(
            3,
            &[],
            Some(0),
            OverflowMenuActivePolicy::Include,
            OverflowMenuEmptyOverflowedPolicy::Empty,
        );
        assert_eq!(items, Vec::<usize>::new());
    }

    #[test]
    fn include_active_adds_active_when_not_overflowed() {
        let items = compute_overflow_menu_item_indices(
            5,
            &[4],
            Some(1),
            OverflowMenuActivePolicy::Include,
            OverflowMenuEmptyOverflowedPolicy::Empty,
        );
        assert_eq!(items, vec![1, 4]);
    }

    #[test]
    fn output_is_canonical_and_deduped() {
        let items = compute_overflow_menu_item_indices(
            5,
            &[3, 1, 1, 4],
            Some(4),
            OverflowMenuActivePolicy::Include,
            OverflowMenuEmptyOverflowedPolicy::Empty,
        );
        assert_eq!(items, vec![1, 3, 4]);
    }
}
