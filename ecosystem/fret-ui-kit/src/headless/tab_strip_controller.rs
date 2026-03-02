//! Shared (policy-layer) helpers for editor-grade tab strip interactions.
//!
//! This module intentionally does **not** define geometry/layout rules. Adapters (workspace,
//! docking, etc.) are responsible for hit-testing and surfacing a `TabStripHitTarget`.
//!
//! Rationale:
//! - Pure mechanism helpers (surface classification, overflow membership, canonical insert index)
//!   live in `fret-ui-headless`.
//! - Policy decisions (close vs activate arbitration, overflow dropdown semantics) live here so we
//!   can share behavior across ecosystem implementations without expanding `fret-ui` contracts.

/// A coarse-grained hit-test result for a tab strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabStripHitTarget {
    /// The overflow dropdown/menu surface.
    OverflowMenuRow {
        index: usize,
        part: OverflowMenuPart,
    },
    /// The overflow control button in the strip.
    OverflowButton,
    /// A tab in the strip.
    Tab { index: usize, part: TabPart },
    /// Header / empty space inside the strip (e.g. end-drop surface).
    HeaderSpace,
    /// Outside the strip.
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowMenuPart {
    Content,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabPart {
    Content,
    Close,
}

/// An intent that an adapter can translate into its domain operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabStripIntent {
    /// Toggle open/close for the overflow dropdown/menu.
    ToggleOverflowMenu,
    /// Activate the given tab index. Adapters can decide what "activate" means (focus, selection).
    Activate { index: usize, ensure_visible: bool },
    /// Close the given tab index. Adapters must ensure this does not implicitly activate.
    Close { index: usize },
    /// No action.
    None,
}

/// Policy: map a click hit target to an intent.
///
/// This function encodes the editor-grade arbitration rules:
/// - Overflow menu close should close without activation.
/// - Overflow menu content should activate and keep the tab visible.
/// - Strip tab close should close without activation.
/// - Strip tab content should activate (without forcing ensure-visible unless adapter wants it).
/// - Overflow button toggles the menu.
pub fn intent_for_click(hit: TabStripHitTarget) -> TabStripIntent {
    match hit {
        TabStripHitTarget::OverflowMenuRow {
            index,
            part: OverflowMenuPart::Close,
        } => TabStripIntent::Close { index },
        TabStripHitTarget::OverflowMenuRow {
            index,
            part: OverflowMenuPart::Content,
        } => TabStripIntent::Activate {
            index,
            ensure_visible: true,
        },
        TabStripHitTarget::OverflowButton => TabStripIntent::ToggleOverflowMenu,
        TabStripHitTarget::Tab {
            index,
            part: TabPart::Close,
        } => TabStripIntent::Close { index },
        TabStripHitTarget::Tab {
            index,
            part: TabPart::Content,
        } => TabStripIntent::Activate {
            index,
            ensure_visible: false,
        },
        TabStripHitTarget::HeaderSpace | TabStripHitTarget::None => TabStripIntent::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_menu_close_does_not_activate() {
        let intent = intent_for_click(TabStripHitTarget::OverflowMenuRow {
            index: 3,
            part: OverflowMenuPart::Close,
        });
        assert_eq!(intent, TabStripIntent::Close { index: 3 });
    }

    #[test]
    fn overflow_menu_content_activates_and_ensures_visible() {
        let intent = intent_for_click(TabStripHitTarget::OverflowMenuRow {
            index: 3,
            part: OverflowMenuPart::Content,
        });
        assert_eq!(
            intent,
            TabStripIntent::Activate {
                index: 3,
                ensure_visible: true
            }
        );
    }

    #[test]
    fn strip_tab_close_does_not_activate() {
        let intent = intent_for_click(TabStripHitTarget::Tab {
            index: 1,
            part: TabPart::Close,
        });
        assert_eq!(intent, TabStripIntent::Close { index: 1 });
    }
}
