use std::sync::Arc;

use fret_core::{FontId, Px, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, LayoutStyle, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::theme_tokens;

fn font_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("font.size")
        .unwrap_or_else(|| theme.metric_required("font.size"))
}

fn font_line_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("font.line_height")
        .unwrap_or_else(|| theme.metric_required("font.line_height"))
}

pub(crate) fn text_sm_style(theme: &Theme) -> TextStyle {
    let size = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX)
        .unwrap_or_else(|| font_size(theme));
    let line_height = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
        .unwrap_or_else(|| font_line_height(theme));

    TextStyle {
        font: FontId::default(),
        size,
        line_height: Some(line_height),
        ..Default::default()
    }
}

pub(crate) fn text_xs_style(theme: &Theme) -> TextStyle {
    let size = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_XS_PX)
        .unwrap_or_else(|| Px(font_size(theme).0 - 2.0));
    let line_height = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_XS_LINE_HEIGHT)
        .unwrap_or_else(|| Px(font_line_height(theme).0 - 4.0));

    TextStyle {
        font: FontId::default(),
        size,
        line_height: Some(line_height),
        ..Default::default()
    }
}

pub(crate) fn text_base_style(theme: &Theme) -> TextStyle {
    let size = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_BASE_PX)
        .unwrap_or_else(|| Px(font_size(theme).0 + 1.0));
    let line_height = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_BASE_LINE_HEIGHT)
        .unwrap_or_else(|| font_line_height(theme));

    TextStyle {
        font: FontId::default(),
        size,
        line_height: Some(line_height),
        ..Default::default()
    }
}

pub(crate) fn text_prose_style(theme: &Theme) -> TextStyle {
    let size = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_PROSE_PX)
        .unwrap_or_else(|| Px(font_size(theme).0 + 2.0));
    let line_height = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_PROSE_LINE_HEIGHT)
        .unwrap_or_else(|| Px(font_line_height(theme).0 + 4.0));

    TextStyle {
        font: FontId::default(),
        size,
        line_height: Some(line_height),
        ..Default::default()
    }
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
    })
}

/// Declarative text helper that matches Tailwind's `text-sm` default usage in shadcn recipes.
///
/// Note: We intentionally map `font.size` to the "sm" baseline by default (editor-friendly).
/// Themes can override this via:
/// - `component.text.sm_px`
/// - `component.text.sm_line_height`
pub fn text_sm<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(text_sm_style(&theme)),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}

/// Declarative text helper that matches Tailwind's `text-xs` default usage in shadcn recipes.
///
/// Themes can override this via:
/// - `component.text.xs_px`
/// - `component.text.xs_line_height`
pub fn text_xs<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(text_xs_style(&theme)),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
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
    let theme = Theme::global(&*cx.app).clone();
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(text_base_style(&theme)),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}

/// Declarative text helper intended for typography pages (`prose`-like body copy).
///
/// This uses a larger baseline than `text_base` so examples like `typography-table` can match
/// upstream web goldens (16px / 24px by default under the shadcn theme).
pub fn text_prose<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(text_prose_style(&theme)),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}

/// Bold variant of [`text_prose`], intended for typography table headers (`<th className=\"... font-bold\">`).
pub fn text_prose_bold<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let mut style = text_prose_style(&theme);
    style.weight = fret_core::FontWeight::BOLD;

    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
    })
}

/// Returns the default label style and line-height baseline used by `primitives::label`.
pub(crate) fn label_style(theme: &Theme) -> (TextStyle, Px) {
    let px = theme
        .metric_by_key("component.label.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.label.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));

    (
        TextStyle {
            font: FontId::default(),
            size: px,
            weight: fret_core::FontWeight::MEDIUM,
            slant: Default::default(),
            line_height: Some(line_height),
            letter_spacing_em: None,
        },
        line_height,
    )
}

/// `text_prose` variant that forces single-line layout (`whitespace-nowrap`-like behavior).
pub fn text_prose_nowrap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(text_prose_style(&theme)),
        color: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    })
}

/// Bold variant of [`text_prose_nowrap`].
pub fn text_prose_bold_nowrap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let mut style = text_prose_style(&theme);
    style.weight = fret_core::FontWeight::BOLD;

    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    })
}
