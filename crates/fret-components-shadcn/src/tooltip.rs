use fret_components_ui::declarative::scheduling;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::hover_intent::{HoverIntentConfig, HoverIntentState};
use fret_components_ui::overlay;
use fret_components_ui::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayController, OverlayPresence,
    OverlayRequest, Radius, Space,
};
use std::sync::Arc;

use fret_core::{Px, Size, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{
    AnyElement, HoverRegionProps, InsetStyle, LayoutStyle, PositionStyle, TextProps,
};
use fret_ui::overlay_placement::{Align, Side, anchored_panel_bounds_sized};
use fret_ui::{ElementCx, Theme, UiHost};

fn tooltip_content_chrome(theme: &Theme) -> ChromeRefinement {
    // shadcn/ui v4 (2025-09-22): tooltip uses `bg-foreground text-background`.
    let bg = theme
        .color_by_key("foreground")
        .unwrap_or(theme.colors.text_primary);

    ChromeRefinement::default()
        .rounded(Radius::Sm)
        .bg(ColorRef::Color(bg))
        .px(Space::N3)
        .py(Space::N2)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TooltipAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TooltipSide {
    #[default]
    Top,
    Right,
    Bottom,
    Left,
}

/// shadcn/ui `Tooltip` root (v4).
///
/// This is implemented as a component-layer policy built on runtime substrate primitives:
/// - `HoverRegion` (hover tracking)
/// - cross-frame geometry queries (`elements::bounds_for_element`)
/// - placement solver (`overlay_placement`)
///
/// Note: This uses a per-window overlay root, so it is not clipped by ancestors with
/// `overflow: Clip`.
#[derive(Debug, Clone)]
pub struct Tooltip {
    trigger: AnyElement,
    content: AnyElement,
    align: TooltipAlign,
    side: TooltipSide,
    side_offset: Px,
    window_margin_override: Option<Px>,
    open_delay_frames: u32,
    close_delay_frames: u32,
    layout: LayoutRefinement,
}

impl Tooltip {
    pub fn new(trigger: AnyElement, content: AnyElement) -> Self {
        Self {
            trigger,
            content,
            align: TooltipAlign::default(),
            side: TooltipSide::default(),
            side_offset: Px(0.0),
            window_margin_override: None,
            open_delay_frames: 0,
            close_delay_frames: 0,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn align(mut self, align: TooltipAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: TooltipSide) -> Self {
        self.side = side;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn open_delay_frames(mut self, frames: u32) -> Self {
        self.open_delay_frames = frames;
        self
    }

    pub fn close_delay_frames(mut self, frames: u32) -> Self {
        self.close_delay_frames = frames;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin_override = Some(margin);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let layout = decl_style::layout_style(&theme, self.layout);
        let side_offset = if self.side_offset == Px(0.0) {
            theme
                .metric_by_key("component.tooltip.side_offset")
                .unwrap_or(self.side_offset)
        } else {
            self.side_offset
        };
        let window_margin = self.window_margin_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.tooltip.window_margin")
                .unwrap_or(Px(8.0))
        });

        let align = self.align;
        let side = self.side;
        let open_delay_frames = self.open_delay_frames;
        let close_delay_frames = self.close_delay_frames;

        let trigger = self.trigger;
        let content = self.content;
        let trigger_id = trigger.id;
        let content_id = content.id;

        cx.hover_region(HoverRegionProps { layout }, move |cx, hovered| {
            let frame = cx.app.frame_id();
            let cfg = HoverIntentConfig::new(open_delay_frames as u64, close_delay_frames as u64);
            let update = cx.with_state(HoverIntentState::default, |state| {
                state.update(hovered, frame.0, cfg)
            });

            scheduling::set_continuous_frames(cx, update.wants_continuous_ticks);

            let out = vec![trigger];
            if !update.open {
                return out;
            }

            let tooltip_id = cx.root_id();
            let overlay_root_name = OverlayController::tooltip_root_name(tooltip_id);

            let overlay_children = cx.with_root_name(&overlay_root_name, |cx| {
                let anchor = overlay::anchor_bounds_for_element(cx, trigger_id);
                let Some(anchor) = anchor else {
                    return Vec::new();
                };

                let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
                let estimated_size = Size::new(Px(240.0), Px(44.0));
                let content_size = last_content_size.unwrap_or(estimated_size);

                let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                let align = match align {
                    TooltipAlign::Start => Align::Start,
                    TooltipAlign::Center => Align::Center,
                    TooltipAlign::End => Align::End,
                };
                let side = match side {
                    TooltipSide::Top => Side::Top,
                    TooltipSide::Right => Side::Right,
                    TooltipSide::Bottom => Side::Bottom,
                    TooltipSide::Left => Side::Left,
                };

                let placed = anchored_panel_bounds_sized(
                    outer,
                    anchor,
                    content_size,
                    side_offset,
                    side,
                    align,
                );

                let wrapper = cx.container(
                    fret_ui::element::ContainerProps {
                        layout: LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                top: Some(placed.origin.y),
                                left: Some(placed.origin.x),
                                ..Default::default()
                            },
                            size: fret_ui::element::SizeStyle {
                                width: fret_ui::element::Length::Px(placed.size.width),
                                height: fret_ui::element::Length::Px(placed.size.height),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |_cx| vec![content],
                );

                vec![wrapper]
            });

            let mut request = OverlayRequest::tooltip(
                tooltip_id,
                OverlayPresence::instant(true),
                overlay_children,
            );
            request.root_name = Some(overlay_root_name);
            OverlayController::request(cx, request);

            out
        })
    }
}

/// shadcn/ui `TooltipTrigger` (v4).
#[derive(Debug, Clone)]
pub struct TooltipTrigger {
    child: AnyElement,
}

impl TooltipTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementCx<'_, H>) -> AnyElement {
        self.child
    }
}

/// shadcn/ui `TooltipContent` (v4).
#[derive(Debug, Clone)]
pub struct TooltipContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl TooltipContent {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn text<H: UiHost>(cx: &mut ElementCx<'_, H>, text: impl Into<Arc<str>>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let text = text.into();

        let text_style = TextStyle {
            font: fret_core::FontId::default(),
            size: theme.metrics.font_size,
            weight: fret_core::FontWeight::NORMAL,
            line_height: Some(theme.metrics.font_line_height),
            letter_spacing_em: None,
        };

        let fg = theme
            .color_by_key("background")
            .unwrap_or(theme.colors.surface_background);

        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text,
            style: Some(text_style),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
        })
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let base_layout = LayoutRefinement::default().flex_shrink_0();
        let chrome = tooltip_content_chrome(&theme).merge(self.chrome);
        let mut props = decl_style::container_props(&theme, chrome, base_layout.merge(self.layout));

        let radius = MetricRef::radius(Radius::Md).resolve(&theme);
        props.shadow = Some(decl_style::shadow_sm(&theme, radius));
        let children = self.children;
        cx.container(props, move |_cx| children)
    }
}
