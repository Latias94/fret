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

fn text_sm_style(theme: &Theme) -> TextStyle {
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

fn text_base_style(theme: &Theme) -> TextStyle {
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
