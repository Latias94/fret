use std::sync::Arc;

use fret_core::{
    AttributedText, FontId, FontWeight, TextAlign, TextOverflow, TextSpan, TextStyle, TextWrap,
};
use fret_ui::element::{AnyElement, Length, SelectableTextProps, SizeStyle, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};

#[derive(Debug, Clone)]
pub struct Label {
    text: Arc<str>,
}

impl Label {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        label(cx, self.text)
    }
}

#[track_caller]
pub fn label<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    let text = text.into();
    let (fg, px, line_height) = {
        let theme = Theme::global(&*cx.app);

        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"));
        let px = theme
            .metric_by_key("component.label.text_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let line_height = theme
            .metric_by_key("component.label.line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        (fg, px, line_height)
    };

    cx.text_props(TextProps {
        layout: fret_ui::element::LayoutStyle {
            size: SizeStyle {
                height: Length::Px(line_height),
                ..Default::default()
            },
            ..Default::default()
        },
        text,
        style: Some(TextStyle {
            font: FontId::default(),
            size: px,
            weight: FontWeight::MEDIUM,
            slant: Default::default(),
            line_height: Some(line_height),
            letter_spacing_em: None,
        }),
        color: Some(fg),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
    })
}

#[derive(Debug, Clone)]
pub struct SelectableLabel {
    text: Arc<str>,
}

impl SelectableLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        selectable_label(cx, self.text)
    }
}

/// A non-editable label that supports text selection (drag-to-select + `edit.copy`).
///
/// Recommended usage:
/// - Use this for "information" labels (IDs, paths, log snippets, read-only values).
/// - Avoid using it inside pressable/clickable rows because it intentionally captures left-drag
///   selection gestures and stops propagation (use a dedicated copy button instead).
#[track_caller]
pub fn selectable_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let text: Arc<str> = text.into();
    let (fg, px, line_height) = {
        let theme = Theme::global(&*cx.app);

        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"));
        let px = theme
            .metric_by_key("component.label.text_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let line_height = theme
            .metric_by_key("component.label.line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        (fg, px, line_height)
    };

    let spans: Arc<[TextSpan]> = Arc::from([TextSpan::new(text.len())]);
    let rich = AttributedText::new(Arc::clone(&text), spans);

    cx.selectable_text_props(SelectableTextProps {
        layout: fret_ui::element::LayoutStyle {
            size: SizeStyle {
                height: Length::Px(line_height),
                ..Default::default()
            },
            ..Default::default()
        },
        rich,
        style: Some(TextStyle {
            font: FontId::default(),
            size: px,
            weight: FontWeight::MEDIUM,
            slant: Default::default(),
            line_height: Some(line_height),
            letter_spacing_em: None,
        }),
        color: Some(fg),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
    })
}
