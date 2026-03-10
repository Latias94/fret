use std::sync::Arc;

use fret_core::{
    FontId, FontWeight, Px, TextAlign, TextOverflow, TextStyle, TextStyleRefinement, TextWrap,
};
use fret_ui::element::{AnyElement, LayoutStyle, TextInkOverflow, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::typography as ui_typography;
use crate::typography::UiTextSize;

pub(crate) fn text_xs_style(theme: &Theme) -> TextStyle {
    ui_typography::control_text_style(theme, UiTextSize::Xs)
}

pub(crate) fn text_sm_style(theme: &Theme) -> TextStyle {
    ui_typography::control_text_style(theme, UiTextSize::Sm)
}

pub(crate) fn text_base_style(theme: &Theme) -> TextStyle {
    ui_typography::control_text_style(theme, UiTextSize::Base)
}

pub(crate) fn text_prose_style(theme: &Theme) -> TextStyle {
    ui_typography::control_text_style(theme, UiTextSize::Prose)
}

pub(crate) fn text_xs_refinement(theme: &Theme) -> TextStyleRefinement {
    ui_typography::composable_refinement_from_style(&text_xs_style(theme))
}

pub(crate) fn text_sm_refinement(theme: &Theme) -> TextStyleRefinement {
    ui_typography::composable_refinement_from_style(&text_sm_style(theme))
}

pub(crate) fn text_base_refinement(theme: &Theme) -> TextStyleRefinement {
    ui_typography::composable_refinement_from_style(&text_base_style(theme))
}

pub(crate) fn text_prose_refinement(theme: &Theme) -> TextStyleRefinement {
    ui_typography::composable_refinement_from_style(&text_prose_style(theme))
}

fn scoped_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
    refinement: TextStyleRefinement,
    wrap: TextWrap,
    overflow: TextOverflow,
) -> AnyElement {
    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: text.into(),
            style: None,
            color: None,
            wrap,
            overflow,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Declarative text helper that matches Tailwind's `truncate` semantics:
/// - `whitespace-nowrap`
/// - `text-overflow: ellipsis`
///
/// Note: ellipsis only applies when the text is laid out with a constrained width.
pub fn text_truncate<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: None,
        color: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    })
}

/// Declarative text helper that matches Tailwind's `whitespace-nowrap` semantics.
pub fn text_nowrap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: None,
        color: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    })
}

/// Declarative text helper that matches Tailwind's `text-sm` default usage in shadcn recipes.
///
/// Note: We intentionally map `font.size` to the "sm" baseline by default (editor-friendly).
/// Themes can override this via:
/// - `component.text.sm_px`
/// - `component.text.sm_line_height`
#[track_caller]
pub fn text_sm<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_sm_refinement(theme)
    };
    scoped_text(cx, text, refinement, TextWrap::Word, TextOverflow::Clip)
}

/// Declarative text helper that matches Tailwind's `text-xs` default usage in shadcn recipes.
///
/// Themes can override this via:
/// - `component.text.xs_px`
/// - `component.text.xs_line_height`
#[track_caller]
pub fn text_xs<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_xs_refinement(theme)
    };
    scoped_text(cx, text, refinement, TextWrap::Word, TextOverflow::Clip)
}

/// Declarative text helper that matches Tailwind's `text-base` default usage in shadcn recipes.
///
/// Themes can override this via:
/// - `component.text.base_px`
/// - `component.text.base_line_height`
pub fn text_base<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_base_refinement(theme)
    };
    scoped_text(cx, text, refinement, TextWrap::Word, TextOverflow::Clip)
}

/// Declarative text helper intended for typography pages (`prose`-like body copy).
///
/// This uses a larger baseline than `text_base` so examples like `typography-table` can match
/// upstream web goldens (16px / 24px by default under the shadcn theme).
///
/// Wrapping notes:
/// - This defaults to `TextWrap::Word` (wrap at word boundaries; do not break long tokens).
/// - For body copy that may contain long URLs/paths/identifiers, prefer [`text_prose_break_words`]
///   so a single token cannot force horizontal overflow.
/// - For editor-like surfaces that must always wrap even within tokens, prefer `TextWrap::Grapheme`.
/// - `WordBreak`/`Grapheme` behave best when the parent provides a definite width (`w_full`,
///   `Length::Fill`, `max_w`, etc.); in shrink-wrapped layouts they can legitimately measure
///   narrower under min-content constraints.
pub fn text_prose<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_prose_refinement(theme)
    };
    scoped_text(cx, text, refinement, TextWrap::Word, TextOverflow::Clip)
}

/// `text_prose` variant that matches Tailwind's `break-words` intent:
/// prefer wrapping at word boundaries, but allow breaking long tokens when needed.
pub fn text_prose_break_words<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_prose_refinement(theme)
    };
    scoped_text(
        cx,
        text,
        refinement,
        TextWrap::WordBreak,
        TextOverflow::Clip,
    )
}

