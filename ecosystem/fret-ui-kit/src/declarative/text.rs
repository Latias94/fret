use std::sync::Arc;

use fret_core::{
    AttributedText, FontId, FontWeight, Px, TextAlign, TextFontFeatureSetting, TextOverflow,
    TextStyle, TextStyleRefinement, TextWrap,
};
use fret_ui::element::{
    AnyElement, LayoutStyle, Length, StyledTextProps, TextInkOverflow, TextProps,
};
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

pub(crate) fn text_button_label_refinement(theme: &Theme) -> TextStyleRefinement {
    let mut refinement = text_sm_refinement(theme);
    refinement.weight = Some(FontWeight::MEDIUM);
    refinement
}

pub(crate) fn text_button_label_compact_refinement(theme: &Theme) -> TextStyleRefinement {
    let mut refinement = text_xs_refinement(theme);
    refinement.weight = Some(FontWeight::MEDIUM);
    refinement
}

pub(crate) fn text_chrome_title_refinement(theme: &Theme) -> TextStyleRefinement {
    let mut refinement = text_sm_refinement(theme);
    refinement.weight = Some(FontWeight::MEDIUM);
    refinement
}

pub(crate) fn text_table_cell_emphasis_refinement(theme: &Theme) -> TextStyleRefinement {
    let mut refinement = text_sm_refinement(theme);
    refinement.weight = Some(FontWeight::MEDIUM);
    refinement
}

pub(crate) fn text_control_readout_tabular_refinement(theme: &Theme) -> TextStyleRefinement {
    let mut refinement = text_sm_refinement(theme);
    refinement.features.push(TextFontFeatureSetting {
        tag: "tnum".into(),
        value: 1,
    });
    refinement
}

pub(crate) fn text_control_readout_tabular_emphasis_refinement(
    theme: &Theme,
) -> TextStyleRefinement {
    let mut refinement = text_control_readout_tabular_refinement(theme);
    refinement.weight = Some(FontWeight::MEDIUM);
    refinement
}

pub(crate) fn text_control_readout_compact_tabular_emphasis_refinement(
    theme: &Theme,
) -> TextStyleRefinement {
    let mut refinement = text_xs_refinement(theme);
    refinement.weight = Some(FontWeight::MEDIUM);
    refinement.features.push(TextFontFeatureSetting {
        tag: "tnum".into(),
        value: 1,
    });
    refinement
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

fn shrinkable_single_line_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.flex.shrink = 1.0;
    layout.size.min_width = Some(Length::Px(Px(0.0)));
    layout
}

fn fill_shrinkable_single_line_layout() -> LayoutStyle {
    let mut layout = shrinkable_single_line_layout();
    layout.size.width = Length::Fill;
    layout
}

fn fill_growing_single_line_layout() -> LayoutStyle {
    fill_growing_zero_min_layout()
}

