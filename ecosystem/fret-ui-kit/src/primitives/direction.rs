//! Direction primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/direction/src/direction.tsx`
//!
//! Radix `direction` is a tiny helper that resolves a local `dir` override against an optional
//! inherited/global direction, defaulting to LTR.
//!
//! Fret does not provide a built-in "direction context" mechanism yet. Callers are expected to
//! thread an inherited direction through their component surfaces (or derive it from app/theme
//! configuration) and use `use_direction(...)` to apply the Radix resolution rule.

pub use fret_ui::overlay_placement::LayoutDirection;

use fret_core::KeyCode;
use fret_ui::{ElementContext, UiHost};

/// Resolve direction using the Radix rule: `local || inherited || Ltr`.
pub fn use_direction(
    local: Option<LayoutDirection>,
    inherited: Option<LayoutDirection>,
) -> LayoutDirection {
    local.or(inherited).unwrap_or_default()
}

/// Returns the nearest inherited direction from the current element scope stack.
///
/// This models the observable outcome of Radix `DirectionProvider` + `useDirection()`, without
/// requiring a dedicated runtime context system.
pub fn inherited_direction<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<LayoutDirection> {
    cx.provided::<LayoutDirection>().copied()
}

/// Runs `f` with `dir` installed as the current inherited direction for the subtree.
///
/// Nested calls restore the previous direction on exit.
#[track_caller]
pub fn with_direction_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    dir: LayoutDirection,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    cx.provide(dir, f)
}

/// Resolve direction from an optional local override plus any inherited provider.
pub fn use_direction_in_scope<H: UiHost>(
    cx: &ElementContext<'_, H>,
    local: Option<LayoutDirection>,
) -> LayoutDirection {
    use_direction(local, inherited_direction(cx))
}

/// Returns whether a horizontal arrow key means "move forward" in logical item order.
///
/// This mirrors Radix/Base UI component behavior: in RTL, ArrowLeft advances and ArrowRight moves
/// backward. Non-horizontal keys return `None`.
pub fn horizontal_forward_for_key(key: KeyCode, dir: LayoutDirection) -> Option<bool> {
    match (key, dir) {
        (KeyCode::ArrowRight, LayoutDirection::Ltr) => Some(true),
        (KeyCode::ArrowLeft, LayoutDirection::Ltr) => Some(false),
        (KeyCode::ArrowRight, LayoutDirection::Rtl) => Some(false),
        (KeyCode::ArrowLeft, LayoutDirection::Rtl) => Some(true),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorizontalVisualItemPosition {
    pub is_visual_first: bool,
    pub is_visual_last: bool,
    pub order: Option<i32>,
}

/// Computes visual first/last and optional flex order for a horizontal row.
///
/// Until Fret's Flex mechanism has a global RTL physical placement contract, components that need
/// DOM-like horizontal RTL visual order can use this helper to keep the policy centralized.
#[inline]
pub fn horizontal_visual_item_position(
    dir: LayoutDirection,
    idx: usize,
    len: usize,
) -> HorizontalVisualItemPosition {
    debug_assert!(len > 0, "horizontal_visual_item_position requires len > 0");

    match dir {
        LayoutDirection::Ltr => HorizontalVisualItemPosition {
            is_visual_first: idx == 0,
            is_visual_last: idx + 1 == len,
            order: None,
        },
        LayoutDirection::Rtl => HorizontalVisualItemPosition {
            is_visual_first: idx + 1 == len,
            is_visual_last: idx == 0,
            order: Some((len - 1 - idx) as i32),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};

    fn bounds() -> Rect {
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)))
    }

    #[test]
    fn direction_provider_inherits_and_restores() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            assert_eq!(inherited_direction(cx), None);
            assert_eq!(use_direction_in_scope(cx, None), LayoutDirection::Ltr);

            with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                assert_eq!(inherited_direction(cx), Some(LayoutDirection::Rtl));
                assert_eq!(use_direction_in_scope(cx, None), LayoutDirection::Rtl);

                cx.scope(|cx| {
                    assert_eq!(use_direction_in_scope(cx, None), LayoutDirection::Rtl);
                    with_direction_provider(cx, LayoutDirection::Ltr, |cx| {
                        assert_eq!(use_direction_in_scope(cx, None), LayoutDirection::Ltr);
                    });
                    assert_eq!(use_direction_in_scope(cx, None), LayoutDirection::Rtl);
                });
            });

            assert_eq!(inherited_direction(cx), None);
            assert_eq!(use_direction_in_scope(cx, None), LayoutDirection::Ltr);
        });
    }

    #[test]
    fn horizontal_forward_for_key_flips_arrow_semantics_in_rtl() {
        assert_eq!(
            horizontal_forward_for_key(KeyCode::ArrowRight, LayoutDirection::Ltr),
            Some(true)
        );
        assert_eq!(
            horizontal_forward_for_key(KeyCode::ArrowLeft, LayoutDirection::Ltr),
            Some(false)
        );
        assert_eq!(
            horizontal_forward_for_key(KeyCode::ArrowRight, LayoutDirection::Rtl),
            Some(false)
        );
        assert_eq!(
            horizontal_forward_for_key(KeyCode::ArrowLeft, LayoutDirection::Rtl),
            Some(true)
        );
    }

    #[test]
    fn horizontal_visual_item_position_marks_visual_edges_and_order() {
        assert_eq!(
            horizontal_visual_item_position(LayoutDirection::Ltr, 0, 3),
            HorizontalVisualItemPosition {
                is_visual_first: true,
                is_visual_last: false,
                order: None,
            }
        );
        assert_eq!(
            horizontal_visual_item_position(LayoutDirection::Ltr, 2, 3),
            HorizontalVisualItemPosition {
                is_visual_first: false,
                is_visual_last: true,
                order: None,
            }
        );
        assert_eq!(
            horizontal_visual_item_position(LayoutDirection::Rtl, 0, 3),
            HorizontalVisualItemPosition {
                is_visual_first: false,
                is_visual_last: true,
                order: Some(2),
            }
        );
        assert_eq!(
            horizontal_visual_item_position(LayoutDirection::Rtl, 2, 3),
            HorizontalVisualItemPosition {
                is_visual_first: true,
                is_visual_last: false,
                order: Some(0),
            }
        );
    }
}
