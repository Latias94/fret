//! Portal inherited context helpers (Radix-aligned outcomes).
//!
//! In Radix/React, context providers (e.g. `DirectionProvider`) propagate through portals.
//! Fret's lightweight provider pattern is implemented via `ElementContext::inherited_state_*`,
//! which searches the current element scope stack.
//!
//! `ElementContext::with_root_name(...)` resets the scope stack to a new global root. That is the
//! correct mechanism for overlays/portals, but it also means inherited provider state is no longer
//! discoverable unless it is explicitly re-installed in the portal root.
//!
//! This module provides a small, explicit bridge: capture the relevant inherited outcomes at the
//! callsite and re-install them inside the portal root.
//!
//! Today this only captures layout direction (`DirectionProvider`). We intentionally keep this
//! narrow to avoid accidentally coupling unrelated provider surfaces across overlay roots.

use super::direction;
use super::direction::LayoutDirection;
use fret_ui::{ElementContext, UiHost};

/// Captured provider outcomes that should be consistent inside a portal root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortalInherited {
    pub direction: LayoutDirection,
}

impl PortalInherited {
    /// Capture inherited values from the current element scope stack.
    pub fn capture<H: UiHost>(cx: &ElementContext<'_, H>) -> Self {
        Self {
            direction: direction::use_direction_in_scope(cx, None),
        }
    }

    /// Re-install the captured inherited values for the duration of `f`.
    #[track_caller]
    pub fn install<H: UiHost, R>(
        self,
        cx: &mut ElementContext<'_, H>,
        f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
    ) -> R {
        direction::with_direction_provider(cx, self.direction, f)
    }
}

/// Run `f` inside a portal root name scope while preserving captured inherited values.
#[track_caller]
pub fn with_root_name_inheriting<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    root_name: &str,
    inherited: PortalInherited,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    cx.with_root_name(root_name, |cx| inherited.install(cx, f))
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
    fn portal_inherited_reinstalls_direction_across_root_name() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            assert_eq!(
                direction::use_direction_in_scope(cx, None),
                LayoutDirection::Ltr
            );

            direction::with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
                assert_eq!(
                    direction::use_direction_in_scope(cx, None),
                    LayoutDirection::Rtl
                );
                let inherited = PortalInherited::capture(cx);
                assert_eq!(inherited.direction, LayoutDirection::Rtl);

                with_root_name_inheriting(cx, "portal", inherited, |cx| {
                    assert_eq!(
                        direction::use_direction_in_scope(cx, None),
                        LayoutDirection::Rtl
                    );
                });

                assert_eq!(
                    direction::use_direction_in_scope(cx, None),
                    LayoutDirection::Rtl
                );
            });

            assert_eq!(
                direction::use_direction_in_scope(cx, None),
                LayoutDirection::Ltr
            );
        });
    }
}
