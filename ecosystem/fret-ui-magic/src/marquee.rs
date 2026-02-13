use fret_core::{Point, Px, Transform2D};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, Overflow, VisualTransformProps,
};
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::declarative::reduced_motion_queries;
use fret_ui_kit::declarative::scheduling::set_continuous_frames;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarqueeDirection {
    Left,
    Right,
}

impl Default for MarqueeDirection {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Clone)]
pub struct MarqueeProps {
    pub layout: LayoutStyle,
    /// Horizontal wrap width in logical pixels.
    ///
    /// This is intentionally explicit in v1 (Phase 0) to avoid depending on dynamic measurement.
    /// Authors can set it to the total width of their repeated content.
    pub wrap_width: Px,
    /// Scroll speed in pixels per second.
    pub speed_px_per_s: f32,
    pub direction: MarqueeDirection,
}

impl Default for MarqueeProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        Self {
            layout,
            wrap_width: Px(400.0),
            speed_px_per_s: 40.0,
            direction: MarqueeDirection::Left,
        }
    }
}

pub fn marquee<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: MarqueeProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let prefers_reduced_motion =
        reduced_motion_queries::prefers_reduced_motion(cx, Invalidation::Paint, false);

    let clock = cx
        .app
        .global::<fret_core::WindowFrameClockService>()
        .and_then(|svc| svc.snapshot(cx.window));

    let can_animate = !prefers_reduced_motion && clock.is_some() && props.wrap_width.0 > 0.0;
    set_continuous_frames(cx, can_animate);
    if can_animate {
        cx.notify_for_animation_frame();
    }

    let dx = if let Some(clock) = clock.filter(|_| can_animate) {
        let seconds = clock.now_monotonic.as_secs_f32();
        let wrap = props.wrap_width.0.max(1.0);
        let mut offset = (seconds * props.speed_px_per_s).rem_euclid(wrap);
        if props.direction == MarqueeDirection::Right {
            offset = -offset;
        }
        Px(-offset)
    } else {
        Px(0.0)
    };

    let outer = ContainerProps {
        layout: LayoutStyle {
            overflow: Overflow::Clip,
            ..props.layout
        },
        ..Default::default()
    };

    cx.container(outer, move |cx| {
        let mut inner_layout = LayoutStyle::default();
        inner_layout.size.width = Length::Fill;
        inner_layout.size.height = Length::Fill;

        vec![cx.visual_transform_props(
            VisualTransformProps {
                layout: inner_layout,
                transform: Transform2D::translation(Point::new(dx, Px(0.0))),
            },
            children,
        )]
    })
}
