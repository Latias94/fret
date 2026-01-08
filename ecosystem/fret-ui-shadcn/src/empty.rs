use std::sync::Arc;

use fret_core::{FontId, FontWeight, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, Radius, Space};

use fret_ui_kit::declarative::stack;

#[derive(Debug, Clone)]
pub struct Empty {
    title: Arc<str>,
    description: Option<Arc<str>>,
}

impl Empty {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            title: title.into(),
            description: None,
        }
    }

    pub fn description(mut self, text: impl Into<Arc<str>>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        empty(cx, self.title, self.description)
    }
}

pub fn empty<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: impl Into<Arc<str>>,
    description: Option<Arc<str>>,
) -> AnyElement {
    let title = title.into();
    let theme = Theme::global(&*cx.app).clone();

    let bg = theme.color_required("card");
    let border = theme.color_required("border");
    let fg = theme.color_required("foreground");
    let muted_fg = theme.color_required("muted-foreground");

    let props = decl_style::container_props(
        &theme,
        ChromeRefinement::default()
            .p(Space::N6)
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Color(bg))
            .border_color(ColorRef::Color(border)),
        LayoutRefinement::default(),
    );

    cx.container(props, |cx| {
        let title_px = theme
            .metric_by_key("component.empty.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let title_lh = theme
            .metric_by_key("component.empty.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        let desc_px = theme
            .metric_by_key("component.empty.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let desc_lh = theme
            .metric_by_key("component.empty.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        vec![stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N1p5)
                .justify(Justify::Start)
                .items(Items::Start),
            |cx| {
                let mut out = Vec::new();
                out.push(cx.text_props(TextProps {
                    layout: Default::default(),
                    text: title,
                    style: Some(TextStyle {
                        font: FontId::default(),
                        size: title_px,
                        weight: FontWeight::SEMIBOLD,
                        slant: Default::default(),
                        line_height: Some(title_lh),
                        letter_spacing_em: None,
                    }),
                    color: Some(fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                }));

                if let Some(desc) = description {
                    out.push(cx.text_props(TextProps {
                        layout: Default::default(),
                        text: desc,
                        style: Some(TextStyle {
                            font: FontId::default(),
                            size: desc_px,
                            weight: FontWeight::NORMAL,
                            slant: Default::default(),
                            line_height: Some(desc_lh),
                            letter_spacing_em: None,
                        }),
                        color: Some(muted_fg),
                        wrap: TextWrap::Word,
                        overflow: TextOverflow::Clip,
                    }));
                }

                out
            },
        )]
    })
}
