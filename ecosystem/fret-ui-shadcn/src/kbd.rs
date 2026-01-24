use std::sync::Arc;

use fret_core::{FontWeight, Px};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, LayoutStyle, MainAlign};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

#[derive(Debug, Clone)]
pub struct Kbd {
    text: Arc<str>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Kbd {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        kbd_with_patch(cx, self.text, self.chrome, self.layout)
    }
}

pub fn kbd<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    kbd_with_patch(
        cx,
        text,
        ChromeRefinement::default(),
        LayoutRefinement::default(),
    )
}

fn kbd_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
) -> AnyElement {
    let text = text.into();
    let theme = Theme::global(&*cx.app).clone();

    let bg = theme.color_required("muted");

    let chrome = ChromeRefinement::default()
        .px(Space::N1)
        .py(Space::N0p5)
        .rounded(Radius::Sm)
        .bg(ColorRef::Color(bg))
        .merge(chrome_override);

    let layout_override = LayoutRefinement::default()
        .h_px(MetricRef::Px(Px(20.0)))
        .min_h(MetricRef::Px(Px(20.0)))
        .min_w(MetricRef::Px(Px(20.0)))
        .merge(layout_override);

    let props = decl_style::container_props(&theme, chrome, layout_override);

    let fg = theme.color_required("muted-foreground");

    let px = theme
        .metric_by_key("component.kbd.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.kbd.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));

    cx.container(props, |cx| {
        vec![cx.flex(
            FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                vec![
                    ui::label(cx, text.clone())
                        .text_size_px(px)
                        .line_height_px(line_height)
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(ColorRef::Color(fg))
                        .h_px(MetricRef::Px(line_height))
                        .into_element(cx),
                ]
            },
        )]
    })
}

#[derive(Debug, Clone)]
pub struct KbdGroup {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl KbdGroup {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N1).resolve(&theme);
        let children = self.children;
        let layout = decl_style::layout_style(&theme, self.layout);

        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Horizontal,
                gap,
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}