/// Bold variant of [`text_prose`], intended for typography table headers (`<th className="... font-bold">`).
pub fn text_prose_bold<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let mut refinement = {
        let theme = Theme::global(&*cx.app);
        text_prose_refinement(theme)
    };
    refinement.weight = Some(FontWeight::BOLD);

    scoped_text(cx, text, refinement, TextWrap::Word, TextOverflow::Clip)
}

/// Returns the default label style and line-height baseline used by `primitives::label`.
pub(crate) fn label_style(theme: &Theme) -> (TextStyle, Px) {
    let px = theme
        .metric_by_key("component.label.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.label.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));

    let mut style = ui_typography::fixed_line_box_style(FontId::ui(), px, line_height);
    style.weight = FontWeight::MEDIUM;
    (style, line_height)
}

pub(crate) fn label_text_refinement(theme: &Theme) -> (TextStyleRefinement, Px) {
    let (style, line_height) = label_style(theme);
    let mut refinement = ui_typography::composable_refinement_from_style(&style);
    refinement.font = Some(FontId::ui());
    (refinement, line_height)
}

/// Declarative helper intended for code-like inline text.
///
/// Defaults:
/// - monospace font (`metric.font.mono_size` / `metric.font.mono_line_height`)
/// - `TextWrap::Grapheme` so long tokens (paths/URLs/identifiers) can still wrap when needed
pub fn text_code_wrap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        ui_typography::composable_refinement_from_style(&ui_typography::fixed_line_box_style(
            FontId::monospace(),
            theme.metric_token("metric.font.mono_size"),
            theme.metric_token("metric.font.mono_line_height"),
        ))
    };

    scoped_text(cx, text, refinement, TextWrap::Grapheme, TextOverflow::Clip)
}

/// `text_prose` variant that forces single-line layout (`whitespace-nowrap`-like behavior).
pub fn text_prose_nowrap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_prose_refinement(theme)
    };
    scoped_text(cx, text, refinement, TextWrap::None, TextOverflow::Clip)
}

/// Bold variant of [`text_prose_nowrap`].
pub fn text_prose_bold_nowrap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let mut refinement = {
        let theme = Theme::global(&*cx.app);
        text_prose_refinement(theme)
    };
    refinement.weight = Some(FontWeight::BOLD);

    scoped_text(cx, text, refinement, TextWrap::None, TextOverflow::Clip)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::ElementKind;
    use fret_ui::elements;
    use fret_ui::{Theme, ThemeConfig};

    fn test_app() -> App {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Text Helpers Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("font.size".to_string(), 13.0),
                    ("font.line_height".to_string(), 20.0),
                    (
                        crate::theme_tokens::metric::COMPONENT_TEXT_XS_PX.to_string(),
                        12.0,
                    ),
                    (
                        crate::theme_tokens::metric::COMPONENT_TEXT_XS_LINE_HEIGHT.to_string(),
                        16.0,
                    ),
                    (
                        crate::theme_tokens::metric::COMPONENT_TEXT_SM_PX.to_string(),
                        13.0,
                    ),
                    (
                        crate::theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT.to_string(),
                        18.0,
                    ),
                    (
                        crate::theme_tokens::metric::COMPONENT_TEXT_BASE_PX.to_string(),
                        14.0,
                    ),
                    (
                        crate::theme_tokens::metric::COMPONENT_TEXT_BASE_LINE_HEIGHT.to_string(),
                        20.0,
                    ),
                    (
                        crate::theme_tokens::metric::COMPONENT_TEXT_PROSE_PX.to_string(),
                        16.0,
                    ),
                    (
                        crate::theme_tokens::metric::COMPONENT_TEXT_PROSE_LINE_HEIGHT.to_string(),
                        24.0,
                    ),
                    ("metric.font.mono_size".to_string(), 13.0),
                    ("metric.font.mono_line_height".to_string(), 18.0),
                ]),
                ..ThemeConfig::default()
            });
        });
        app
    }

    fn test_bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(160.0)),
        )
    }

    #[test]
    fn text_sm_scopes_inherited_refinement_without_leaf_style() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el =
            elements::with_element_cx(&mut app, window, bounds, "test", |cx| text_sm(cx, "Hello"));
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_sm(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(el.inherited_text_style, Some(text_sm_refinement(&theme)));
    }

    #[test]
    fn prose_variants_and_code_wrap_install_semantic_inherited_overrides() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();
        let mut expected_prose = {
            let theme = Theme::global(&app);
            text_prose_refinement(theme)
        };
        expected_prose.weight = Some(FontWeight::BOLD);

        let prose_bold = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_prose_bold(cx, "Heading")
        });
        let ElementKind::Text(props) = &prose_bold.kind else {
            panic!("expected text_prose_bold(...) to build a Text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(prose_bold.inherited_text_style, Some(expected_prose));

        let code = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_code_wrap(cx, "let answer = 42;")
        });
        let ElementKind::Text(props) = &code.kind else {
            panic!("expected text_code_wrap(...) to build a Text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.wrap, TextWrap::Grapheme);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(
            code.inherited_text_style
                .as_ref()
                .and_then(|style| style.font.clone()),
            Some(FontId::monospace())
        );
    }
}
