use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::hover_intent::{HoverIntentConfig, HoverIntentState};
use fret_components_ui::window_overlays;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};
use fret_core::{Edges, Px, Size};
use fret_runtime::Effect;
use fret_ui::element::{AnyElement, HoverRegionProps, InsetStyle, LayoutStyle, PositionStyle};
use fret_ui::{ElementCx, Theme, UiHost, overlay_placement};

fn hover_card_content_chrome(theme: &Theme) -> ChromeRefinement {
    let bg = theme
        .color_by_key("popover")
        .unwrap_or(theme.colors.panel_background);
    let border = theme
        .color_by_key("border")
        .unwrap_or(theme.colors.panel_border);

    ChromeRefinement::default()
        .rounded(Radius::Md)
        .border_1()
        .bg(ColorRef::Color(bg))
        .border_color(ColorRef::Color(border))
        .p(Space::N4)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HoverCardAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Debug, Default, Clone, Copy)]
struct HoverCardSharedState {
    overlay_hovered: bool,
}

/// shadcn/ui `HoverCard` root (v4).
///
/// This is a floating hover surface anchored to a trigger. In Radix/shadcn this uses a portal;
/// in Fret this is implemented as a component-layer policy built on runtime substrate primitives:
/// - `HoverRegion` (hover tracking)
/// - cross-frame geometry queries (`elements::bounds_for_element`)
/// - placement solver (`overlay_placement`)
#[derive(Debug, Clone)]
pub struct HoverCard {
    trigger: AnyElement,
    content: AnyElement,
    align: HoverCardAlign,
    side_offset: Px,
    window_margin_override: Option<Px>,
    open_delay_frames: u32,
    close_delay_frames: u32,
    layout: LayoutRefinement,
}

impl HoverCard {
    pub fn new(trigger: AnyElement, content: AnyElement) -> Self {
        Self {
            trigger,
            content,
            align: HoverCardAlign::default(),
            side_offset: Px(4.0),
            window_margin_override: None,
            open_delay_frames: 0,
            // Non-zero by default so the user can move from trigger to the overlay content.
            close_delay_frames: 6,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn align(mut self, align: HoverCardAlign) -> Self {
        self.align = align;
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
        let side_offset = if self.side_offset == Px(4.0) {
            theme
                .metric_by_key("component.hover_card.side_offset")
                .unwrap_or(self.side_offset)
        } else {
            self.side_offset
        };
        let window_margin = self.window_margin_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.hover_card.window_margin")
                .unwrap_or(Px(8.0))
        });

        let align = self.align;
        let open_delay_frames = self.open_delay_frames;
        let close_delay_frames = self.close_delay_frames;

        let trigger = self.trigger;
        let content = self.content;
        let trigger_id = trigger.id;
        let content_id = content.id;
        cx.hover_region(HoverRegionProps { layout }, move |cx, hovered| {
            let frame = cx.app.frame_id();
            let hover_card_id = cx.root_id();

            let overlay_hovered =
                cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                    st.overlay_hovered
                });
            let hovered = hovered || overlay_hovered;

            let cfg = HoverIntentConfig::new(open_delay_frames as u64, close_delay_frames as u64);
            let update = cx.with_state(HoverIntentState::default, |state| {
                state.update(hovered, frame.0, cfg)
            });

            if update.wants_continuous_ticks {
                cx.app.push_effect(Effect::RequestAnimationFrame(cx.window));
                cx.app.request_redraw(cx.window);
            }

            let out = vec![trigger];
            if !update.open {
                cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                    st.overlay_hovered = false;
                });
                return out;
            }

            let overlay_root_name = window_overlays::hover_overlay_root_name(hover_card_id);

            let overlay_children = cx.with_root_name(&overlay_root_name, |cx| {
                let anchor = crate::overlay_anchor::anchor_bounds_for_element(cx, trigger_id);
                let Some(anchor) = anchor else {
                    cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                        st.overlay_hovered = false;
                    });
                    return Vec::new();
                };

                let last_content_size = cx.last_bounds_for_element(content_id).map(|r| r.size);
                let estimated_size = Size::new(Px(256.0), Px(120.0));
                let content_size = last_content_size.unwrap_or(estimated_size);

                let outer = overlay_placement::inset_rect(cx.bounds, Edges::all(window_margin));

                let align = match align {
                    HoverCardAlign::Start => overlay_placement::Align::Start,
                    HoverCardAlign::Center => overlay_placement::Align::Center,
                    HoverCardAlign::End => overlay_placement::Align::End,
                };

                let placed = overlay_placement::anchored_panel_bounds_sized(
                    outer,
                    anchor,
                    content_size,
                    side_offset,
                    overlay_placement::Side::Bottom,
                    align,
                );

                vec![cx.hover_region(
                    HoverRegionProps {
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
                    },
                    move |cx, hovered| {
                        cx.with_state_for(hover_card_id, HoverCardSharedState::default, |st| {
                            st.overlay_hovered = hovered;
                        });
                        vec![content]
                    },
                )]
            });

            window_overlays::request_hover_overlay(
                cx,
                window_overlays::HoverOverlayRequest {
                    id: hover_card_id,
                    root_name: overlay_root_name,
                    trigger: trigger_id,
                    children: overlay_children,
                },
            );

            out
        })
    }
}

/// shadcn/ui `HoverCardTrigger` (v4).
///
/// In the DOM this is a context-aware wrapper that does not impose layout. In Fret's declarative
/// authoring, the trigger is expressed as the first child passed to `HoverCard::new(...)`.
#[derive(Debug, Clone)]
pub struct HoverCardTrigger {
    child: AnyElement,
}

impl HoverCardTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self { child }
    }

    pub fn into_element<H: UiHost>(self, _cx: &mut ElementCx<'_, H>) -> AnyElement {
        self.child
    }
}

/// shadcn/ui `HoverCardContent` (v4).
#[derive(Debug, Clone)]
pub struct HoverCardContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl HoverCardContent {
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let base_layout = LayoutRefinement::default()
            .w_px(MetricRef::Px(Px(256.0)))
            .flex_shrink_0();

        let chrome = hover_card_content_chrome(&theme).merge(self.chrome);
        let radius = MetricRef::radius(Radius::Md).resolve(&theme);
        let mut props = decl_style::container_props(&theme, chrome, base_layout.merge(self.layout));
        props.shadow = Some(decl_style::shadow_md(&theme, radius));
        let children = self.children;
        cx.container(props, move |_cx| children)
    }
}
