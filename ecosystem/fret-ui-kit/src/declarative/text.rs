use std::sync::Arc;

use fret_core::{FontId, Px, TextAlign, TextOverflow, TextStyle, TextWrap};
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
    let style = {
        let theme = Theme::global(&*cx.app);
        text_sm_style(theme)
    };
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    })
}

/// Declarative text helper that matches Tailwind's `text-xs` default usage in shadcn recipes.
///
/// Themes can override this via:
/// - `component.text.xs_px`
/// - `component.text.xs_line_height`
#[track_caller]
pub fn text_xs<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let style = {
        let theme = Theme::global(&*cx.app);
        text_xs_style(theme)
    };
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
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
    let style = {
        let theme = Theme::global(&*cx.app);
        text_base_style(theme)
    };
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    })
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
    let style = {
        let theme = Theme::global(&*cx.app);
        text_prose_style(theme)
    };
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    })
}

/// `text_prose` variant that matches Tailwind's `break-words` intent:
/// prefer wrapping at word boundaries, but allow breaking long tokens when needed.
pub fn text_prose_break_words<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let style = {
        let theme = Theme::global(&*cx.app);
        text_prose_style(theme)
    };
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::WordBreak,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    })
}

/// Bold variant of [`text_prose`], intended for typography table headers (`<th className=\"... font-bold\">`).
pub fn text_prose_bold<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let mut style = {
        let theme = Theme::global(&*cx.app);
        text_prose_style(theme)
    };
    style.weight = fret_core::FontWeight::BOLD;

    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    })
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
    style.weight = fret_core::FontWeight::MEDIUM;
    (style, line_height)
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
    let style = {
        let theme = Theme::global(&*cx.app);
        ui_typography::fixed_line_box_style(
            FontId::monospace(),
            theme.metric_token("metric.font.mono_size"),
            theme.metric_token("metric.font.mono_line_height"),
        )
    };

    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::Grapheme,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    })
}

/// `text_prose` variant that forces single-line layout (`whitespace-nowrap`-like behavior).
pub fn text_prose_nowrap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let style = {
        let theme = Theme::global(&*cx.app);
        text_prose_style(theme)
    };
    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    })
}

/// Bold variant of [`text_prose_nowrap`].
pub fn text_prose_bold_nowrap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let mut style = {
        let theme = Theme::global(&*cx.app);
        text_prose_style(theme)
    };
    style.weight = fret_core::FontWeight::BOLD;

    cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: text.into(),
        style: Some(style),
        color: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    })
}
