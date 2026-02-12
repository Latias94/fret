use std::sync::Arc;

use fret_core::{Edges, KeyCode, MouseButton, Point, Px, SemanticsRole};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, KeyDownCx, OnKeyDown, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, ElementKind, FlexProps, LayoutStyle, MainAlign,
    PointerRegionProps, RenderTransformProps, SemanticsDecoration, VisualTransformProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, LengthRefinement, MetricRef, Radius, Space};

use crate::{Button, ButtonSize, ButtonVariant};

const CAROUSEL_SETTLE_TICKS: u64 = 12;

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

#[derive(Debug, Clone, Copy, Default)]
struct CarouselRuntime {
    dragging: bool,
    start: Point,
    start_offset: Px,
    settling: bool,
    settle_from: Px,
    settle_to: Px,
    settle_tick: u64,
}

#[derive(Default)]
struct CarouselState {
    index: Option<Model<usize>>,
    offset: Option<Model<Px>>,
    runtime: Option<Model<CarouselRuntime>>,
    extent: Option<Model<Px>>,
}

fn carousel_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<usize>, Model<Px>, Model<CarouselRuntime>, Model<Px>) {
    let needs_init = cx.with_state(CarouselState::default, |st| {
        st.index.is_none() || st.offset.is_none() || st.runtime.is_none() || st.extent.is_none()
    });

    if needs_init {
        let index = cx.app.models_mut().insert(0usize);
        let offset = cx.app.models_mut().insert(Px(0.0));
        let runtime = cx.app.models_mut().insert(CarouselRuntime::default());
        let extent = cx.app.models_mut().insert(Px(0.0));
        cx.with_state(CarouselState::default, |st| {
            st.index = Some(index.clone());
            st.offset = Some(offset.clone());
            st.runtime = Some(runtime.clone());
            st.extent = Some(extent.clone());
        });
        return (index, offset, runtime, extent);
    }

    let (index, offset, runtime, extent) = cx.with_state(CarouselState::default, |st| {
        (
            st.index.clone().expect("index"),
            st.offset.clone().expect("offset"),
            st.runtime.clone().expect("runtime"),
            st.extent.clone().expect("extent"),
        )
    });
    (index, offset, runtime, extent)
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let orientation = self.orientation;
            let root_test_id = self.test_id.unwrap_or_else(|| Arc::from("carousel"));

            let (index_model, offset_model, runtime_model, extent_model) = carousel_models(cx);

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

            let track_layout = match orientation {
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

            let (track_direction, button_axis) = match orientation {
                CarouselOrientation::Horizontal => {
                    (fret_core::Axis::Horizontal, fret_core::Axis::Vertical)
                }
                CarouselOrientation::Vertical => {
                    (fret_core::Axis::Vertical, fret_core::Axis::Horizontal)
                }
            };

            let items_len = self.items.len();
            let item_basis = self.item_basis_main_px;
            let item_layout_patch = self.item_layout;
            let items = self.items;
            let theme_for_items = theme.clone();
            let root_test_id_for_items = root_test_id.clone();

            let index_now = cx.watch_model(&index_model).copied().unwrap_or(0);
            let mut offset_now = cx.watch_model(&offset_model).copied().unwrap_or(Px(0.0));
            let runtime_snapshot = cx.watch_model(&runtime_model).copied().unwrap_or_default();

            if runtime_snapshot.settling {
                let tick = runtime_snapshot.settle_tick.saturating_add(1);
                let t = (tick as f32 / CAROUSEL_SETTLE_TICKS as f32).min(1.0);
                let eased = crate::overlay_motion::shadcn_ease(t);
                let next = Px(runtime_snapshot.settle_from.0
                    + (runtime_snapshot.settle_to.0 - runtime_snapshot.settle_from.0) * eased);
                offset_now = next;
                let _ = cx.app.models_mut().update(&offset_model, |v| *v = next);
                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                    st.settle_tick = tick;
                    if t >= 1.0 {
                        st.settling = false;
                    }
                });
                cx.request_frame();
            }

            let axis_offset = offset_now;
            let transform = match orientation {
                CarouselOrientation::Horizontal => {
                    fret_core::Transform2D::translation(Point::new(Px(-axis_offset.0), Px(0.0)))
                }
                CarouselOrientation::Vertical => {
                    fret_core::Transform2D::translation(Point::new(Px(0.0), Px(-axis_offset.0)))
                }
            };

            let offset_for_down = offset_model.clone();
            let runtime_for_down = runtime_model.clone();
            let on_down: fret_ui::action::OnPointerDown = Arc::new(move |host, _cx, down| {
                if down.button != MouseButton::Left {
                    return false;
                }
                host.capture_pointer();
                let start_offset = host
                    .models_mut()
                    .read(&offset_for_down, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));
                let _ = host.models_mut().update(&runtime_for_down, |st| {
                    st.dragging = true;
                    st.start = down.position;
                    st.start_offset = start_offset;
                    st.settling = false;
                    st.settle_tick = 0;
                });
                true
            });

            let runtime_for_move = runtime_model.clone();
            let offset_for_move = offset_model.clone();
            let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, _cx, mv| {
                let runtime = host
                    .models_mut()
                    .read(&runtime_for_move, |st| *st)
                    .ok()
                    .unwrap_or_default();
                if !runtime.dragging {
                    return false;
                }

                let bounds = host.bounds();
                let extent = match item_basis {
                    Some(px) => px,
                    None => match track_direction {
                        fret_core::Axis::Horizontal => bounds.size.width,
                        fret_core::Axis::Vertical => bounds.size.height,
                    },
                };
                if extent.0 <= 0.0 {
                    return true;
                }
                let max_offset = Px((extent.0 * (items_len.saturating_sub(1) as f32)).max(0.0));

                let delta = match track_direction {
                    fret_core::Axis::Horizontal => mv.position.x.0 - runtime.start.x.0,
                    fret_core::Axis::Vertical => mv.position.y.0 - runtime.start.y.0,
                };

                let next = Px((runtime.start_offset.0 - delta).clamp(0.0, max_offset.0));
                let _ = host.models_mut().update(&offset_for_move, |v| *v = next);
                host.request_redraw(_cx.window);
                true
            });

            let runtime_for_up = runtime_model.clone();
            let offset_for_up = offset_model.clone();
            let index_for_up = index_model.clone();
            let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, cx, up| {
                let runtime = host
                    .models_mut()
                    .read(&runtime_for_up, |st| *st)
                    .ok()
                    .unwrap_or_default();
                if !runtime.dragging {
                    return false;
                }

                host.release_pointer_capture();

                let bounds = host.bounds();
                let extent = match item_basis {
                    Some(px) => px,
                    None => match track_direction {
                        fret_core::Axis::Horizontal => bounds.size.width,
                        fret_core::Axis::Vertical => bounds.size.height,
                    },
                };

                let offset = host
                    .models_mut()
                    .read(&offset_for_up, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));

                let max_index = items_len.saturating_sub(1);
                let start_index = if extent.0 > 0.0 {
                    (runtime.start_offset.0 / extent.0)
                        .round()
                        .clamp(0.0, max_index as f32) as usize
                } else {
                    0
                };

                let delta = match track_direction {
                    fret_core::Axis::Horizontal => up.position.x.0 - runtime.start.x.0,
                    fret_core::Axis::Vertical => up.position.y.0 - runtime.start.y.0,
                };

                let mut next_index = start_index;
                if extent.0 > 0.0 {
                    let threshold = extent.0 * 0.25;
                    if delta.abs() > threshold {
                        if delta > 0.0 {
                            next_index = start_index.saturating_sub(1);
                        } else {
                            next_index = (start_index + 1).min(max_index);
                        }
                    } else {
                        next_index = start_index;
                    }
                }

                let target = if extent.0 > 0.0 {
                    Px((next_index as f32) * extent.0)
                } else {
                    Px(0.0)
                };

                let _ = host.models_mut().update(&index_for_up, |v| *v = next_index);
                let _ = host.models_mut().update(&runtime_for_up, |st| {
                    st.dragging = false;
                    st.settling = true;
                    st.settle_from = offset;
                    st.settle_to = target;
                    st.settle_tick = 0;
                });
                host.request_redraw(cx.window);
                true
            });

            let offset_for_prev = offset_model.clone();
            let runtime_for_prev = runtime_model.clone();
            let index_for_prev = index_model.clone();
            let extent_for_prev = extent_model.clone();
            let on_prev: fret_ui::action::OnActivate = Arc::new(
                move |host: &mut dyn UiActionHost, cx: ActionCx, _reason: ActivateReason| {
                    let index: usize = host
                        .models_mut()
                        .read(&index_for_prev, |v| *v)
                        .ok()
                        .unwrap_or(0);
                    if index == 0 {
                        return;
                    }

                    let extent = host
                        .models_mut()
                        .read(&extent_for_prev, |v| *v)
                        .ok()
                        .unwrap_or(Px(0.0));
                    if extent.0 <= 0.0 {
                        return;
                    }

                    let target_index = index.saturating_sub(1);
                    let target = Px((target_index as f32) * extent.0);
                    let cur = host
                        .models_mut()
                        .read(&offset_for_prev, |v| *v)
                        .ok()
                        .unwrap_or(Px(0.0));
                    let _ = host
                        .models_mut()
                        .update(&index_for_prev, |v| *v = target_index);
                    let _ = host.models_mut().update(&runtime_for_prev, |st| {
                        st.dragging = false;
                        st.settling = true;
                        st.settle_from = cur;
                        st.settle_to = target;
                        st.settle_tick = 0;
                    });
                    host.request_redraw(cx.window);
                },
            );

            let offset_for_next = offset_model.clone();
            let runtime_for_next = runtime_model.clone();
            let index_for_next = index_model.clone();
            let extent_for_next = extent_model.clone();
            let on_next: fret_ui::action::OnActivate = Arc::new(
                move |host: &mut dyn UiActionHost, cx: ActionCx, _reason: ActivateReason| {
                    let index: usize = host
                        .models_mut()
                        .read(&index_for_next, |v| *v)
                        .ok()
                        .unwrap_or(0);
                    if index + 1 >= items_len {
                        return;
                    }

                    let extent = host
                        .models_mut()
                        .read(&extent_for_next, |v| *v)
                        .ok()
                        .unwrap_or(Px(0.0));
                    if extent.0 <= 0.0 {
                        return;
                    }

                    let target_index = (index + 1).min(items_len.saturating_sub(1));
                    let target = Px((target_index as f32) * extent.0);
                    let cur = host
                        .models_mut()
                        .read(&offset_for_next, |v| *v)
                        .ok()
                        .unwrap_or(Px(0.0));
                    let _ = host
                        .models_mut()
                        .update(&index_for_next, |v| *v = target_index);
                    let _ = host.models_mut().update(&runtime_for_next, |st| {
                        st.dragging = false;
                        st.settling = true;
                        st.settle_from = cur;
                        st.settle_to = target;
                        st.settle_tick = 0;
                    });
                    host.request_redraw(cx.window);
                },
            );

            let index_for_key = index_model.clone();
            let offset_for_key = offset_model.clone();
            let runtime_for_key = runtime_model.clone();
            let extent_for_key = extent_model.clone();
            let on_key_down: OnKeyDown = Arc::new(
                move |host: &mut dyn fret_ui::action::UiFocusActionHost,
                      cx: ActionCx,
                      down: KeyDownCx| {
                    if items_len <= 1 {
                        return false;
                    }

                    // shadcn/ui v4 Carousel uses left/right keys even when `orientation="vertical"`
                    // (it rotates the controls instead of switching the key mapping).
                    let (prev_key, next_key) = (KeyCode::ArrowLeft, KeyCode::ArrowRight);

                    if down.key != prev_key && down.key != next_key {
                        return false;
                    }

                    let index: usize = host
                        .models_mut()
                        .read(&index_for_key, |v| *v)
                        .ok()
                        .unwrap_or(0);
                    let extent = host
                        .models_mut()
                        .read(&extent_for_key, |v| *v)
                        .ok()
                        .unwrap_or(Px(0.0));
                    if extent.0 <= 0.0 {
                        return true;
                    }

                    let target_index = if down.key == prev_key {
                        if index == 0 {
                            return true;
                        }
                        index.saturating_sub(1)
                    } else {
                        if index + 1 >= items_len {
                            return true;
                        }
                        (index + 1).min(items_len.saturating_sub(1))
                    };

                    let target = Px((target_index as f32) * extent.0);
                    let cur = host
                        .models_mut()
                        .read(&offset_for_key, |v| *v)
                        .ok()
                        .unwrap_or(Px(0.0));
                    let _ = host
                        .models_mut()
                        .update(&index_for_key, |v| *v = target_index);
                    let _ = host.models_mut().update(&runtime_for_key, |st| {
                        st.dragging = false;
                        st.settling = true;
                        st.settle_from = cur;
                        st.settle_to = target;
                        st.settle_tick = 0;
                    });
                    host.request_redraw(cx.window);
                    true
                },
            );

            let track = cx.flex(
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
                                item_layout =
                                    item_layout.basis(LengthRefinement::Px(MetricRef::Px(basis)));
                            } else {
                                // Match shadcn/ui v4 `basis-full` default for both orientations.
                                item_layout = item_layout.basis(LengthRefinement::Fill);
                            }

                            let item_layout =
                                decl_style::layout_style(&theme_for_items, item_layout);
                            let test_id = Arc::from(format!(
                                "{}-item-{}",
                                root_test_id_for_items.as_ref(),
                                idx + 1
                            ));

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

                            let inner = cx.container(
                                fret_ui::element::ContainerProps {
                                    padding,
                                    ..Default::default()
                                },
                                move |_cx| vec![content.clone()],
                            );

                            let item = cx.container(
                                fret_ui::element::ContainerProps {
                                    layout: item_layout,
                                    ..Default::default()
                                },
                                move |_cx| vec![inner],
                            );

                            item.attach_semantics(
                                SemanticsDecoration::default()
                                    .role(SemanticsRole::Group)
                                    .test_id(test_id),
                            )
                        })
                        .collect::<Vec<_>>()
                },
            );

            let track = cx.render_transform_props(
                RenderTransformProps {
                    layout: LayoutStyle::default(),
                    transform,
                },
                move |_cx| vec![track],
            );

            let pointer_layout =
                decl_style::layout_style(&theme, LayoutRefinement::default().size_full());

            let pointer_region = cx.pointer_region(
                PointerRegionProps {
                    layout: pointer_layout,
                    enabled: items_len > 1,
                },
                move |cx| {
                    cx.pointer_region_on_pointer_down(on_down);
                    cx.pointer_region_on_pointer_move(on_move);
                    cx.pointer_region_on_pointer_up(on_up);
                    vec![track]
                },
            );

            let (viewport_id, viewport) = cx.scope(|cx| {
                let id = cx.root_id();
                (
                    id,
                    AnyElement::new(
                        id,
                        ElementKind::Container(ContainerProps {
                            layout: viewport_layout,
                            ..Default::default()
                        }),
                        vec![pointer_region],
                    ),
                )
            });

            let extent_now = match item_basis {
                Some(px) => px,
                None => cx
                    .last_bounds_for_element(viewport_id)
                    .map(|b| match orientation {
                        CarouselOrientation::Horizontal => b.size.width,
                        CarouselOrientation::Vertical => b.size.height,
                    })
                    .unwrap_or(Px(0.0)),
            };
            let _ = cx
                .app
                .models_mut()
                .update(&extent_model, |v| *v = extent_now);

            let prev_disabled = index_now == 0 || items_len <= 1;
            let next_disabled = index_now + 1 >= items_len;

            let prev_test_id = Arc::from(format!("{}-previous", root_test_id.as_ref()));
            let next_test_id = Arc::from(format!("{}-next", root_test_id.as_ref()));

            let rotate_controls = orientation == CarouselOrientation::Vertical;
            let arrow_rotation = if rotate_controls { 90.0 } else { 0.0 };
            let arrow_center = Point::new(Px(8.0), Px(8.0));
            let arrow_transform = fret_core::Transform2D::rotation_about_degrees(
                arrow_rotation,
                arrow_center,
            );
            let arrow_layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default()
                    .w_px(Px(16.0))
                    .h_px(Px(16.0))
                    .flex_shrink_0(),
            );

            let prev_button = Button::new("Previous slide")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::IconSm)
                .disabled(prev_disabled)
                .test_id(prev_test_id)
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .children([cx.visual_transform_props(
                    VisualTransformProps {
                        layout: arrow_layout,
                        transform: arrow_transform,
                    },
                    move |cx| vec![decl_icon::icon(cx, ids::ui::ARROW_LEFT)],
                )])
                .on_activate(on_prev)
                .into_element(cx);

            let next_button = Button::new("Next slide")
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::IconSm)
                .disabled(next_disabled)
                .test_id(next_test_id)
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .children([cx.visual_transform_props(
                    VisualTransformProps {
                        layout: arrow_layout,
                        transform: arrow_transform,
                    },
                    move |cx| vec![decl_icon::icon(cx, ids::ui::ARROW_RIGHT)],
                )])
                .on_activate(on_next)
                .into_element(cx);

            let offset = MetricRef::Px(Px(48.0));
            let button_size = MetricRef::Px(Px(32.0));

            let (prev_layout, next_layout) = match orientation {
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

            let root = cx.container(
                fret_ui::element::ContainerProps {
                    layout: root_layout,
                    ..Default::default()
                },
                move |_cx| vec![viewport, prev_wrapper, next_wrapper],
            );

            cx.key_add_on_key_down_capture_for(root.id, on_key_down);

            root.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(root_test_id),
            )
        })
    }
}
