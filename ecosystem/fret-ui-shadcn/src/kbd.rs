use std::sync::Arc;

use fret_core::window::ColorScheme;
use fret_core::{FontWeight, Px};
use fret_icons::IconId;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

use crate::surface_slot::{ShadcnSurfaceSlot, surface_slot_in_scope};

#[derive(Debug)]
enum KbdContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

#[derive(Debug)]
pub struct Kbd {
    content: KbdContent,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Kbd {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: KbdContent::Text(text.into()),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn from_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: KbdContent::Children(children.into_iter().collect()),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Overrides the contents of this `Kbd` with an explicit child list.
    ///
    /// This is primarily used for icon-first keycap rendering (shadcn `&>svg` patterns), so
    /// demos can avoid relying on platform fonts having `⌘`/`⌥` glyphs.
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.content = KbdContent::Children(children.into_iter().collect());
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        kbd_with_patch(cx, self.content, self.chrome, self.layout)
    }
}

pub fn kbd<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
    kbd_with_patch(
        cx,
        KbdContent::Text(text.into()),
        ChromeRefinement::default(),
        LayoutRefinement::default(),
    )
}

#[track_caller]
pub fn kbd_icon<H: UiHost>(cx: &mut ElementContext<'_, H>, icon: IconId) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let px = theme
        .metric_by_key("component.kbd.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    decl_icon::icon_with(cx, icon, Some(px), None)
}

fn kbd_with_patch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    content: KbdContent,
    chrome_override: ChromeRefinement,
    layout_override: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();

    let (bg, fg) = if surface_slot_in_scope(cx) == Some(ShadcnSurfaceSlot::TooltipContent) {
        // Upstream shadcn/ui (new-york-v4):
        // - default: `[[data-slot=tooltip-content]_&]:bg-background/20`
        // - dark: `dark:[[data-slot=tooltip-content]_&]:bg-background/10`
        let alpha = if theme.color_scheme == Some(ColorScheme::Dark) {
            0.10
        } else {
            0.20
        };
        (
            alpha_mul(theme.color_token("background"), alpha),
            theme.color_token("background"),
        )
    } else {
        (
            theme.color_token("muted"),
            theme.color_token("muted-foreground"),
        )
    };

    let chrome = ChromeRefinement::default()
        .px(Space::N1)
        .py(Space::N0p5)
        .rounded(Radius::Sm)
        .bg(ColorRef::Color(bg))
        .merge(chrome_override);

    let layout_override = LayoutRefinement::default()
        .h_px(Px(20.0))
        .min_h(Px(20.0))
        .min_w(Px(20.0))
        .merge(layout_override);

    let props = decl_style::container_props(&theme, chrome, layout_override);

    let gap = MetricRef::space(Space::N1).resolve(&theme);

    let px = theme
        .metric_by_key("component.kbd.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.kbd.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));

    cx.container(props, |cx| {
        vec![cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                gap: gap.into(),
                padding: fret_core::Edges::all(Px(0.0)).into(),
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| match content {
                KbdContent::Text(text) => vec![ui::label(cx, text)
                        .text_size_px(px)
                        .fixed_line_box_px(line_height)
                        .line_box_in_bounds()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(ColorRef::Color(fg))
                        .into_element(cx)],
                KbdContent::Children(children) => children,
            },
        )]
    })
}

fn alpha_mul(mut c: fret_core::Color, mul: f32) -> fret_core::Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug)]
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let gap = MetricRef::space(Space::N1).resolve(&theme);
        let direction = direction_prim::use_direction_in_scope(cx, None);
        let children = match direction {
            direction_prim::LayoutDirection::Ltr => self.children,
            direction_prim::LayoutDirection::Rtl => self.children.into_iter().rev().collect(),
        };
        let layout = decl_style::layout_style(&theme, self.layout);

        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Horizontal,
                gap: gap.into(),
                padding: fret_core::Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}
