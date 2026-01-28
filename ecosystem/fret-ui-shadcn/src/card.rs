use std::sync::Arc;

use fret_core::{Px, TextWrap};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Space, ui};

use crate::layout as shadcn_layout;

fn card_chrome(theme: &Theme) -> ChromeRefinement {
    let bg = theme.color_required("card");
    let border = theme.color_required("border");

    // shadcn/ui v4: Card uses `rounded-xl`, which is computed from the base `--radius`.
    //
    // In the shadcn token model:
    // - `rounded-lg` ~= `--radius`
    // - `rounded-md` ~= `--radius - 2px`
    // - `rounded-xl` ~= `--radius + 4px`
    //
    // We model the base radius as `metric.radius.lg`, and derive `rounded-xl` from it to keep
    // behavior stable when the theme radius changes.
    let base_radius = theme.metric_required("metric.radius.lg");
    let rounded_xl = Px(base_radius.0 + 4.0);

    ChromeRefinement::default()
        .radius(rounded_xl)
        .border_1()
        .bg(ColorRef::Color(bg))
        .border_color(ColorRef::Color(border))
}

#[derive(Debug, Clone)]
pub struct Card {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Card {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
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
        let chrome = card_chrome(&theme).merge(self.chrome);
        let mut props = decl_style::container_props(&theme, chrome, self.layout);
        let radius = props.corner_radii.top_left;
        props.shadow = Some(decl_style::shadow_sm(&theme, radius));
        let children = self.children;
        // Cards behave like block containers in shadcn/ui examples: their sections are expected to
        // stretch to the card width unless explicitly constrained.
        shadcn_layout::container_vstack(
            cx,
            props,
            stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
            children,
        )
    }
}

pub fn card<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    Card::new(f(cx)).into_element(cx)
}

#[derive(Debug, Clone)]
pub struct CardHeader {
    children: Vec<AnyElement>,
}

impl CardHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default().p(Space::N6),
            LayoutRefinement::default().w_full(),
        );
        let children = self.children;
        shadcn_layout::container_vstack(
            cx,
            props,
            stack::VStackProps::default()
                .gap(Space::N1p5)
                .layout(LayoutRefinement::default().w_full()),
            children,
        )
    }
}

#[derive(Debug, Clone)]
pub struct CardContent {
    children: Vec<AnyElement>,
}

impl CardContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default().p(Space::N6).pt(Space::N0),
            LayoutRefinement::default().w_full(),
        );
        let children = self.children;
        shadcn_layout::container_vstack(
            cx,
            props,
            stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
            children,
        )
    }
}

#[derive(Debug, Clone)]
pub struct CardFooter {
    children: Vec<AnyElement>,
}

impl CardFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default().p(Space::N6).pt(Space::N0),
            LayoutRefinement::default().w_full(),
        );
        let children = self.children;
        shadcn_layout::container_hstack(
            cx,
            props,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center(),
            children,
        )
    }
}

#[derive(Debug, Clone)]
pub struct CardTitle {
    text: Arc<str>,
}

impl CardTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_required("card-foreground");

        let px = theme
            .metric_by_key("component.card.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.card.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_semibold()
            .letter_spacing_em(-0.02)
            .nowrap()
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct CardDescription {
    text: Arc<str>,
}

impl CardDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_required("muted-foreground");

        let px = theme
            .metric_by_key("component.card.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.card.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_normal()
            .wrap(TextWrap::Word)
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}
