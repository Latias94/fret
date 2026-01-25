use std::sync::Arc;

use fret_core::{Edges, Px, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

#[derive(Debug, Clone)]
pub struct Empty {
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl Empty {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children: children.into_iter().collect(),
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
        let theme = Theme::global(&*cx.app).clone();

        let border = theme.color_required("border");
        let fg = theme.color_required("foreground");

        let chrome = ChromeRefinement::default()
            .p(Space::N6)
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(border))
            .text_color(ColorRef::Color(fg))
            .merge(self.chrome);

        let layout = LayoutRefinement::default()
            .min_w_0()
            .flex_1()
            .merge(self.layout);

        let props = decl_style::container_props(&theme, chrome, layout);
        let children = self.children;

        cx.container(props, move |cx| {
            let layout = decl_style::layout_style(&theme, LayoutRefinement::default().size_full());
            let gap = MetricRef::space(Space::N6).resolve(&theme);
            vec![cx.flex(
                FlexProps {
                    layout,
                    direction: fret_core::Axis::Vertical,
                    gap,
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| children,
            )]
        })
    }
}

#[derive(Debug, Clone)]
pub struct EmptyHeader {
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl EmptyHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            layout: LayoutRefinement::default(),
            children: children.into_iter().collect(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N2).resolve(&theme);
        let max_w = MetricRef::Px(Px(384.0));
        let layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .max_w(max_w)
                .min_w_0()
                .merge(self.layout),
        );
        let children = self.children;
        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Vertical,
                gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmptyMediaVariant {
    #[default]
    Default,
    Icon,
}

#[derive(Debug, Clone)]
pub struct EmptyMedia {
    variant: EmptyMediaVariant,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl EmptyMedia {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            variant: EmptyMediaVariant::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children: children.into_iter().collect(),
        }
    }

    pub fn variant(mut self, variant: EmptyMediaVariant) -> Self {
        self.variant = variant;
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
        let theme = Theme::global(&*cx.app).clone();

        let mut layout = LayoutRefinement::default()
            .mb(Space::N2)
            .flex_shrink_0()
            .merge(self.layout);
        let mut chrome = ChromeRefinement::default().merge(self.chrome);

        if self.variant == EmptyMediaVariant::Icon {
            let bg = theme
                .color_by_key("muted")
                .unwrap_or_else(|| theme.color_required("muted.background"));
            let fg = theme.color_required("foreground");
            layout = layout
                .w_px(MetricRef::space(Space::N10))
                .h_px(MetricRef::space(Space::N10));
            chrome = ChromeRefinement::default()
                .rounded(Radius::Lg)
                .bg(ColorRef::Color(bg))
                .text_color(ColorRef::Color(fg))
                .merge(chrome);
        }

        let props = decl_style::container_props(&theme, chrome, layout);
        let children = self.children;
        cx.container(props, move |cx| {
            let layout = decl_style::layout_style(&theme, LayoutRefinement::default().size_full());
            vec![cx.flex(
                FlexProps {
                    layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| children,
            )]
        })
    }
}

#[derive(Debug, Clone)]
pub struct EmptyTitle {
    text: Arc<str>,
}

impl EmptyTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_required("foreground");
        let px = theme
            .metric_by_key("component.empty.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.empty.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_medium()
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct EmptyDescription {
    text: Arc<str>,
}

impl EmptyDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_required("muted-foreground");
        let px = theme
            .metric_by_key("component.empty.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.empty.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_normal()
            .wrap(TextWrap::Word)
            .overflow(TextOverflow::Clip)
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct EmptyContent {
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl EmptyContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            layout: LayoutRefinement::default(),
            children: children.into_iter().collect(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N4).resolve(&theme);
        let max_w = MetricRef::Px(Px(384.0));
        let layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w)
                .min_w_0()
                .merge(self.layout),
        );
        let children = self.children;
        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Vertical,
                gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}