fn fill_growing_zero_min_layout() -> LayoutStyle {
    let mut layout = fill_shrinkable_single_line_layout();
    layout.flex.grow = 1.0;
    layout.flex.basis = Length::Px(Px(0.0));
    layout
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

/// Declarative text helper for dense table cells.
///
/// This keeps the compact `text-sm` baseline but forces single-line truncation so rows stay
/// visually stable under resize.
pub fn text_table_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_sm_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Declarative text helper for emphasized dense table cells.
///
/// Use this for row-identifying cells that need medium emphasis while retaining table-cell
/// single-line truncation under resize.
pub fn text_table_cell_emphasis<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_table_cell_emphasis_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Declarative text helper for list and command-row labels.
///
/// Use this for dense selectable/menu/tree rows. These labels fill the row's available inline
/// space, can shrink to zero in flex layouts, and truncate instead of increasing row height.
pub fn text_list_row_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_sm_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: fill_growing_zero_min_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Attributed-text variant of [`text_list_row_label`].
///
/// Use this for dense selectable/menu/tree rows that need per-span decoration, such as a completed
/// todo row with strikethrough. It keeps the same fill-width, shrinkable, single-line truncation
/// contract as plain list-row labels.
pub fn text_list_row_label_attributed<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    rich: AttributedText,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_sm_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.styled_text_props(StyledTextProps {
            layout: fill_growing_zero_min_layout(),
            rich,
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Declarative text helper for compact control readouts.
///
/// Use this for status/value text that sits next to dense controls in toolbars or editor panels.
/// It uses muted `text-xs` styling and single-line truncation so controls keep their row height
/// stable under resize.
pub fn text_control_readout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let (refinement, foreground) = {
        let theme = Theme::global(&*cx.app);
        (
            text_xs_refinement(theme),
            ui_typography::muted_foreground_color(theme),
        )
    };

    ui_typography::scope_text_style_with_color(
        cx.text_props(TextProps {
            layout: shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
        foreground,
    )
}

/// Declarative text helper for muted menu/select group labels.
///
/// Use this for non-interactive group headings inside fixed-height menu/listbox rows. These labels
/// keep the shadcn-like `text-xs` muted treatment while truncating instead of increasing row
/// height under resize.
pub fn text_menu_group_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let (refinement, foreground) = {
        let theme = Theme::global(&*cx.app);
        let mut refinement = text_xs_refinement(theme);
        refinement.weight = Some(FontWeight::MEDIUM);
        (refinement, ui_typography::muted_foreground_color(theme))
    };

    ui_typography::scope_text_style_with_color(
        cx.text_props(TextProps {
            layout: fill_shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
        foreground,
    )
}

/// Declarative text helper for non-interactive status messages.
///
/// Use this for compact empty/loading/status messages in list and command surfaces. It keeps a
/// muted `text-sm` treatment while truncating instead of increasing fixed-row height under resize.
pub fn text_status_message<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let (refinement, foreground) = {
        let theme = Theme::global(&*cx.app);
        (
            text_sm_refinement(theme),
            ui_typography::muted_foreground_color(theme),
        )
    };

    ui_typography::scope_text_style_with_color(
        cx.text_props(TextProps {
            layout: shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
        foreground,
    )
}

/// Declarative text helper for tabular control readouts.
///
/// Use this for data-table and dashboard counters that must keep stable digit widths while still
/// staying single-line, shrinkable, muted, and ellipsized under resize.
pub fn text_control_readout_tabular<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let (refinement, foreground) = {
        let theme = Theme::global(&*cx.app);
        (
            text_control_readout_tabular_refinement(theme),
            ui_typography::muted_foreground_color(theme),
        )
    };

    ui_typography::scope_text_style_with_color(
        cx.text_props(TextProps {
            layout: shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
        foreground,
    )
}

/// Declarative text helper for medium-weight tabular control readouts.
///
/// Use this for pagination and compact dashboard summaries where the count is primary within the
/// control row. It preserves the same single-line resize contract as numeric readouts.
pub fn text_control_readout_tabular_emphasis<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_control_readout_tabular_emphasis_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Declarative text helper for compact medium-weight tabular control readouts.
///
/// Use this for fixed badge/counter slots that match shadcn's
/// `text-xs font-medium tabular-nums` treatment. It stays in the control-readout family while
/// preserving the shared single-line shrink/ellipsis resize contract.
pub fn text_control_readout_compact_tabular_emphasis<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_control_readout_compact_tabular_emphasis_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Declarative text helper for compact button labels.
///
/// Button labels are intentionally single-line. In constrained toolbars/editor panels they should
/// truncate instead of increasing the button row height.
pub fn text_button_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_button_label_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Fill-width variant of [`text_button_label`].
///
/// Use this when a button-like row label owns the remaining inline space between icons/actions.
/// It stays in the button-label family while adding fill/grow/basis-zero flex behavior.
pub fn text_button_label_fill<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_button_label_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: fill_growing_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Compact fill-width variant of [`text_button_label`].
///
/// Use this for `text-xs font-medium` button-like triggers such as small sidebar menu rows.
pub fn text_button_label_compact_fill<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_button_label_compact_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: fill_growing_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Declarative text helper for compact control labels.
///
/// Use this for checkbox, radio, switch, combo, and slider captions that occupy remaining control
/// row space. These labels fill available width, shrink to zero in flex rows, and truncate instead
/// of increasing fixed control chrome height.
pub fn text_control_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_sm_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: fill_growing_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Declarative text helper for compact section and chrome labels.
///
/// Use this for separator labels, panel-section headings, and similar fixed chrome. These labels
/// should stay on one line and truncate under resize instead of increasing row height.
pub fn text_section_chrome_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_sm_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Declarative text helper for compact chrome glyphs.
///
/// Use this for disclosure arrows and similar glyph-only chrome inside fixed-size slots. Glyphs
/// stay single-line and clip instead of growing fixed chrome rows under resize.
pub fn text_chrome_glyph<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_sm_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Declarative text helper for fill-width chrome titles.
///
/// Use this for window/panel title bars that occupy remaining chrome row space. It keeps compact
/// medium-weight chrome text while opting into fill, grow, and `min-width: 0` layout.
pub fn text_chrome_title<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_chrome_title_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: fill_growing_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
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

/// Semantic alias for paragraph body copy.
///
/// `text_prose(...)` remains available for shadcn/Tailwind-style naming; new app/framework
/// surfaces should prefer this role name when they mean ordinary paragraph text.
pub fn text_paragraph<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    text_prose(cx, text)
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

/// Paragraph variant that prefers word boundaries but can break long tokens when needed.
pub fn text_paragraph_break_words<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    text_prose_break_words(cx, text)
}

/// Compact paragraph/body copy for dense editor surfaces.
///
/// This role is for explanatory copy that should wrap and may grow row height, but still needs
/// fill-width, `min-width: 0` flex behavior inside dense panels.
pub fn text_compact_paragraph<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_sm_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: fill_growing_zero_min_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Compact paragraph/body copy that inherits typography from an owning component.
///
/// Use this when the surrounding component already installs a description/body text refinement but
/// the leaf still needs the shared wrapping and fill-width resize contract.
pub fn text_compact_paragraph_inherited<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: fill_growing_zero_min_layout(),
        text: text.into(),
        style: None,
        color: None,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
    })
}

/// Compact paragraph/body copy clamped to a fixed number of dense text lines.
///
/// This role is for list/card descriptions where prose should wrap, but the surrounding item has a
/// fixed visual rhythm. Use plain [`text_compact_paragraph`] when the parent should grow to fit all
/// lines.
pub fn text_compact_paragraph_line_clamp<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
    max_lines: u16,
) -> AnyElement {
    let (refinement, line_height) = {
        let theme = Theme::global(&*cx.app);
        let style = text_sm_style(theme);
        let line_height = style
            .line_height
            .unwrap_or_else(|| theme.metric_token("font.line_height"));
        (
            ui_typography::composable_refinement_from_style(&style),
            line_height,
        )
    };

    let mut layout = fill_growing_zero_min_layout();
    layout.size.max_height = Some(Length::Px(Px(line_height.0 * max_lines.max(1) as f32)));

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout,
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
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

fn text_code_refinement(theme: &Theme) -> TextStyleRefinement {
    ui_typography::composable_refinement_from_style(&ui_typography::fixed_line_box_style(
        FontId::monospace(),
        theme.metric_token("metric.font.mono_size"),
        theme.metric_token("metric.font.mono_line_height"),
    ))
}

fn text_code_label_emphasis_refinement(theme: &Theme) -> TextStyleRefinement {
    let mut refinement = text_code_refinement(theme);
    refinement.weight = Some(FontWeight::MEDIUM);
    refinement
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
        text_code_refinement(theme)
    };

    scoped_text(cx, text, refinement, TextWrap::Grapheme, TextOverflow::Clip)
}

/// Declarative helper intended for code-like labels inside fixed-height chrome.
///
/// Use this for package names, env keys, dependency rows, and compact identifier labels that should
/// stay single-line under resize. Use [`text_code_wrap`] when inline code is allowed to wrap.
pub fn text_code_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_code_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Emphasized variant of [`text_code_label`].
///
/// Use this for primary identifier slots, such as package names or target versions, that need
/// medium emphasis while keeping code-label single-line truncation under resize.
pub fn text_code_label_emphasis<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_code_label_emphasis_refinement(theme)
    };

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout: shrinkable_single_line_layout(),
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
}

