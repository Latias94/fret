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
/// layout subtree (v2).
///
/// Prefer this helper when you already have a concrete layout root (for example a row/column/
/// container) and want to avoid accidentally treating `ForegroundScope` like a layout fragment.
#[track_caller]
pub fn scope_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: ColorRef,
    child: fret_ui::element::AnyElement,
) -> fret_ui::element::AnyElement {
    let theme = Theme::global(&*cx.app);
    let fg = color.resolve(theme);
    cx.foreground_scope(fg, move |_cx| vec![child])
}

/// Returns a foreground scope wrapper around the children returned by `f`.
///
/// Important: `ForegroundScope` is paint-only and input-transparent, but it is **not** a layout
/// fragment. When `f` returns multiple siblings, they are laid out inside the wrapper's own
/// passthrough box rather than participating directly in the parent flow. Callers that need normal
/// row/column flow should first build an explicit layout root and then use [`scope_element`].
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
    vec![cx.foreground_scope(fg, f)]
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
}
