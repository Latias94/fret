//! shadcn/ui `direction` helpers (v4).
//!
//! Upstream source of truth:
//! - `repo-ref/ui/apps/v4/registry/bases/radix/ui/direction.tsx`
//!
//! In the DOM implementation this is a thin wrapper over Radix `DirectionProvider` +
//! `useDirection()`. In Fret we model the observable outcome via inherited state in
//! `fret-ui-kit::primitives::direction`.

use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::primitives::direction as direction_prim;

pub use direction_prim::LayoutDirection;

/// shadcn/ui `DirectionProvider` (v4).
///
/// This does not produce a visual element; it installs direction as inherited state for the
/// subtree built by `children`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectionProvider {
    dir: LayoutDirection,
}

impl DirectionProvider {
    pub fn new(dir: LayoutDirection) -> Self {
        Self { dir }
    }

    /// Runs `children` with `dir` installed as the inherited direction for the subtree.
    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> fret_ui::element::AnyElement,
    ) -> fret_ui::element::AnyElement {
        direction_prim::with_direction_provider(cx, self.dir, children)
    }
}

/// shadcn/ui `useDirection` (v4).
///
/// Resolves the current direction using the Radix rule: `local || inherited || Ltr`.
pub fn use_direction<H: UiHost>(
    cx: &ElementContext<'_, H>,
    local: Option<LayoutDirection>,
) -> LayoutDirection {
    direction_prim::use_direction_in_scope(cx, local)
}

/// Fret convenience wrapper over the Radix-style `DirectionProvider`.
///
/// This mirrors the `with_direction_provider(...)` helper in `fret-ui-kit` so gallery snippets and
/// recipes don't need to reach into the primitives layer directly.
#[track_caller]
pub fn with_direction_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    dir: LayoutDirection,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    direction_prim::with_direction_provider(cx, dir, f)
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
    fn direction_provider_installs_inherited_direction_for_children() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "dir_provider", |cx| {
            assert_eq!(use_direction(cx, None), LayoutDirection::Ltr);

            DirectionProvider::new(LayoutDirection::Rtl).into_element(cx, |cx| {
                assert_eq!(use_direction(cx, None), LayoutDirection::Rtl);
                cx.text("rtl")
            });

            assert_eq!(use_direction(cx, None), LayoutDirection::Ltr);
        });
    }
}