/// Declarative helper for block code rendered inside a scrollable/code surface.
///
/// This keeps code single-line per source line. The caller should provide horizontal scrolling or a
/// constrained container when long lines are possible.
pub fn text_code_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let refinement = {
        let theme = Theme::global(&*cx.app);
        text_code_refinement(theme)
    };

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;

    ui_typography::scope_text_style(
        cx.text_props(TextProps {
            layout,
            text: text.into(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: TextAlign::Start,
            ink_overflow: TextInkOverflow::None,
        }),
        refinement,
    )
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
    use fret_core::{
        AppWindowId, MaterialDescriptor, MaterialId, MaterialRegistrationError, PathCommand,
        PathConstraints, PathId, PathMetrics, PathService, Point, Rect, Size, SvgId, SvgService,
        TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
    };
    use fret_ui::element::ElementKind;
    use fret_ui::elements;
    use fret_ui::elements::GlobalElementId;
    use fret_ui::{Theme, ThemeConfig, UiTree, declarative};

    #[derive(Default)]
    struct WrappingTextServices;

    impl TextService for WrappingTextServices {
        fn prepare(
            &mut self,
            input: &TextInput,
            constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            let text = input.text();
            let char_width = Px(7.0);
            let line_height = Px(14.0);
            let char_count = text.chars().count().max(1);
            let unwrapped_width = Px(char_count as f32 * char_width.0);
            let lines = match (constraints.wrap, constraints.max_width) {
                (TextWrap::None, _) | (_, None) => 1usize,
                (_, Some(max_width)) if max_width.0 <= char_width.0 => char_count,
                (_, Some(max_width)) => {
                    let chars_per_line = (max_width.0 / char_width.0).floor().max(1.0) as usize;
                    char_count.div_ceil(chars_per_line)
                }
            };
            let width = match (constraints.overflow, constraints.max_width) {
                (TextOverflow::Ellipsis, Some(max_width)) => Px(unwrapped_width.0.min(max_width.0)),
                (_, Some(max_width)) if constraints.wrap != TextWrap::None => {
                    Px(unwrapped_width.0.min(max_width.0))
                }
                _ => unwrapped_width,
            };

            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(width, Px(lines as f32 * line_height.0)),
                    baseline: Px(line_height.0 * 0.8),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for WrappingTextServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: fret_core::PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for WrappingTextServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for WrappingTextServices {
        fn register_material(
            &mut self,
            _desc: MaterialDescriptor,
        ) -> Result<MaterialId, MaterialRegistrationError> {
            Err(MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: MaterialId) -> bool {
            false
        }
    }

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

    fn narrow_bounds() -> Rect {
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(96.0), Px(220.0)))
    }

    fn layout_text_role(
        role_name: &'static str,
        build: impl FnOnce(&mut ElementContext<'_, App>) -> AnyElement + 'static,
    ) -> Rect {
        let window = AppWindowId::default();
        let mut app = test_app();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let bounds = narrow_bounds();
        let mut services = WrappingTextServices;
        let text_id = std::sync::Arc::new(std::sync::Mutex::new(None::<GlobalElementId>));
        let text_id_for_render = std::sync::Arc::clone(&text_id);

        let root = declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            role_name,
            move |cx| {
                let text = build(cx);
                *text_id_for_render.lock().unwrap() = Some(text.id);
                vec![text]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let text_id = text_id.lock().unwrap().expect("text id");
        elements::current_bounds_for_element(&mut app, window, text_id).expect("text bounds")
    }

    fn assert_single_line_text_role(
        role_name: &'static str,
        line_height: Px,
        build: impl FnOnce(&mut ElementContext<'_, App>) -> AnyElement + 'static,
    ) {
        let bounds = layout_text_role(role_name, build);
        assert!(
            bounds.size.height.0 <= line_height.0 + 0.5,
            "{role_name} should stay one measured line under narrow resize, got {bounds:?}"
        );
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

        let code_label = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_code_label(cx, "pkg/runtime-with-a-long-name")
        });
        let ElementKind::Text(props) = &code_label.kind else {
            panic!("expected text_code_label(...) to build a Text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(
            code_label
                .inherited_text_style
                .as_ref()
                .and_then(|style| style.font.clone()),
            Some(FontId::monospace())
        );

        let code_label_emphasis =
            elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
                text_code_label_emphasis(cx, "pkg/runtime-with-a-long-name")
            });
        let ElementKind::Text(props) = &code_label_emphasis.kind else {
            panic!("expected text_code_label_emphasis(...) to build a Text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        let mut expected_code_emphasis = {
            let theme = Theme::global(&app);
            text_code_refinement(theme)
        };
        expected_code_emphasis.weight = Some(FontWeight::MEDIUM);
        assert_eq!(
            code_label_emphasis.inherited_text_style,
            Some(expected_code_emphasis)
        );

        let paragraph = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_paragraph(cx, "Paragraph body copy")
        });
        let ElementKind::Text(props) = &paragraph.kind else {
            panic!("expected text_paragraph(...) to build a Text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
        let expected_paragraph = {
            let theme = Theme::global(&app);
            text_prose_refinement(theme)
        };
        assert_eq!(
            paragraph.inherited_text_style,
            Some(expected_paragraph.clone())
        );

        let paragraph_break = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_paragraph_break_words(cx, "https://example.invalid/very/long/path")
        });
        let ElementKind::Text(props) = &paragraph_break.kind else {
            panic!("expected text_paragraph_break_words(...) to build a Text element");
        };
        assert_eq!(props.wrap, TextWrap::WordBreak);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(
            paragraph_break.inherited_text_style,
            Some(expected_paragraph)
        );

        let code_block = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_code_block(cx, "fn main() {}")
        });
        let ElementKind::Text(props) = &code_block.kind else {
            panic!("expected text_code_block(...) to build a Text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(
            code_block
                .inherited_text_style
                .as_ref()
                .and_then(|style| style.font.clone()),
            Some(FontId::monospace())
        );
    }

    #[test]
    fn compact_paragraph_text_uses_wrapping_fill_width_layout() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_compact_paragraph(cx, "Dense editor body copy wraps inside the available row")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_compact_paragraph(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(el.inherited_text_style, Some(text_sm_refinement(&theme)));
    }

    #[test]
    fn inherited_compact_paragraph_keeps_wrapping_layout_without_leaf_refinement() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_compact_paragraph_inherited(
                cx,
                "Component-owned description copy wraps inside the available row",
            )
        });

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_compact_paragraph_inherited(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(el.inherited_text_style, None);
    }

    #[test]
    fn compact_paragraph_line_clamp_text_uses_two_line_clamped_layout() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_compact_paragraph_line_clamp(
                cx,
                "Dense card description that may occupy at most two lines",
                2,
            )
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_compact_paragraph_line_clamp(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.layout.size.max_height, Some(Length::Px(Px(36.0))));
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(el.inherited_text_style, Some(text_sm_refinement(&theme)));
    }

    #[test]
    fn table_cell_text_uses_compact_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_table_cell(cx, "Compact table cell")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_table_cell(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(el.inherited_text_style, Some(text_sm_refinement(&theme)));
    }

    #[test]
    fn table_cell_emphasis_text_keeps_single_line_truncation_and_medium_weight() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_table_cell_emphasis(cx, "Primary table cell")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_table_cell_emphasis(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(
            el.inherited_text_style,
            Some(text_table_cell_emphasis_refinement(&theme))
        );
    }

    #[test]
    fn list_row_label_text_uses_fill_width_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_list_row_label(cx, "Open recent project")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_list_row_label(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(el.inherited_text_style, Some(text_sm_refinement(&theme)));
    }

    #[test]
    fn attributed_list_row_label_text_uses_fill_width_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let text = Arc::<str>::from("Completed task with decoration");
        let rich = AttributedText::new(
            Arc::clone(&text),
            Arc::<[fret_core::TextSpan]>::from([fret_core::TextSpan::new(text.len())]),
        );
        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_list_row_label_attributed(cx, rich)
        });
        let theme = Theme::global(&app);

        let ElementKind::StyledText(props) = &el.kind else {
            panic!("expected text_list_row_label_attributed(...) to build a StyledText element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(el.inherited_text_style, Some(text_sm_refinement(&theme)));
    }

    #[test]
    fn control_readout_text_uses_muted_compact_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_control_readout(cx, "Soft wrap: 80 cols")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_control_readout(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(el.inherited_text_style, Some(text_xs_refinement(&theme)));
        assert_eq!(
            el.inherited_foreground,
            Some(ui_typography::muted_foreground_color(theme))
        );
    }

    #[test]
    fn menu_group_label_text_uses_muted_medium_xs_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_menu_group_label(cx, "Recently opened projects")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_menu_group_label(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        let mut expected = text_xs_refinement(&theme);
        expected.weight = Some(FontWeight::MEDIUM);
        assert_eq!(el.inherited_text_style, Some(expected));
        assert_eq!(
            el.inherited_foreground,
            Some(ui_typography::muted_foreground_color(theme))
        );
    }

    #[test]
    fn status_message_text_uses_muted_sm_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_status_message(cx, "No results found")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_status_message(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(el.inherited_text_style, Some(text_sm_refinement(&theme)));
        assert_eq!(
            el.inherited_foreground,
            Some(ui_typography::muted_foreground_color(theme))
        );
    }

    #[test]
    fn control_readout_tabular_text_uses_muted_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_control_readout_tabular(cx, "128 of 4096")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_control_readout_tabular(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(
            el.inherited_text_style,
            Some(text_control_readout_tabular_refinement(&theme))
        );
        assert!(el.inherited_text_style.as_ref().is_some_and(|style| {
            style
                .features
                .iter()
                .any(|feature| feature.tag.as_str() == "tnum" && feature.value == 1)
        }));
        assert_eq!(
            el.inherited_foreground,
            Some(ui_typography::muted_foreground_color(theme))
        );
    }

    #[test]
    fn control_readout_tabular_emphasis_text_uses_medium_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_control_readout_tabular_emphasis(cx, "Page 12 of 128")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_control_readout_tabular_emphasis(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(
            el.inherited_text_style,
            Some(text_control_readout_tabular_emphasis_refinement(&theme))
        );
        assert!(el.inherited_text_style.as_ref().is_some_and(|style| {
            style
                .features
                .iter()
                .any(|feature| feature.tag.as_str() == "tnum" && feature.value == 1)
        }));
        assert_eq!(
            el.inherited_text_style
                .as_ref()
                .and_then(|style| style.weight),
            Some(FontWeight::MEDIUM)
        );
    }

    #[test]
    fn compact_tabular_emphasis_readout_uses_xs_medium_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_control_readout_compact_tabular_emphasis(cx, "128")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!(
                "expected text_control_readout_compact_tabular_emphasis(...) to build a Text element"
            );
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(
            el.inherited_text_style,
            Some(text_control_readout_compact_tabular_emphasis_refinement(
                &theme
            ))
        );
        let inherited = el
            .inherited_text_style
            .as_ref()
            .expect("expected inherited compact tabular readout style");
        assert_eq!(inherited.weight, Some(FontWeight::MEDIUM));
        assert_eq!(inherited.size, text_xs_refinement(&theme).size);
        assert!(
            inherited
                .features
                .iter()
                .any(|feature| { feature.tag.as_str() == "tnum" && feature.value == 1 })
        );
    }

    #[test]
    fn button_label_text_uses_medium_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_button_label(cx, "Apply selected changes")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_button_label(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(
            el.inherited_text_style,
            Some(text_button_label_refinement(&theme))
        );
    }

    #[test]
    fn fill_button_label_text_uses_growing_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_button_label_fill(cx, "Project settings and workspace tools")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_button_label_fill(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(
            el.inherited_text_style,
            Some(text_button_label_refinement(&theme))
        );
    }

    #[test]
    fn compact_fill_button_label_text_uses_xs_growing_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_button_label_compact_fill(cx, "Small nested project row")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_button_label_compact_fill(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(
            el.inherited_text_style,
            Some(text_button_label_compact_refinement(&theme))
        );
        assert_eq!(
            el.inherited_text_style
                .as_ref()
                .and_then(|style| style.size),
            text_xs_refinement(&theme).size
        );
    }

    #[test]
    fn section_chrome_label_text_uses_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_section_chrome_label(cx, "Inspector section heading with a long name")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_section_chrome_label(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(el.inherited_text_style, Some(text_sm_refinement(&theme)));
    }

    #[test]
    fn chrome_title_text_uses_medium_fill_width_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_chrome_title(cx, "Floating diagnostics window title")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_chrome_title(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(
            el.inherited_text_style,
            Some(text_chrome_title_refinement(&theme))
        );
    }

    #[test]
    fn chrome_glyph_text_uses_fixed_slot_single_line_clip() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_chrome_glyph(cx, ">")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_chrome_glyph(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(el.inherited_text_style, Some(text_sm_refinement(&theme)));
    }

    #[test]
    fn control_label_text_uses_fill_width_single_line_truncation() {
        let window = AppWindowId::default();
        let mut app = test_app();
        let bounds = test_bounds();

        let el = elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            text_control_label(cx, "Long checkbox/radio label that should not wrap")
        });
        let theme = Theme::global(&app);

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected text_control_label(...) to build a Text element");
        };

        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(el.inherited_text_style, Some(text_sm_refinement(&theme)));
    }

    #[test]
    fn base_single_line_text_roles_stay_single_line_under_narrow_layout() {
        let long = "Very long editor text role label that must truncate instead of wrapping";

        assert_single_line_text_role("control-readout", Px(16.0), move |cx| {
            text_control_readout(cx, long)
        });
        assert_single_line_text_role("control-readout-tabular", Px(18.0), move |cx| {
            text_control_readout_tabular(cx, "128 of 4096 selected")
        });
        assert_single_line_text_role("control-readout-tabular-emphasis", Px(18.0), move |cx| {
            text_control_readout_tabular_emphasis(cx, "Page 12 of 128")
        });
        assert_single_line_text_role(
            "control-readout-compact-tabular-emphasis",
            Px(18.0),
            move |cx| text_control_readout_compact_tabular_emphasis(cx, "128"),
        );
        assert_single_line_text_role("button-label", Px(18.0), move |cx| {
            text_button_label(cx, long)
        });
        assert_single_line_text_role("button-label-fill", Px(18.0), move |cx| {
            text_button_label_fill(cx, long)
        });
        assert_single_line_text_role("button-label-compact-fill", Px(18.0), move |cx| {
            text_button_label_compact_fill(cx, long)
        });
        assert_single_line_text_role("table-cell", Px(18.0), move |cx| text_table_cell(cx, long));
        assert_single_line_text_role("code-label", Px(18.0), move |cx| {
            text_code_label(cx, "pkg/runtime-with-a-very-long-name")
        });
        assert_single_line_text_role("code-block", Px(18.0), move |cx| {
            text_code_block(cx, "fn main() { println!(\"a very long line\"); }")
        });
    }

    #[test]
    fn paragraph_text_role_measures_multiple_lines_under_narrow_layout() {
        let bounds = layout_text_role("paragraph", |cx| {
            text_paragraph(
                cx,
                "Paragraph text is allowed to wrap and the parent layout must account for it.",
            )
        });

        assert!(
            bounds.size.height.0 > 24.0,
            "paragraph text should measure as multiple lines under narrow layout, got {bounds:?}"
        );
    }
}
