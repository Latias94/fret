use std::sync::Arc;

use fret_core::{Point, Px, SemanticsRole, Transform2D};
use fret_ui::element::{AnyElement, VisualTransformProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::scheduling;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};

use crate::badge::{Badge, BadgeVariant};
use crate::test_id::attach_test_id;

/// A shadcn-styled marquee/ticker block inspired by Kibo's shadcn blocks.
///
/// This is intentionally frame-driven (runner-owned animation frames) and keeps scheduling policy
/// in the component layer.
///
/// Upstream inspiration (MIT):
/// - `repo-ref/kibo/packages/marquee`
#[derive(Debug, Clone)]
pub struct Marquee {
    items: Vec<Arc<str>>,
    direction: MarqueeDirection,
    /// Scroll speed in pixels per frame (`0` disables animation).
    speed_px_per_frame: Px,
    /// Optional cycle width for wrapping (one full track width, not including `track_gap`).
    ///
    /// Notes:
    /// - When not set, this falls back to the root render bounds width for determinism.
    /// - For a seamless loop, callers should pass a measured width that matches the rendered track.
    cycle_width_px: Option<Px>,
    /// Gap between repeated tracks (also used for the wrap cycle width).
    track_gap: Space,
    /// Gap between items within a track.
    item_gap: Space,
    test_id: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarqueeDirection {
    #[default]
    Left,
    Right,
}

impl Marquee {
    pub fn new(items: impl IntoIterator<Item = impl Into<Arc<str>>>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            direction: MarqueeDirection::default(),
            speed_px_per_frame: Px(0.5),
            cycle_width_px: None,
            track_gap: Space::N4,
            item_gap: Space::N2,
            test_id: None,
            a11y_label: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn direction(mut self, direction: MarqueeDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Scroll speed in pixels per frame (`0` disables animation).
    pub fn speed_px_per_frame(mut self, speed: Px) -> Self {
        self.speed_px_per_frame = speed;
        self
    }

    /// Override the wrap cycle width (one full track width, not including `track_gap`).
    pub fn cycle_width_px(mut self, width: Px) -> Self {
        self.cycle_width_px = Some(width);
        self
    }

    pub fn track_gap(mut self, gap: Space) -> Self {
        self.track_gap = gap;
        self
    }

    pub fn item_gap(mut self, gap: Space) -> Self {
        self.item_gap = gap;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
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
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let speed = self.speed_px_per_frame;
            let animating = speed.0.abs() > 0.0;
            scheduling::set_continuous_frames(cx, animating);

            let items = self.items;
            let item_gap = self.item_gap;
            let track_gap = self.track_gap;

            let track0 = cx.named("track0", |cx| build_track(cx, &items, item_gap));

            let track1 = cx.named("track1", |cx| build_track(cx, &items, item_gap));

            let track_row = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .items_center()
                    .gap_x(track_gap)
                    .layout(LayoutRefinement::default().flex_shrink_0()),
                |_cx| vec![track0, track1],
            );

            let translate_x = if animating {
                let raw = cx.app.frame_id().0 as f32 * speed.0;
                let base = self.cycle_width_px.unwrap_or_else(|| cx.bounds.size.width);
                let gap_px = MetricRef::space(track_gap).resolve(&theme);
                let cycle = base.0.max(0.0) + gap_px.0.max(0.0);
                if cycle > 0.0 {
                    raw.rem_euclid(cycle)
                } else {
                    raw
                }
            } else {
                0.0
            };

            let translate_x = match self.direction {
                MarqueeDirection::Left => -translate_x,
                MarqueeDirection::Right => translate_x,
            };

            let mut props = decl_style::container_props(
                &theme,
                ChromeRefinement::default().merge(self.chrome),
                LayoutRefinement::default()
                    .w_full()
                    .overflow_hidden()
                    .merge(self.layout),
            );
            props.layout.overflow = fret_ui::element::Overflow::Clip;

            let transform = Transform2D::translation(Point::new(Px(translate_x), Px(0.0)));
            let layout = decl_style::layout_style(&theme, LayoutRefinement::default());
            let inner = cx
                .visual_transform_props(VisualTransformProps { layout, transform }, move |_cx| {
                    vec![track_row]
                });

            let root = cx.container(props, move |_cx| vec![inner]);

            let test_id = self
                .test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.marquee"));
            let root = attach_test_id(root, test_id);

            root.attach_semantics(
                fret_ui::element::SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .label(
                        self.a11y_label
                            .unwrap_or_else(|| Arc::<str>::from("Marquee")),
                    ),
            )
        })
    }
}

fn build_track<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: &[Arc<str>],
    gap: Space,
) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default()
            .items_center()
            .gap_x(gap)
            .layout(LayoutRefinement::default().flex_shrink_0()),
        |cx| {
            items
                .iter()
                .enumerate()
                .map(|(idx, label)| {
                    cx.keyed(idx as u64, |cx| {
                        Badge::new(label.clone())
                            .variant(BadgeVariant::Secondary)
                            .refine_layout(LayoutRefinement::default().flex_shrink_0())
                            .into_element(cx)
                    })
                })
                .collect::<Vec<_>>()
        },
    )
}
