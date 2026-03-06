//! `currentColor` / `IconTheme`-style foreground inheritance for declarative UI.
//!
//! Motivation:
//! - Web (shadcn/Radix) commonly relies on CSS `currentColor` so icons inherit text color.
//! - Flutter provides `IconTheme` / `DefaultTextStyle` so descendants inherit foreground styling.
//!
//! Fret does not have a dedicated runtime context system yet, but `ElementContext` supports a
//! lightweight provider pattern via `inherited_state_*`. This module defines a small, opt-in
//! `currentColor` surface that components can install (e.g. buttons, menu items) so icons/spinners
//! can inherit the resolved foreground color without each callsite manually threading tokens.

use crate::ColorRef;
use fret_ui::Theme;
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Default)]
struct CurrentColorProviderState {
    current: Option<ColorRef>,
}

/// Returns the nearest inherited `currentColor` for the current element scope stack.
///
/// When unset, callers should apply their own fallback (typically a theme token).
pub fn inherited_current_color<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<ColorRef> {
    cx.inherited_state_where::<CurrentColorProviderState>(|st| st.current.is_some())
        .and_then(|st| st.current.clone())
}

/// Runs `f` with `color` installed as the inherited `currentColor` for the subtree.
///
/// Nested calls restore the previous value on exit.
#[track_caller]
pub fn with_current_color_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    color: ColorRef,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(CurrentColorProviderState::default, |st| {
        let prev = st.current.clone();
        st.current = Some(color.clone());
        prev
    });
    let out = f(cx);
    cx.with_state(CurrentColorProviderState::default, |st| {
        st.current = prev;
    });
    out
}

/// Returns a wrapper element that installs `color` as the inherited foreground for one explicit
/// layout subtree without introducing a layout wrapper (v2).
///
/// This stamps the returned subtree root directly, so fill/shrink/sibling flow semantics remain
/// owned by the original element.
#[track_caller]
pub fn scope_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: ColorRef,
    child: fret_ui::element::AnyElement,
) -> fret_ui::element::AnyElement {
    let theme = Theme::global(&*cx.app);
    let fg = color.resolve(theme);
    child.inherit_foreground(fg)
}

/// Returns the children from `f` with `color` stamped as their inherited foreground.
///
/// This preserves normal sibling participation in parent layout because it does not insert an
/// extra wrapper node. The closure also runs with `color` installed as the authoring-time
/// `currentColor` provider so nested builders can read it via [`inherited_current_color`].
#[track_caller]
pub fn scope_children<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    color: ColorRef,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> Vec<fret_ui::element::AnyElement>
where
    I: IntoIterator<Item = fret_ui::element::AnyElement>,
{
    let theme = Theme::global(&*cx.app);
    let fg = color.resolve(theme);
    with_current_color_provider(cx, color, |cx| {
        f(cx)
            .into_iter()
            .map(|child| child.inherit_foreground(fg))
            .collect()
    })
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
    fn current_color_provider_inherits_and_restores() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            assert!(inherited_current_color(cx).is_none());

            let red = ColorRef::Color(fret_core::Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            });
            let blue = ColorRef::Color(fret_core::Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            });

            with_current_color_provider(cx, red.clone(), |cx| {
                assert!(matches!(
                    inherited_current_color(cx),
                    Some(ColorRef::Color(_))
                ));

                cx.scope(|cx| {
                    assert!(inherited_current_color(cx).is_some());
                    with_current_color_provider(cx, blue, |cx| {
                        assert!(inherited_current_color(cx).is_some());
                    });
                    assert!(inherited_current_color(cx).is_some());
                });
            });

            assert!(inherited_current_color(cx).is_none());
        });
    }

    #[test]
    fn scope_element_stamps_inherited_foreground_without_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let expected = fret_core::Color {
                r: 0.25,
                g: 0.5,
                b: 0.75,
                a: 1.0,
            };
            let child = cx.text("hello");

            let el = scope_element(cx, ColorRef::Color(expected), child);

            assert!(matches!(el.kind, fret_ui::element::ElementKind::Text(_)));
            assert_eq!(
                el.inherited_foreground,
                Some(expected),
                "expected scope_element(...) to stamp inherited foreground on the existing root"
            );
        });
    }

    #[test]
    fn scope_children_stamps_each_root_and_installs_provider() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let expected = fret_core::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            };

            let els = scope_children(cx, ColorRef::Color(expected), |cx| {
                let inherited = inherited_current_color(cx);
                assert!(matches!(
                    inherited,
                    Some(ColorRef::Color(color)) if color == expected
                ));

                [cx.text("a"), cx.text("b")]
            });

            assert_eq!(els.len(), 2, "expected sibling count to be preserved");
            assert!(
                els.iter()
                    .all(|el| matches!(el.kind, fret_ui::element::ElementKind::Text(_)))
            );
            assert!(
                els.iter()
                    .all(|el| el.inherited_foreground == Some(expected))
            );
        });
    }
}
