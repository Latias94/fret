use std::sync::Arc;

use fret_core::{TextOverflow, TextWrap};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, Radius, Space, ui,
};

use fret_ui_kit::declarative::stack;

#[derive(Debug, Clone)]
pub struct Empty {
    title: Arc<str>,
    description: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Empty {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            title: title.into(),
            description: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn description(mut self, text: impl Into<Arc<str>>) -> Self {
        self.description = Some(text.into());
        self
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
        empty_with_patch(cx, self.title, self.description, self.chrome, self.layout)
    }
}

pub fn empty<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: impl Into<Arc<str>>,
    description: Option<Arc<str>>,
) -> AnyElement {
    empty_with_patch(
        cx,
        title,
        description,
        ChromeRefinement::default(),
        LayoutRefinement::default(),
    )
}

fn empty_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: impl Into<Arc<str>>,
    description: Option<Arc<str>>,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
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
            .border_color(ColorRef::Color(border))
            .merge(chrome_override),
        LayoutRefinement::default().merge(layout_override),
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
                out.push(
                    ui::text(cx, title)
                        .text_size_px(title_px)
                        .line_height_px(title_lh)
                        .font_semibold()
                        .nowrap()
                        .text_color(ColorRef::Color(fg))
                        .into_element(cx),
                );

                if let Some(desc) = description {
                    out.push(
                        ui::text(cx, desc)
                            .text_size_px(desc_px)
                            .line_height_px(desc_lh)
                            .font_normal()
                            .wrap(TextWrap::Word)
                            .overflow(TextOverflow::Clip)
                            .text_color(ColorRef::Color(muted_fg))
                            .into_element(cx),
                    );
                }

                out
            },
        )]
    })
}
