use std::sync::Arc;

use fret_core::{Edges, Px, SemanticsRole};
use fret_icons::IconId;
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign, SemanticsProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, LengthRefinement, MetricRef, Radius, Space};

use crate::{Button, ButtonSize, ButtonVariant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct Carousel {
    items: Vec<AnyElement>,
    layout: LayoutRefinement,
    viewport_layout: LayoutRefinement,
    track_layout: LayoutRefinement,
    item_layout: LayoutRefinement,
    orientation: CarouselOrientation,
    track_start_neg_margin: Space,
    item_padding_start: Space,
    item_basis_main_px: Option<Px>,
    test_id: Option<Arc<str>>,
}

impl Default for Carousel {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Carousel {
    pub fn new(items: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            items: items.into_iter().collect(),
            layout: LayoutRefinement::default(),
            viewport_layout: LayoutRefinement::default(),
            track_layout: LayoutRefinement::default(),
            item_layout: LayoutRefinement::default(),
            orientation: CarouselOrientation::Horizontal,
            track_start_neg_margin: Space::N4,
            item_padding_start: Space::N4,
            item_basis_main_px: None,
            test_id: None,
        }
    }

    pub fn items(mut self, items: impl IntoIterator<Item = AnyElement>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_viewport_layout(mut self, layout: LayoutRefinement) -> Self {
        self.viewport_layout = self.viewport_layout.merge(layout);
        self
    }

    pub fn refine_track_layout(mut self, layout: LayoutRefinement) -> Self {
        self.track_layout = self.track_layout.merge(layout);
        self
    }

    pub fn refine_item_layout(mut self, layout: LayoutRefinement) -> Self {
        self.item_layout = self.item_layout.merge(layout);
        self
    }

    pub fn orientation(mut self, orientation: CarouselOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn track_start_neg_margin(mut self, margin: Space) -> Self {
        self.track_start_neg_margin = margin;
        self
    }

    pub fn item_padding_start(mut self, padding: Space) -> Self {
        self.item_padding_start = padding;
        self
    }

    pub fn item_basis_main_px(mut self, basis: Px) -> Self {
        self.item_basis_main_px = Some(basis);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let root_layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default().relative().merge(self.layout),
            );

            let viewport_layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default()
                    .w_full()
                    .overflow_hidden()
                    .merge(self.viewport_layout),
            );

            let track_layout = match self.orientation {
                CarouselOrientation::Horizontal => LayoutRefinement::default()
                    .w_full()
                    .ml_neg(self.track_start_neg_margin)
                    .merge(self.track_layout),
                CarouselOrientation::Vertical => LayoutRefinement::default()
                    .w_full()
                    .mt_neg(self.track_start_neg_margin)
                    .merge(self.track_layout),
            };
            let track_layout = decl_style::layout_style(&theme, track_layout);

            let item_pad = decl_style::space(&theme, self.item_padding_start);

            let (track_direction, button_axis) = match self.orientation {
                CarouselOrientation::Horizontal => {
                    (fret_core::Axis::Horizontal, fret_core::Axis::Vertical)
                }
                CarouselOrientation::Vertical => {
                    (fret_core::Axis::Vertical, fret_core::Axis::Horizontal)
                }
            };

            let items_len = self.items.len();
            let items = self.items;
            let item_basis = self.item_basis_main_px;
            let item_layout_patch = self.item_layout;
            let theme_for_items = theme.clone();

            let viewport = cx.container(
                fret_ui::element::ContainerProps {
                    layout: viewport_layout,
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: track_layout,
                            direction: track_direction,
                            wrap: false,
                            ..Default::default()
                        },
                        move |cx| {
                            items
                                .into_iter()
                                .enumerate()
                                .map(|(idx, content)| {
                                    let mut item_layout = LayoutRefinement::default()
                                        .flex_none()
                                        .min_w(MetricRef::Px(Px(0.0)))
                                        .merge(item_layout_patch.clone());

                                    if let Some(basis) = item_basis {
                                        item_layout = item_layout
                                            .basis(LengthRefinement::Px(MetricRef::Px(basis)));
                                    } else if track_direction == fret_core::Axis::Horizontal {
                                        item_layout = item_layout.basis(LengthRefinement::Fill);
                                    }

                                    let item_layout =
                                        decl_style::layout_style(&theme_for_items, item_layout);
                                    let test_id = Arc::from(format!("carousel-item-{}", idx + 1));

                                    cx.semantics(
                                        SemanticsProps {
                                            layout: item_layout,
                                            role: SemanticsRole::Group,
                                            test_id: Some(test_id),
                                            ..Default::default()
                                        },
                                        move |cx| {
                                            let padding = match track_direction {
                                                fret_core::Axis::Horizontal => Edges {
                                                    left: item_pad,
                                                    ..Edges::all(Px(0.0))
                                                },
                                                fret_core::Axis::Vertical => Edges {
                                                    top: item_pad,
                                                    ..Edges::all(Px(0.0))
                                                },
                                            };

                                            vec![cx.container(
                                                fret_ui::element::ContainerProps {
                                                    padding,
                                                    ..Default::default()
                                                },
                                                move |_cx| vec![content.clone()],
                                            )]
                                        },
                                    )
                                })
                                .collect::<Vec<_>>()
                        },
                    )]
                },
            );

            let prev_disabled = true;
            let next_disabled = items_len <= 1;

            let prev_button = Button::new("Previous slide")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::IconSm)
                .disabled(prev_disabled)
                .test_id("carousel-previous")
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .children([decl_icon::icon(cx, IconId::new_static("lucide.arrow-left"))])
                .into_element(cx);

            let next_button = Button::new("Next slide")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::IconSm)
                .disabled(next_disabled)
                .test_id("carousel-next")
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .children([decl_icon::icon(
                    cx,
                    IconId::new_static("lucide.arrow-right"),
                )])
                .into_element(cx);

            let offset = MetricRef::Px(Px(48.0));
            let button_size = MetricRef::Px(Px(32.0));

            let (prev_layout, next_layout) = match self.orientation {
                CarouselOrientation::Horizontal => (
                    LayoutRefinement::default()
                        .absolute()
                        .top(Space::N0)
                        .bottom(Space::N0)
                        .left_neg_px(offset.clone())
                        .w_px(button_size.clone()),
                    LayoutRefinement::default()
                        .absolute()
                        .top(Space::N0)
                        .bottom(Space::N0)
                        .right_neg_px(offset)
                        .w_px(button_size),
                ),
                CarouselOrientation::Vertical => (
                    LayoutRefinement::default()
                        .absolute()
                        .left(Space::N0)
                        .right(Space::N0)
                        .top_neg_px(offset.clone())
                        .h_px(button_size.clone()),
                    LayoutRefinement::default()
                        .absolute()
                        .left(Space::N0)
                        .right(Space::N0)
                        .bottom_neg_px(offset)
                        .h_px(button_size),
                ),
            };

            let prev_layout = decl_style::layout_style(&theme, prev_layout);
            let next_layout = decl_style::layout_style(&theme, next_layout);

            let prev_wrapper = cx.flex(
                FlexProps {
                    layout: prev_layout,
                    direction: button_axis,
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                    ..Default::default()
                },
                move |_cx| vec![prev_button],
            );

            let next_wrapper = cx.flex(
                FlexProps {
                    layout: next_layout,
                    direction: button_axis,
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                    ..Default::default()
                },
                move |_cx| vec![next_button],
            );

            let root_test_id = self.test_id.unwrap_or_else(|| Arc::from("carousel"));
            cx.semantics(
                SemanticsProps {
                    layout: root_layout,
                    role: SemanticsRole::Group,
                    test_id: Some(root_test_id),
                    ..Default::default()
                },
                move |_cx| vec![viewport, prev_wrapper, next_wrapper],
            )
        })
    }
}
