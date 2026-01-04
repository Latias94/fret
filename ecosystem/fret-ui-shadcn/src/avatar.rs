use std::sync::Arc;

use fret_core::{FontId, FontWeight, ImageId, Px, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, ImageProps, MainAlign, Overflow, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};

/// shadcn/ui `Avatar` root.
///
/// This is a fixed-size, overflow-clipped, fully-rounded container intended to host exactly one
/// `AvatarImage` and one `AvatarFallback` (order controls paint stacking).
#[derive(Debug, Clone)]
pub struct Avatar {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Avatar {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            children,
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
        let theme = Theme::global(&*cx.app).clone();

        let base_chrome = ChromeRefinement::default().rounded(Radius::Full);
        let base_layout = LayoutRefinement::default()
            .relative()
            .w_px(MetricRef::space(Space::N8))
            .h_px(MetricRef::space(Space::N8))
            .flex_shrink_0();

        let mut props = decl_style::container_props(
            &theme,
            base_chrome.merge(self.chrome),
            base_layout.merge(self.layout),
        );
        props.layout.overflow = Overflow::Clip;

        let children = self.children;
        cx.container(props, move |_cx| children)
    }
}

/// shadcn/ui `AvatarImage`.
#[derive(Debug, Clone)]
pub struct AvatarImage {
    image: ImageId,
    opacity: f32,
    layout: LayoutRefinement,
}

impl AvatarImage {
    pub fn new(image: ImageId) -> Self {
        Self {
            image,
            opacity: 1.0,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .absolute()
                .inset(Space::N0)
                .size_full()
                .aspect_ratio(1.0)
                .merge(self.layout),
        );

        cx.image_props(ImageProps {
            layout,
            image: self.image,
            opacity: self.opacity.clamp(0.0, 1.0),
            uv: None,
        })
    }
}

/// shadcn/ui `AvatarFallback`.
#[derive(Debug, Clone)]
pub struct AvatarFallback {
    text: Arc<str>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl AvatarFallback {
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
        let theme = Theme::global(&*cx.app).clone();

        let bg = theme
            .color_by_key("muted")
            .unwrap_or(theme.colors.panel_background);
        let fg = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);

        let base_chrome = ChromeRefinement::default()
            .rounded(Radius::Full)
            .bg(ColorRef::Color(bg));

        let base_layout = LayoutRefinement::default()
            .absolute()
            .inset(Space::N0)
            .size_full()
            .aspect_ratio(1.0);

        let props = decl_style::container_props(
            &theme,
            base_chrome.merge(self.chrome),
            base_layout.merge(self.layout),
        );

        let text_px = theme
            .metric_by_key("component.avatar.fallback_text_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.avatar.fallback_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        let label = cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: text_px,
                weight: FontWeight::MEDIUM,
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        let flex_layout = decl_style::layout_style(&theme, LayoutRefinement::default().size_full());
        let flex = cx.flex(
            FlexProps {
                layout: flex_layout,
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: fret_core::Edges::all(Px(0.0)),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| vec![label],
        );

        cx.container(ContainerProps { ..props }, move |_cx| vec![flex])
    }
}
