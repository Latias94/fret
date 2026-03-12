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
}
