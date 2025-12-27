use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};
use fret_core::{Edges, FrameId, Px, Size};
use fret_runtime::Effect;
use fret_ui::element::{AnyElement, HoverRegionProps, InsetStyle, LayoutStyle, PositionStyle};
use fret_ui::{ElementCx, Theme, UiHost, elements, overlay_placement};

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
struct HoverCardOpenState {
    open: bool,
    hover_start: Option<FrameId>,
    leave_start: Option<FrameId>,
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
            close_delay_frames: 0,
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
            let open_delay = open_delay_frames as u64;
            let close_delay = close_delay_frames as u64;

            let open = cx.with_state(HoverCardOpenState::default, |state| {
                if hovered {
                    state.leave_start = None;
                    if !state.open {
                        let start = state.hover_start.get_or_insert(frame);
                        let elapsed = frame.0.saturating_sub(start.0);
                        if elapsed >= open_delay {
                            state.open = true;
                            state.hover_start = None;
                        }
                    }
                } else {
                    state.hover_start = None;
                    if state.open {
                        let start = state.leave_start.get_or_insert(frame);
                        let elapsed = frame.0.saturating_sub(start.0);
                        if elapsed >= close_delay {
                            state.open = false;
                            state.leave_start = None;
                        }
                    } else {
                        state.leave_start = None;
                    }
                }
                state.open
            });

            if (hovered && !open && open_delay > 0) || (!hovered && open && close_delay > 0) {
                cx.app.push_effect(Effect::RequestAnimationFrame(cx.window));
                cx.app.request_redraw(cx.window);
            }

            let mut out = vec![trigger];
            if !open {
                return out;
            }

            let anchor = elements::bounds_for_element(&mut *cx.app, cx.window, trigger_id);
            let region_id = cx.root_id();
            let outer = elements::root_bounds_for_element(&mut *cx.app, cx.window, region_id)
                .unwrap_or(cx.bounds);
            let outer = overlay_placement::inset_rect(outer, Edges::all(window_margin));

            let last_content_size = elements::bounds_for_element(&mut *cx.app, cx.window, content_id)
                .map(|r| r.size);
            let estimated_size = Size::new(Px(256.0), Px(120.0));
            let content_size = last_content_size.unwrap_or(estimated_size);

            let Some(anchor) = anchor else {
                return out;
            };

            let align = match align {
                HoverCardAlign::Start => overlay_placement::Align::Start,
                HoverCardAlign::Center => overlay_placement::Align::Center,
                HoverCardAlign::End => overlay_placement::Align::End,
            };

            let bounds = overlay_placement::anchored_panel_bounds_sized(
                outer,
                anchor,
                content_size,
                side_offset,
                overlay_placement::Side::Bottom,
                align,
            );

            let region_bounds = elements::bounds_for_element(&mut *cx.app, cx.window, region_id)
                .unwrap_or(cx.bounds);
            let local_x = Px(bounds.origin.x.0 - region_bounds.origin.x.0);
            let local_y = Px(bounds.origin.y.0 - region_bounds.origin.y.0);

            let mut overlay_layout = LayoutStyle::default();
            overlay_layout.position = PositionStyle::Absolute;
            overlay_layout.inset = InsetStyle {
                top: Some(local_y),
                left: Some(local_x),
                right: None,
                bottom: None,
            };

            let wrapper = cx.container(
                fret_ui::element::ContainerProps {
                    layout: overlay_layout,
                    ..Default::default()
                },
                move |_cx| vec![content],
            );
            out.push(wrapper);
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
