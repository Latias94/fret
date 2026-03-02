use std::sync::Arc;

use fret_core::{
    AttributedText, FontId, FontWeight, TextAlign, TextOverflow, TextSpan, TextStyle, TextWrap,
};
use fret_ui::element::{
    AnyElement, Length, SelectableTextProps, SizeStyle, TextInkOverflow, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::typography::{self, TextIntent};

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
        style: Some(typography::with_intent(
            TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::MEDIUM,
                line_height: Some(line_height),
                ..Default::default()
            },
            TextIntent::Control,
        )),
        color: Some(fg),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
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
        style: Some(typography::with_intent(
            TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::MEDIUM,
                line_height: Some(line_height),
                ..Default::default()
            },
            TextIntent::Control,
        )),
        color: Some(fg),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: TextInkOverflow::None,
        interactive_spans: Arc::from([]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::ElementKind;

    #[test]
    fn label_defaults_match_shadcn_expectations() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            label(cx, "Email")
        });

        let ElementKind::Text(props) = &el.kind else {
            panic!("expected label(...) to build a Text element");
        };

        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Clip);

        let style = props.style.as_ref().expect("label text style");
        assert_eq!(style.weight, FontWeight::MEDIUM);
    }
}
