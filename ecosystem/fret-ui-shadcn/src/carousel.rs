use std::sync::Arc;
use std::time::Duration;

use fret_core::{Edges, KeyCode, LayoutDirection, MouseButton, Point, Px, SemanticsRole};
use fret_icons::ids;
use fret_runtime::{Effect, Model, TimerToken};
use fret_ui::action::{ActionCx, ActivateReason, KeyDownCx, OnKeyDown, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, ElementKind, FlexProps, HoverRegionProps, LayoutStyle,
    MainAlign, PointerRegionProps, RenderTransformProps, SemanticsDecoration, VisualTransformProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::transition as decl_transition;
use fret_ui_kit::headless::carousel as headless_carousel;
use fret_ui_kit::headless::embla as headless_embla;
use fret_ui_kit::headless::snap_points as headless_snap_points;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, LengthRefinement, MetricRef, Radius, Space};

use crate::{Button, ButtonSize, ButtonVariant};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CarouselApiSnapshot {
    /// Zero-based selected slide index.
    pub selected_index: usize,
    /// Total snap count (when measurable).
    pub snap_count: usize,
    pub can_scroll_prev: bool,
    pub can_scroll_next: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarouselAutoplayConfig {
    pub delay: Duration,
    pub stop_on_interaction: bool,
    pub pause_on_hover: bool,
    pub reset_on_hover_leave: bool,
}

impl Default for CarouselAutoplayConfig {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(2000),
            stop_on_interaction: true,
            pause_on_hover: true,
            reset_on_hover_leave: true,
        }
    }
}

impl CarouselAutoplayConfig {
    pub fn new(delay: Duration) -> Self {
        Self {
            delay,
            ..Default::default()
        }
    }

    pub fn stop_on_interaction(mut self, stop_on_interaction: bool) -> Self {
        self.stop_on_interaction = stop_on_interaction;
        self
    }

    pub fn pause_on_hover(mut self, pause_on_hover: bool) -> Self {
        self.pause_on_hover = pause_on_hover;
        self
    }

    pub fn reset_on_hover_leave(mut self, reset_on_hover_leave: bool) -> Self {
        self.reset_on_hover_leave = reset_on_hover_leave;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselAlign {
    Start,
    #[default]
    Center,
    End,
}

impl CarouselAlign {
    fn to_headless(self) -> headless_carousel::CarouselSnapAlign {
        match self {
            CarouselAlign::Start => headless_carousel::CarouselSnapAlign::Start,
            CarouselAlign::Center => headless_carousel::CarouselSnapAlign::Center,
            CarouselAlign::End => headless_carousel::CarouselSnapAlign::End,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarouselSlidesToScroll {
    Fixed(usize),
    Auto,
}

impl Default for CarouselSlidesToScroll {
    fn default() -> Self {
        Self::Fixed(1)
    }
}

impl CarouselSlidesToScroll {
    fn to_headless(self) -> headless_carousel::CarouselSlidesToScrollOption {
        match self {
            CarouselSlidesToScroll::Fixed(n) => {
                headless_carousel::CarouselSlidesToScrollOption::Fixed(n.max(1))
            }
            CarouselSlidesToScroll::Auto => headless_carousel::CarouselSlidesToScrollOption::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselContainScroll {
    None,
    KeepSnaps,
    #[default]
    TrimSnaps,
}

impl CarouselContainScroll {
    fn to_headless(self) -> headless_carousel::CarouselContainScrollOption {
        match self {
            CarouselContainScroll::None => headless_carousel::CarouselContainScrollOption::None,
            CarouselContainScroll::KeepSnaps => {
                headless_carousel::CarouselContainScrollOption::KeepSnaps
            }
            CarouselContainScroll::TrimSnaps => {
                headless_carousel::CarouselContainScrollOption::TrimSnaps
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CarouselOptions {
    pub align: CarouselAlign,
    pub slides_to_scroll: CarouselSlidesToScroll,
    pub contain_scroll: CarouselContainScroll,
    /// Layout direction (Embla `direction`).
    pub direction: LayoutDirection,
    /// Initial selected snap index (Embla `startSnap`).
    pub start_snap: usize,
    /// Whether pointer dragging is enabled (Embla `draggable`).
    pub draggable: bool,
    /// Wrap prev/next selection (note: this is *not* a seamless loop engine yet).
    pub loop_enabled: bool,
    /// Embla-style skipSnaps (best-effort without momentum physics).
    pub skip_snaps: bool,
    /// Drag-free release (settle to the projected offset instead of the nearest snap).
    pub drag_free: bool,
    /// Settle animation duration for the deterministic (non-physics) driver (v1 behavior).
    pub duration: Duration,
    /// Enable the Embla-style headless engine (v2 parity MVP).
    ///
    /// Note: when disabled, Carousel uses the deterministic settle driver (v1 behavior).
    pub embla_engine: bool,
    /// Embla-style `duration` (integrator parameter, default `25`).
    ///
    /// This is *not* a wall-clock duration in milliseconds. See:
    /// - `docs/workstreams/carousel-embla-parity-v2/contracts.md`
    pub embla_duration: f32,
    pub pixel_tolerance_px: f32,
}

impl Default for CarouselOptions {
    fn default() -> Self {
        Self {
            // Match Embla defaults unless examples override via `opts`.
            align: CarouselAlign::Center,
            slides_to_scroll: CarouselSlidesToScroll::Fixed(1),
            contain_scroll: CarouselContainScroll::TrimSnaps,
            direction: LayoutDirection::Ltr,
            start_snap: 0,
            draggable: true,
            loop_enabled: false,
            skip_snaps: false,
            drag_free: false,
            duration: Duration::from_millis(25),
            embla_engine: false,
            embla_duration: 25.0,
            pixel_tolerance_px: 2.0,
        }
    }
}

impl CarouselOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn align(mut self, align: CarouselAlign) -> Self {
        self.align = align;
        self
    }

    pub fn slides_to_scroll(mut self, slides_to_scroll: CarouselSlidesToScroll) -> Self {
        self.slides_to_scroll = slides_to_scroll;
        self
    }

    pub fn contain_scroll(mut self, contain_scroll: CarouselContainScroll) -> Self {
        self.contain_scroll = contain_scroll;
        self
    }

    pub fn direction(mut self, direction: LayoutDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn start_snap(mut self, start_snap: usize) -> Self {
        self.start_snap = start_snap;
        self
    }

    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    pub fn loop_enabled(mut self, loop_enabled: bool) -> Self {
        self.loop_enabled = loop_enabled;
        self
    }

    pub fn skip_snaps(mut self, skip_snaps: bool) -> Self {
        self.skip_snaps = skip_snaps;
        self
    }

    pub fn drag_free(mut self, drag_free: bool) -> Self {
        self.drag_free = drag_free;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn embla_engine(mut self, embla_engine: bool) -> Self {
        self.embla_engine = embla_engine;
        self
    }

    pub fn embla_duration(mut self, duration: f32) -> Self {
        self.embla_duration = duration;
        self
    }

    pub fn pixel_tolerance_px(mut self, pixel_tolerance_px: f32) -> Self {
        self.pixel_tolerance_px = pixel_tolerance_px;
        self
    }
}

#[derive(Debug)]
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
    options: CarouselOptions,
    drag_config: headless_carousel::CarouselDragConfig,
    api_snapshot: Option<Model<CarouselApiSnapshot>>,
    autoplay: Option<CarouselAutoplayConfig>,
    test_id: Option<Arc<str>>,
}

#[derive(Debug, Clone, Copy)]
struct CarouselRuntime {
    drag: headless_carousel::CarouselDragState,
    settling: bool,
    embla_settling: bool,
    settle_from: Px,
    settle_to: Px,
    settle_generation: u64,
    selection_initialized: bool,
    autoplay_token: Option<TimerToken>,
    autoplay_paused: bool,
    autoplay_hovered: bool,
}

impl Default for CarouselRuntime {
    fn default() -> Self {
        Self {
            drag: headless_carousel::CarouselDragState::default(),
            settling: false,
            embla_settling: false,
            settle_from: Px(0.0),
            settle_to: Px(0.0),
            settle_generation: 0,
            selection_initialized: false,
            autoplay_token: None,
            autoplay_paused: false,
            autoplay_hovered: false,
        }
    }
}

#[derive(Default)]
struct CarouselState {
    index: Option<Model<usize>>,
    offset: Option<Model<Px>>,
    runtime: Option<Model<CarouselRuntime>>,
    extent: Option<Model<Px>>,
    snaps: Option<Model<Arc<[Px]>>>,
    max_offset: Option<Model<Px>>,
    embla_engine: Option<Model<Option<headless_embla::engine::Engine>>>,
}

fn carousel_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    start_snap: usize,
) -> (
    Model<usize>,
    Model<Px>,
    Model<CarouselRuntime>,
    Model<Px>,
    Model<Arc<[Px]>>,
    Model<Px>,
    Model<Option<headless_embla::engine::Engine>>,
) {
    let needs_init = cx.with_state(CarouselState::default, |st| {
        st.index.is_none()
            || st.offset.is_none()
            || st.runtime.is_none()
            || st.extent.is_none()
            || st.snaps.is_none()
            || st.max_offset.is_none()
            || st.embla_engine.is_none()
    });

    if needs_init {
        let index = cx.app.models_mut().insert(start_snap);
        let offset = cx.app.models_mut().insert(Px(0.0));
        let runtime = cx.app.models_mut().insert(CarouselRuntime::default());
        let extent = cx.app.models_mut().insert(Px(0.0));
        let snaps: Arc<[Px]> = Arc::from(Vec::<Px>::new());
        let snaps = cx.app.models_mut().insert(snaps);
        let max_offset = cx.app.models_mut().insert(Px(0.0));
        let embla_engine: Option<headless_embla::engine::Engine> = None;
        let embla_engine = cx.app.models_mut().insert(embla_engine);
        cx.with_state(CarouselState::default, |st| {
            st.index = Some(index.clone());
            st.offset = Some(offset.clone());
            st.runtime = Some(runtime.clone());
            st.extent = Some(extent.clone());
            st.snaps = Some(snaps.clone());
            st.max_offset = Some(max_offset.clone());
            st.embla_engine = Some(embla_engine.clone());
        });
        return (
            index,
            offset,
            runtime,
            extent,
            snaps,
            max_offset,
            embla_engine,
        );
    }

    let (index, offset, runtime, extent, snaps, max_offset, embla_engine) =
        cx.with_state(CarouselState::default, |st| {
            (
                st.index.clone().expect("index"),
                st.offset.clone().expect("offset"),
                st.runtime.clone().expect("runtime"),
                st.extent.clone().expect("extent"),
                st.snaps.clone().expect("snaps"),
                st.max_offset.clone().expect("max_offset"),
                st.embla_engine.clone().expect("embla_engine"),
            )
        });
    (
        index,
        offset,
        runtime,
        extent,
        snaps,
        max_offset,
        embla_engine,
    )
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
            options: CarouselOptions::default(),
            drag_config: headless_carousel::CarouselDragConfig::default(),
            api_snapshot: None,
            autoplay: None,
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

    /// Embla-style options that influence snap model generation (alignment, containment, etc).
    pub fn opts(mut self, opts: CarouselOptions) -> Self {
        self.options = opts;
        self
    }

    pub fn drag_config(mut self, config: headless_carousel::CarouselDragConfig) -> Self {
        self.drag_config = config;
        self
    }

    pub fn drag_threshold_px(mut self, threshold: Px) -> Self {
        self.drag_config.drag_threshold_px = threshold.0;
        self
    }

    pub fn snap_threshold_fraction(mut self, threshold: f32) -> Self {
        self.drag_config.snap_threshold_fraction = threshold;
        self
    }

    pub fn touch_prevent_scroll(mut self, enabled: bool) -> Self {
        self.drag_config.touch_prevent_scroll = enabled;
        self
    }

    pub fn touch_scroll_lock_threshold_px(mut self, threshold: Px) -> Self {
        self.drag_config.touch_scroll_lock_threshold_px = threshold.0;
        self
    }

    /// Exposes a small, deterministic API surface as a snapshot model (policy-only).
    ///
    /// This exists to support shadcn-style "API" examples (slide counters, etc.) without exposing
    /// Embla's full imperative API surface.
    pub fn api_snapshot_model(mut self, model: Model<CarouselApiSnapshot>) -> Self {
        self.api_snapshot = Some(model);
        self
    }

    /// Adds an Embla-style autoplay policy surface (shadcn `carousel-plugin` outcome).
    pub fn autoplay(mut self, config: CarouselAutoplayConfig) -> Self {
        self.autoplay = Some(config);
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
            let theme = Theme::global(&*cx.app).snapshot();
            let orientation = self.orientation;
            let options = self.options;
            let autoplay_cfg = self.autoplay;
            let autoplay_stop_on_interaction =
                autoplay_cfg.is_some_and(|cfg| cfg.stop_on_interaction);
            let root_test_id = self.test_id.unwrap_or_else(|| Arc::from("carousel"));

            let (
                index_model,
                offset_model,
                runtime_model,
                extent_model,
                snaps_model,
                max_offset_model,
                embla_engine_model,
            ) = carousel_models(cx, options.start_snap);

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
            let layout_direction = options.direction;

            let items_len = self.items.len();
            let item_basis = self.item_basis_main_px;
            let item_layout_patch = self.item_layout;
            let drag_config = self.drag_config;
            let items = self.items;
            let theme_for_items = theme.clone();
            let root_test_id_for_items = root_test_id.clone();

            let index_now = cx.watch_model(&index_model).copied().unwrap_or(0);
            let mut offset_now = cx.watch_model(&offset_model).copied().unwrap_or(Px(0.0));
            let runtime_snapshot = cx.watch_model(&runtime_model).copied().unwrap_or_default();

            let embla_engine_enabled = options.embla_engine
                || std::env::var("FRET_DEBUG_CAROUSEL_EMBLA_ENGINE")
                    .ok()
                    .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

            if runtime_snapshot.embla_settling {
                let _frames = cx.begin_continuous_frames();

                let max_offset = cx
                    .watch_model(&max_offset_model)
                    .copied()
                    .unwrap_or(Px(0.0));
                let mut next_offset = None;
                let mut settled = false;

                let _ = cx.app.models_mut().update(&embla_engine_model, |v| {
                    let Some(engine) = v.as_mut() else {
                        return;
                    };

                    engine.tick(false);
                    settled = engine.scroll_body.settled();
                    let loc = engine.scroll_body.location();
                    let mapped = Px((-loc).clamp(0.0, max_offset.0.max(0.0)));
                    next_offset = Some(mapped);
                });

                if let Some(next) = next_offset {
                    offset_now = next;
                    let _ = cx.app.models_mut().update(&offset_model, |v| *v = next);
                }

                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                    if settled {
                        st.embla_settling = false;
                    }
                });
            }

            if runtime_snapshot.settling {
                let duration = options.duration;
                let settle_generation = runtime_snapshot.settle_generation;
                let motion = cx.keyed(("carousel-settle", settle_generation), |cx| {
                    decl_transition::drive_transition_with_durations_and_easing_duration_with_mount_behavior(
                        cx,
                        true,
                        duration,
                        duration,
                        crate::overlay_motion::shadcn_ease,
                        true,
                    )
                });
                let next = Px(runtime_snapshot.settle_from.0
                    + (runtime_snapshot.settle_to.0 - runtime_snapshot.settle_from.0)
                        * motion.progress);
                offset_now = next;
                let _ = cx.app.models_mut().update(&offset_model, |v| *v = next);
                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                    if !motion.animating {
                        st.settling = false;
                    }
                });
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
            let embla_engine_for_down = embla_engine_model.clone();
            let autoplay_stop_for_down = autoplay_stop_on_interaction;
            let on_down: fret_ui::action::OnPointerDown = Arc::new(move |host, _cx, down| {
                if down.button != MouseButton::Left {
                    return false;
                }
                if down.hit_is_text_input {
                    return false;
                }

                if autoplay_stop_for_down {
                    let token = host
                        .models_mut()
                        .read(&runtime_for_down, |st| st.autoplay_token)
                        .ok()
                        .flatten();
                    if let Some(token) = token {
                        host.push_effect(Effect::CancelTimer { token });
                    }
                    let _ = host.models_mut().update(&runtime_for_down, |st| {
                        st.autoplay_token = None;
                        st.autoplay_paused = true;
                    });
                }

                let start_offset = host
                    .models_mut()
                    .read(&offset_for_down, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));
                let _ = host.models_mut().update(&runtime_for_down, |st| {
                    headless_carousel::on_pointer_down(
                        &mut st.drag,
                        true,
                        down.position,
                        start_offset,
                    );
                    st.settling = false;
                    st.embla_settling = false;
                });
                let _ = host.models_mut().update(&embla_engine_for_down, |v| {
                    *v = None;
                });
                false
            });

            let runtime_for_move = runtime_model.clone();
            let offset_for_move = offset_model.clone();
            let max_offset_for_move = max_offset_model.clone();
            let dnd_service_for_move = fret_ui_kit::dnd::dnd_service_model(cx);
            let direction_for_move = layout_direction;
            let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, _cx, mv| {
                let runtime = host
                    .models_mut()
                    .read(&runtime_for_move, |st| *st)
                    .ok()
                    .unwrap_or_default();
                if !runtime.drag.armed && !runtime.drag.dragging {
                    return false;
                }

                if fret_ui_kit::dnd::pointer_is_tracking_any_sensor(
                    host.models_mut(),
                    &dnd_service_for_move,
                    _cx.window,
                    mv.pointer_id,
                ) {
                    let _ = host.models_mut().update(&runtime_for_move, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                    });
                    return false;
                }

                if !mv.buttons.left {
                    let _ = host.models_mut().update(&runtime_for_move, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                    });
                    return false;
                }

                let max_offset = host
                    .models_mut()
                    .read(&max_offset_for_move, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));

                let max_offset = if max_offset.0 > 0.0 {
                    max_offset
                } else {
                    // Fall back to the legacy extent-based max offset when snap geometry has not
                    // been measured yet (common in single-frame tests and before the first render
                    // loop settles).
                    let bounds = host.bounds();
                    let extent = match item_basis {
                        Some(px) => px,
                        None => match track_direction {
                            fret_core::Axis::Horizontal => bounds.size.width,
                            fret_core::Axis::Vertical => bounds.size.height,
                        },
                    };
                    Px((extent.0 * (items_len.saturating_sub(1) as f32)).max(0.0))
                };

                if max_offset.0 <= 0.0 {
                    let _ = host.models_mut().update(&runtime_for_move, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                    });
                    return false;
                }

                let mut steal_capture = false;
                let mut consumed = false;
                let mut next_offset = None;

                let _ = host.models_mut().update(&runtime_for_move, |st| {
                    let out = headless_carousel::on_pointer_move(
                        drag_config,
                        &mut st.drag,
                        track_direction,
                        direction_for_move,
                        mv.position,
                        mv.buttons.left,
                        match mv.pointer_type {
                            fret_core::PointerType::Touch => {
                                headless_carousel::CarouselDragInputKind::Touch
                            }
                            _ => headless_carousel::CarouselDragInputKind::Mouse,
                        },
                        max_offset,
                    );
                    steal_capture = out.steal_capture;
                    consumed = out.consumed;
                    next_offset = out.next_offset;
                });

                if steal_capture {
                    host.capture_pointer();
                }

                if let Some(next) = next_offset {
                    let _ = host.models_mut().update(&offset_for_move, |v| *v = next);
                    host.request_redraw(_cx.window);
                }

                consumed
            });

            let runtime_for_up = runtime_model.clone();
            let offset_for_up = offset_model.clone();
            let index_for_up = index_model.clone();
            let snaps_for_up = snaps_model.clone();
            let max_offset_for_up = max_offset_model.clone();
            let embla_engine_for_up = embla_engine_model.clone();
            let embla_engine_enabled_for_up = embla_engine_enabled;
            let embla_duration_for_up = options.embla_duration;
            let skip_snaps_for_up = options.skip_snaps;
            let drag_free_for_up = options.drag_free;
            let loop_for_up = options.loop_enabled;
            let direction_for_up = layout_direction;
            let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, cx, up| {
                let runtime = host
                    .models_mut()
                    .read(&runtime_for_up, |st| *st)
                    .ok()
                    .unwrap_or_default();
                if !runtime.drag.dragging {
                    let _ = host.models_mut().update(&runtime_for_up, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                    });
                    return false;
                }

                host.release_pointer_capture();

                let offset = host
                    .models_mut()
                    .read(&offset_for_up, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));
                let max_offset = host
                    .models_mut()
                    .read(&max_offset_for_up, |v| *v)
                    .ok()
                    .unwrap_or(Px(0.0));

                let mut drag = runtime.drag;
                let snaps: Arc<[Px]> = host
                    .models_mut()
                    .read(&snaps_for_up, |v| v.clone())
                    .ok()
                    .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));

                let can_use_embla_engine = embla_engine_enabled_for_up
                    && !loop_for_up
                    && snaps.len() > 1
                    && up.velocity_window.is_some();

                if can_use_embla_engine {
                    let bounds = host.bounds();
                    let view_size = match track_direction {
                        fret_core::Axis::Horizontal => bounds.size.width.0.max(0.0),
                        fret_core::Axis::Vertical => bounds.size.height.0.max(0.0),
                    };

                    let content_size = if max_offset.0 > 0.0 {
                        max_offset.0
                    } else {
                        let extent = match item_basis {
                            Some(px) => px,
                            None => match track_direction {
                                fret_core::Axis::Horizontal => bounds.size.width,
                                fret_core::Axis::Vertical => bounds.size.height,
                            },
                        };
                        (extent.0 * (items_len.saturating_sub(1) as f32)).max(0.0)
                    };

                    let mut scroll_snaps = snaps.iter().map(|px| -px.0).collect::<Vec<_>>();
                    if scroll_snaps.is_empty() {
                        scroll_snaps.push(0.0);
                    }

                    let index: usize = host
                        .models_mut()
                        .read(&index_for_up, |v| *v)
                        .ok()
                        .unwrap_or(0);

                    let mut engine = headless_embla::engine::Engine::new(
                        scroll_snaps,
                        content_size,
                        headless_embla::engine::EngineConfig {
                            loop_enabled: false,
                            drag_free: drag_free_for_up,
                            skip_snaps: skip_snaps_for_up,
                            duration: embla_duration_for_up.max(0.0),
                            base_friction: 0.68,
                            view_size,
                            start_snap: index,
                        },
                    );

                    // Sync engine state to the current drag location (Embla updates target while
                    // dragging; our v1 path directly updates the offset model).
                    let engine_location = -offset.0;
                    engine.scroll_body.set_location(engine_location);
                    engine.scroll_body.set_target(engine_location);
                    engine.scroll_target.set_target_vector(engine_location);

                    let pointer_kind = match up.pointer_type {
                        fret_core::PointerType::Touch => headless_embla::drag_release::PointerKind::Touch,
                        _ => headless_embla::drag_release::PointerKind::Mouse,
                    };
                    let velocity = up
                        .velocity_window
                        .map(|v| match track_direction {
                            fret_core::Axis::Horizontal => v.x.0,
                            fret_core::Axis::Vertical => v.y.0,
                        })
                        .unwrap_or(0.0);
                    // Embla uses px/ms (`event.timeStamp` is ms). Fret provides px/s.
                    let velocity = velocity / 1000.0;

                    let sign = if track_direction == fret_core::Axis::Horizontal
                        && direction_for_up == LayoutDirection::Rtl
                    {
                        -1.0
                    } else {
                        1.0
                    };
                    let (_release, select) =
                        engine.on_drag_release(pointer_kind, velocity, move |v| v * sign);

                    let _ = host.models_mut().update(&embla_engine_for_up, |v| {
                        *v = Some(engine);
                    });

                    if let Some(select) = select {
                        let _ = host
                            .models_mut()
                            .update(&index_for_up, |v| *v = select.target_snap);
                    }

                    let _ = host.models_mut().update(&runtime_for_up, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                        st.settling = false;
                        st.embla_settling = true;
                    });
                    host.request_redraw(cx.window);
                    return true;
                }
                let release = if snaps.len() > 1 {
                    headless_carousel::on_pointer_up_with_snaps_options(
                        drag_config,
                        &mut drag,
                        track_direction,
                        direction_for_up,
                        up.position,
                        &snaps,
                        max_offset,
                        loop_for_up,
                        skip_snaps_for_up,
                        drag_free_for_up,
                    )
                    .expect("release output when dragging")
                } else {
                    let bounds = host.bounds();
                    let extent = match item_basis {
                        Some(px) => px,
                        None => match track_direction {
                            fret_core::Axis::Horizontal => bounds.size.width,
                            fret_core::Axis::Vertical => bounds.size.height,
                        },
                    };
                    headless_carousel::on_pointer_up(
                        drag_config,
                        &mut drag,
                        track_direction,
                        direction_for_up,
                        up.position,
                        extent,
                        items_len,
                    )
                    .expect("release output when dragging")
                };

                let _ = host
                    .models_mut()
                    .update(&index_for_up, |v| *v = release.next_index);
                let _ = host.models_mut().update(&runtime_for_up, |st| {
                    st.drag = headless_carousel::CarouselDragState::default();
                    st.settling = true;
                    st.embla_settling = false;
                    st.settle_from = offset;
                    st.settle_to = release.target_offset;
                    st.settle_generation = st.settle_generation.saturating_add(1);
                });
                host.request_redraw(cx.window);
                true
            });

            let runtime_for_cancel = runtime_model.clone();
            let embla_engine_for_cancel = embla_engine_model.clone();
            let on_cancel: fret_ui::action::OnPointerCancel = Arc::new(move |host, cx, _cancel| {
                let runtime = host
                    .models_mut()
                    .read(&runtime_for_cancel, |st| *st)
                    .ok()
                    .unwrap_or_default();
                if !runtime.drag.dragging && !runtime.drag.armed {
                    return false;
                }

                host.release_pointer_capture();
                let _ = host.models_mut().update(&runtime_for_cancel, |st| {
                    st.drag = headless_carousel::CarouselDragState::default();
                    st.settling = false;
                    st.embla_settling = false;
                });
                let _ = host.models_mut().update(&embla_engine_for_cancel, |v| {
                    *v = None;
                });
                host.request_redraw(cx.window);
                true
            });

            let offset_for_prev = offset_model.clone();
            let runtime_for_prev = runtime_model.clone();
            let index_for_prev = index_model.clone();
            let snaps_for_prev = snaps_model.clone();
            let embla_engine_for_prev = embla_engine_model.clone();
            let max_offset_for_prev = max_offset_model.clone();
            let extent_for_prev = extent_model.clone();
            let autoplay_stop_for_prev = autoplay_stop_on_interaction;
            let loop_for_prev = options.loop_enabled;
            let embla_engine_enabled_for_prev = embla_engine_enabled;
            let embla_duration_for_prev = options.embla_duration;
            let on_prev: fret_ui::action::OnActivate = Arc::new(
                move |host: &mut dyn UiActionHost, cx: ActionCx, _reason: ActivateReason| {
                    if autoplay_stop_for_prev {
                        let token = host
                            .models_mut()
                            .read(&runtime_for_prev, |st| st.autoplay_token)
                            .ok()
                            .flatten();
                        if let Some(token) = token {
                            host.push_effect(Effect::CancelTimer { token });
                        }
                        let _ = host.models_mut().update(&runtime_for_prev, |st| {
                            st.autoplay_token = None;
                            st.autoplay_paused = true;
                        });
                    }

                    let snaps: Arc<[Px]> = host
                        .models_mut()
                        .read(&snaps_for_prev, |v| v.clone())
                        .ok()
                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));
                    let index: usize = host
                        .models_mut()
                        .read(&index_for_prev, |v| *v)
                        .ok()
                        .unwrap_or(0);
                    if snaps.len() <= 1 {
                        return;
                    }

                    if embla_engine_enabled_for_prev && !loop_for_prev {
                        let view_size = host
                            .models_mut()
                            .read(&extent_for_prev, |v| v.0.max(0.0))
                            .ok()
                            .unwrap_or(0.0);
                        let max_offset = host
                            .models_mut()
                            .read(&max_offset_for_prev, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let content_size = max_offset.0.max(0.0);
                        let mut scroll_snaps = snaps.iter().map(|px| -px.0).collect::<Vec<_>>();
                        if scroll_snaps.is_empty() {
                            scroll_snaps.push(0.0);
                        }

                        let mut engine = headless_embla::engine::Engine::new(
                            scroll_snaps,
                            content_size,
                            headless_embla::engine::EngineConfig {
                                loop_enabled: false,
                                drag_free: false,
                                skip_snaps: false,
                                duration: embla_duration_for_prev.max(0.0),
                                base_friction: 0.68,
                                view_size,
                                start_snap: index,
                            },
                        );
                        let cur = host
                            .models_mut()
                            .read(&offset_for_prev, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let loc = -cur.0;
                        engine.scroll_body.set_location(loc);
                        engine.scroll_body.set_target(loc);
                        engine.scroll_target.set_target_vector(loc);
                        let select = engine.scroll_to_prev();

                        let _ = host.models_mut().update(&embla_engine_for_prev, |v| {
                            *v = Some(engine);
                        });
                        if let Some(select) = select {
                            let _ = host
                                .models_mut()
                                .update(&index_for_prev, |v| *v = select.target_snap);
                        }
                        let _ = host.models_mut().update(&runtime_for_prev, |st| {
                            st.drag = headless_carousel::CarouselDragState::default();
                            st.settling = false;
                            st.embla_settling = true;
                        });
                        host.request_redraw(cx.window);
                        return;
                    }

                    let target_index = if loop_for_prev {
                        headless_snap_points::step_index_wrapped(snaps.len(), index, -1)
                    } else {
                        headless_snap_points::step_index_clamped(snaps.len(), index, -1)
                    };
                    let Some(target_index) = target_index else {
                        return;
                    };
                    if target_index == index {
                        return;
                    }
                    let target = snaps[target_index];
                    let cur = host
                        .models_mut()
                        .read(&offset_for_prev, |v| *v)
                        .ok()
                        .unwrap_or(Px(0.0));
                    let _ = host
                        .models_mut()
                        .update(&index_for_prev, |v| *v = target_index);
                    let _ = host.models_mut().update(&runtime_for_prev, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                        st.settling = true;
                        st.embla_settling = false;
                        st.settle_from = cur;
                        st.settle_to = target;
                        st.settle_generation = st.settle_generation.saturating_add(1);
                    });
                    host.request_redraw(cx.window);
                },
            );

            let offset_for_next = offset_model.clone();
            let runtime_for_next = runtime_model.clone();
            let index_for_next = index_model.clone();
            let snaps_for_next = snaps_model.clone();
            let embla_engine_for_next = embla_engine_model.clone();
            let max_offset_for_next = max_offset_model.clone();
            let extent_for_next = extent_model.clone();
            let autoplay_stop_for_next = autoplay_stop_on_interaction;
            let loop_for_next = options.loop_enabled;
            let embla_engine_enabled_for_next = embla_engine_enabled;
            let embla_duration_for_next = options.embla_duration;
            let on_next: fret_ui::action::OnActivate = Arc::new(
                move |host: &mut dyn UiActionHost, cx: ActionCx, _reason: ActivateReason| {
                    if autoplay_stop_for_next {
                        let token = host
                            .models_mut()
                            .read(&runtime_for_next, |st| st.autoplay_token)
                            .ok()
                            .flatten();
                        if let Some(token) = token {
                            host.push_effect(Effect::CancelTimer { token });
                        }
                        let _ = host.models_mut().update(&runtime_for_next, |st| {
                            st.autoplay_token = None;
                            st.autoplay_paused = true;
                        });
                    }

                    let snaps: Arc<[Px]> = host
                        .models_mut()
                        .read(&snaps_for_next, |v| v.clone())
                        .ok()
                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));
                    let index: usize = host
                        .models_mut()
                        .read(&index_for_next, |v| *v)
                        .ok()
                        .unwrap_or(0);
                    if snaps.len() <= 1 {
                        return;
                    }

                    if embla_engine_enabled_for_next && !loop_for_next {
                        let view_size = host
                            .models_mut()
                            .read(&extent_for_next, |v| v.0.max(0.0))
                            .ok()
                            .unwrap_or(0.0);
                        let max_offset = host
                            .models_mut()
                            .read(&max_offset_for_next, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let content_size = max_offset.0.max(0.0);
                        let mut scroll_snaps = snaps.iter().map(|px| -px.0).collect::<Vec<_>>();
                        if scroll_snaps.is_empty() {
                            scroll_snaps.push(0.0);
                        }

                        let mut engine = headless_embla::engine::Engine::new(
                            scroll_snaps,
                            content_size,
                            headless_embla::engine::EngineConfig {
                                loop_enabled: false,
                                drag_free: false,
                                skip_snaps: false,
                                duration: embla_duration_for_next.max(0.0),
                                base_friction: 0.68,
                                view_size,
                                start_snap: index,
                            },
                        );
                        let cur = host
                            .models_mut()
                            .read(&offset_for_next, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let loc = -cur.0;
                        engine.scroll_body.set_location(loc);
                        engine.scroll_body.set_target(loc);
                        engine.scroll_target.set_target_vector(loc);
                        let select = engine.scroll_to_next();

                        let _ = host.models_mut().update(&embla_engine_for_next, |v| {
                            *v = Some(engine);
                        });
                        if let Some(select) = select {
                            let _ = host
                                .models_mut()
                                .update(&index_for_next, |v| *v = select.target_snap);
                        }
                        let _ = host.models_mut().update(&runtime_for_next, |st| {
                            st.drag = headless_carousel::CarouselDragState::default();
                            st.settling = false;
                            st.embla_settling = true;
                        });
                        host.request_redraw(cx.window);
                        return;
                    }

                    let target_index = if loop_for_next {
                        headless_snap_points::step_index_wrapped(snaps.len(), index, 1)
                    } else {
                        headless_snap_points::step_index_clamped(snaps.len(), index, 1)
                    };
                    let Some(target_index) = target_index else {
                        return;
                    };
                    if target_index == index {
                        return;
                    }
                    let target = snaps[target_index];
                    let cur = host
                        .models_mut()
                        .read(&offset_for_next, |v| *v)
                        .ok()
                        .unwrap_or(Px(0.0));
                    let _ = host
                        .models_mut()
                        .update(&index_for_next, |v| *v = target_index);
                    let _ = host.models_mut().update(&runtime_for_next, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                        st.settling = true;
                        st.embla_settling = false;
                        st.settle_from = cur;
                        st.settle_to = target;
                        st.settle_generation = st.settle_generation.saturating_add(1);
                    });
                    host.request_redraw(cx.window);
                },
            );

            let index_for_key = index_model.clone();
            let offset_for_key = offset_model.clone();
            let runtime_for_key = runtime_model.clone();
            let snaps_for_key = snaps_model.clone();
            let autoplay_stop_for_key = autoplay_stop_on_interaction;
            let loop_for_key = options.loop_enabled;
            let direction_for_key = layout_direction;
            let on_key_down: OnKeyDown = Arc::new(
                move |host: &mut dyn fret_ui::action::UiFocusActionHost,
                      cx: ActionCx,
                      down: KeyDownCx| {
                    let snaps: Arc<[Px]> = host
                        .models_mut()
                        .read(&snaps_for_key, |v| v.clone())
                        .ok()
                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));
                    if snaps.len() <= 1 {
                        return false;
                    }

                    // shadcn/ui v4 Carousel uses left/right keys even when `orientation="vertical"`
                    // (it rotates the controls instead of switching the key mapping).
                    let (prev_key, next_key) = match direction_for_key {
                        LayoutDirection::Rtl => (KeyCode::ArrowRight, KeyCode::ArrowLeft),
                        LayoutDirection::Ltr => (KeyCode::ArrowLeft, KeyCode::ArrowRight),
                    };

                    if down.key != prev_key && down.key != next_key {
                        return false;
                    }

                    if autoplay_stop_for_key {
                        let token = host
                            .models_mut()
                            .read(&runtime_for_key, |st| st.autoplay_token)
                            .ok()
                            .flatten();
                        if let Some(token) = token {
                            host.push_effect(Effect::CancelTimer { token });
                        }
                        let _ = host.models_mut().update(&runtime_for_key, |st| {
                            st.autoplay_token = None;
                            st.autoplay_paused = true;
                        });
                    }

                    let index: usize = host
                        .models_mut()
                        .read(&index_for_key, |v| *v)
                        .ok()
                        .unwrap_or(0);

                    let delta = if down.key == prev_key { -1 } else { 1 };
                    let target_index = if loop_for_key {
                        headless_snap_points::step_index_wrapped(snaps.len(), index, delta)
                    } else {
                        headless_snap_points::step_index_clamped(snaps.len(), index, delta)
                    };
                    let Some(target_index) = target_index else {
                        return true;
                    };
                    if target_index == index {
                        return true;
                    }

                    let target = snaps[target_index];
                    let cur = host
                        .models_mut()
                        .read(&offset_for_key, |v| *v)
                        .ok()
                        .unwrap_or(Px(0.0));
                    let _ = host
                        .models_mut()
                        .update(&index_for_key, |v| *v = target_index);
                    let _ = host.models_mut().update(&runtime_for_key, |st| {
                        st.drag = headless_carousel::CarouselDragState::default();
                        st.settling = true;
                        st.settle_from = cur;
                        st.settle_to = target;
                        st.settle_generation = st.settle_generation.saturating_add(1);
                    });
                    host.request_redraw(cx.window);
                    true
                },
            );

            let mut item_ids = Vec::with_capacity(items_len);
            let item_ids_ref = &mut item_ids;
            let track = cx.flex(
                FlexProps {
                    layout: track_layout,
                    direction: track_direction,
                    align: CrossAlign::Stretch,
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

                                // When an explicit item basis is provided, treat it as the
                                // authoritative snap extent and clamp the item's main-axis size
                                // to match. This keeps `item_basis_main_px` deterministic even
                                // when children would otherwise expand the flex item.
                                if track_direction == fret_core::Axis::Vertical {
                                    item_layout = item_layout
                                        .h_px(MetricRef::Px(basis))
                                        .min_h(MetricRef::Px(basis))
                                        .max_h(MetricRef::Px(basis));
                                }
                            } else {
                                // Match shadcn/ui v4 `basis-full` default for horizontal tracks.
                                //
                                // For vertical tracks, the upstream demo uses `md:basis-1/2` and
                                // relies on `min-height: auto` to clamp items to their content.
                                // Since we don't have breakpoint-aware layout here, default to an
                                // auto basis so vertical item geometry remains content-driven in
                                // common layouts (matching our web goldens).
                                item_layout = match track_direction {
                                    fret_core::Axis::Horizontal => {
                                        item_layout.basis(LengthRefinement::Fill)
                                    }
                                    fret_core::Axis::Vertical => {
                                        item_layout.basis(LengthRefinement::Auto)
                                    }
                                };
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

                            let item = cx.container(
                                fret_ui::element::ContainerProps {
                                    layout: item_layout,
                                    padding: padding.into(),
                                    ..Default::default()
                                },
                                move |_cx| vec![content],
                            );

                            item_ids_ref.push(item.id);

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

            let drag_enabled = items_len > 1 && options.draggable;
            let pointer_region = cx.pointer_region(
                PointerRegionProps {
                    layout: pointer_layout,
                    enabled: drag_enabled,
                    capture_phase_pointer_moves: true,
                },
                move |cx| {
                    cx.pointer_region_on_pointer_down(on_down);
                    cx.pointer_region_on_pointer_move(on_move);
                    cx.pointer_region_on_pointer_up(on_up);
                    cx.pointer_region_on_pointer_cancel(on_cancel);
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

            let viewport_bounds = cx.last_bounds_for_element(viewport_id);
            let view_size_now = viewport_bounds
                .map(|b| match orientation {
                    CarouselOrientation::Horizontal => b.size.width,
                    CarouselOrientation::Vertical => b.size.height,
                })
                .unwrap_or(Px(0.0));
            let _ = cx
                .app
                .models_mut()
                .update(&extent_model, |v| *v = view_size_now);

            let mut snaps_now: Vec<Px> = vec![Px(0.0)];
            let mut max_offset_now = Px(0.0);

            if view_size_now.0 > 0.0 {
                if let Some(viewport_bounds) = viewport_bounds {
                    let mut slides = Vec::with_capacity(items_len);
                    for id in &item_ids {
                        let Some(bounds) = cx.last_bounds_for_element(*id) else {
                            continue;
                        };

                        let (start, size) = match orientation {
                            CarouselOrientation::Horizontal => (
                                Px(bounds.origin.x.0 - viewport_bounds.origin.x.0),
                                bounds.size.width,
                            ),
                            CarouselOrientation::Vertical => (
                                Px(bounds.origin.y.0 - viewport_bounds.origin.y.0),
                                bounds.size.height,
                            ),
                        };
                        slides.push(headless_carousel::CarouselSlide1D { start, size });
                    }

                    let mut start_gap = Px(0.0);
                    if slides.len() == items_len {
                        let min_start = slides
                            .iter()
                            .map(|s| s.start)
                            .fold(Px(f32::INFINITY), |a, b| Px(a.0.min(b.0)));
                        if min_start.0.is_finite() && min_start.0 > 0.0 {
                            start_gap = min_start;
                            for s in &mut slides {
                                s.start = Px(s.start.0 - min_start.0);
                            }
                        }
                    }

                    let slides_ready =
                        slides.len() == items_len && slides.iter().all(|s| s.size.0 > 0.0);

                    if slides_ready {
                        let model = headless_carousel::snap_model_1d(
                            view_size_now,
                            &slides,
                            start_gap,
                            Px(0.0),
                            options.slides_to_scroll.to_headless(),
                            false,
                            options.align.to_headless(),
                            options.contain_scroll.to_headless(),
                            options.pixel_tolerance_px.max(0.0),
                        );
                        snaps_now = model.snaps_px;
                        max_offset_now = model.max_offset_px;
                    } else if let Some(first_item_id) = item_ids.first().copied()
                        && let Some(first_bounds) = cx.last_bounds_for_element(first_item_id)
                    {
                        // Fallback: keep the previous "uniform item" approximation when we don't
                        // have stable bounds for all items yet (e.g. first frame).
                        let (first_start, item_main_size) = match orientation {
                            CarouselOrientation::Horizontal => (
                                Px(first_bounds.origin.x.0 - viewport_bounds.origin.x.0),
                                first_bounds.size.width,
                            ),
                            CarouselOrientation::Vertical => (
                                Px(first_bounds.origin.y.0 - viewport_bounds.origin.y.0),
                                first_bounds.size.height,
                            ),
                        };

                        if item_main_size.0 > 0.0 {
                            let slides = (0..items_len)
                                .map(|i| headless_carousel::CarouselSlide1D {
                                    start: Px(first_start.0 + (i as f32) * item_main_size.0),
                                    size: item_main_size,
                                })
                                .collect::<Vec<_>>();

                            let model = headless_carousel::snap_model_1d(
                                view_size_now,
                                &slides,
                                Px(0.0),
                                Px(0.0),
                                options.slides_to_scroll.to_headless(),
                                false,
                                options.align.to_headless(),
                                options.contain_scroll.to_headless(),
                                options.pixel_tolerance_px.max(0.0),
                            );
                            snaps_now = model.snaps_px;
                            max_offset_now = model.max_offset_px;
                        }
                    }
                };
            }

            let snaps_arc: Arc<[Px]> = Arc::from(snaps_now.clone().into_boxed_slice());
            let _ = cx
                .app
                .models_mut()
                .update(&snaps_model, |v| *v = snaps_arc.clone());
            let _ = cx
                .app
                .models_mut()
                .update(&max_offset_model, |v| *v = max_offset_now);

            // Clamp index/offset when snaps change (e.g. window resize).
            let snaps_len = snaps_now.len();
            let selection_source_index = if !runtime_snapshot.selection_initialized
                && !runtime_snapshot.settling
                && !runtime_snapshot.drag.dragging
            {
                options.start_snap
            } else {
                index_now
            };
            let clamped_index = if snaps_len == 0 {
                // Keep the requested selection until we have measurable snaps to clamp against.
                selection_source_index
            } else {
                selection_source_index.min(snaps_len.saturating_sub(1))
            };
            if snaps_len > 0 && clamped_index != index_now {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&index_model, |v| *v = clamped_index);
            }
            let offset_clamped = Px(offset_now.0.clamp(0.0, max_offset_now.0.max(0.0)));
            if offset_clamped != offset_now {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&offset_model, |v| *v = offset_clamped);
            }

            // Match upstream behavior: controls are disabled until the carousel has a measurable
            // viewport/item extent (Embla initializes canScrollPrev/Next to false until it
            // measures and emits `select`/`reInit`).
            let extent_ready = view_size_now.0 > 0.0 && snaps_len > 0;

            // Embla's `startSnap` selects the initial snap before the user interacts. Because this
            // recipe derives snaps from measured geometry (available after at least one layout
            // pass), we apply the initial snap once snaps become available.
            if extent_ready
                && !runtime_snapshot.selection_initialized
                && !runtime_snapshot.settling
                && !runtime_snapshot.drag.dragging
            {
                let target = snaps_now
                    .get(clamped_index)
                    .copied()
                    .unwrap_or(Px(0.0));
                let target = Px(target.0.clamp(0.0, max_offset_now.0.max(0.0)));
                let _ = cx.app.models_mut().update(&offset_model, |v| *v = target);
                let _ = cx.app.models_mut().update(&runtime_model, |st| {
                    st.selection_initialized = true;
                });
                cx.request_frame();
            }

            let prev_disabled =
                !extent_ready || snaps_len <= 1 || (!options.loop_enabled && clamped_index == 0);
            let next_disabled = !extent_ready
                || snaps_len <= 1
                || (!options.loop_enabled && clamped_index + 1 >= snaps_len);

            if let Some(api_snapshot) = self.api_snapshot {
                let snapshot = CarouselApiSnapshot {
                    selected_index: clamped_index,
                    snap_count: if extent_ready { snaps_len } else { 0 },
                    can_scroll_prev: !prev_disabled,
                    can_scroll_next: !next_disabled,
                };
                let _ = cx.app.models_mut().update(&api_snapshot, |v| *v = snapshot);
            }

            let prev_test_id = Arc::from(format!("{}-previous", root_test_id.as_ref()));
            let next_test_id = Arc::from(format!("{}-next", root_test_id.as_ref()));

            let rotate_controls = orientation == CarouselOrientation::Vertical;
            let arrow_rotation = if rotate_controls { 90.0 } else { 0.0 };
            let arrow_center = Point::new(Px(8.0), Px(8.0));
            let arrow_transform =
                fret_core::Transform2D::rotation_about_degrees(arrow_rotation, arrow_center);
            let arrow_layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default()
                    .w_px(Px(16.0))
                    .h_px(Px(16.0))
                    .flex_shrink_0(),
            );

            let rtl_controls =
                layout_direction == LayoutDirection::Rtl && orientation == CarouselOrientation::Horizontal;
            let (prev_icon, next_icon) = if rtl_controls {
                (ids::ui::ARROW_RIGHT, ids::ui::ARROW_LEFT)
            } else {
                (ids::ui::ARROW_LEFT, ids::ui::ARROW_RIGHT)
            };

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
                    move |cx| vec![decl_icon::icon(cx, prev_icon)],
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
                    move |cx| vec![decl_icon::icon(cx, next_icon)],
                )])
                .on_activate(on_next)
                .into_element(cx);

            let offset = MetricRef::Px(Px(48.0));
            let button_size = MetricRef::Px(Px(32.0));

            let (prev_layout, next_layout) = match orientation {
                CarouselOrientation::Horizontal => {
                    if rtl_controls {
                        (
                            LayoutRefinement::default()
                                .absolute()
                                .top(Space::N0)
                                .bottom(Space::N0)
                                .right_neg_px(offset.clone())
                                .w_px(button_size.clone()),
                            LayoutRefinement::default()
                                .absolute()
                                .top(Space::N0)
                                .bottom(Space::N0)
                                .left_neg_px(offset)
                                .w_px(button_size),
                        )
                    } else {
                        (
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
                        )
                    }
                }
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

            let pointer_can_hover =
                fret_ui_kit::declarative::primary_pointer_can_hover(cx, Invalidation::Layout, true);
            let runtime_for_hover = runtime_model.clone();
            let snaps_for_hover = snaps_model.clone();
            let theme_for_hover = theme.clone();
            let root = cx.container(
                ContainerProps {
                    layout: root_layout,
                    ..Default::default()
                },
                move |cx| {
                    let hover_overlay = autoplay_cfg.map(|cfg| {
                        cx.hover_region(
                            HoverRegionProps {
                                layout: decl_style::layout_style(
                                    &theme_for_hover,
                                    LayoutRefinement::default()
                                        .absolute()
                                        .top(Space::N0)
                                        .right(Space::N0)
                                        .bottom(Space::N0)
                                        .left(Space::N0),
                                ),
                            },
                            move |cx, hovered| {
                                let hovered = hovered && pointer_can_hover;

                                let runtime_snapshot = cx
                                    .watch_model(&runtime_for_hover)
                                    .copied()
                                    .unwrap_or_default();

                                let was_hovered = runtime_snapshot.autoplay_hovered;
                                let left_hover = was_hovered && !hovered;
                                let entered_hover = !was_hovered && hovered;

                                if was_hovered != hovered {
                                    let _ = cx.app.models_mut().update(&runtime_for_hover, |st| {
                                        st.autoplay_hovered = hovered;
                                    });
                                }

                                if entered_hover && cfg.pause_on_hover {
                                    if let Some(token) = runtime_snapshot.autoplay_token {
                                        cx.app.push_effect(Effect::CancelTimer { token });
                                    }
                                    let _ = cx.app.models_mut().update(&runtime_for_hover, |st| {
                                        st.autoplay_token = None;
                                    });
                                }

                                if left_hover
                                    && cfg.reset_on_hover_leave
                                    && !runtime_snapshot.autoplay_paused
                                {
                                    let snaps: Arc<[Px]> = cx
                                        .watch_model(&snaps_for_hover)
                                        .layout()
                                        .cloned()
                                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));

                                    if snaps.len() > 1 && runtime_snapshot.autoplay_token.is_none()
                                    {
                                        let token = cx.app.next_timer_token();
                                        let _ =
                                            cx.app.models_mut().update(&runtime_for_hover, |st| {
                                                st.autoplay_token = Some(token);
                                            });
                                        cx.app.push_effect(Effect::SetTimer {
                                            window: Some(cx.window),
                                            token,
                                            after: cfg.delay,
                                            repeat: None,
                                        });
                                    }
                                }

                                if !hovered
                                    && !runtime_snapshot.autoplay_paused
                                    && runtime_snapshot.autoplay_token.is_none()
                                {
                                    let snaps: Arc<[Px]> = cx
                                        .watch_model(&snaps_for_hover)
                                        .layout()
                                        .cloned()
                                        .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));

                                    if snaps.len() > 1 {
                                        let token = cx.app.next_timer_token();
                                        let _ =
                                            cx.app.models_mut().update(&runtime_for_hover, |st| {
                                                st.autoplay_token = Some(token);
                                            });
                                        cx.app.push_effect(Effect::SetTimer {
                                            window: Some(cx.window),
                                            token,
                                            after: cfg.delay,
                                            repeat: None,
                                        });
                                    }
                                }

                                Vec::new()
                            },
                        )
                    });

                    let mut children = vec![viewport, prev_wrapper, next_wrapper];
                    if let Some(hover_overlay) = hover_overlay {
                        children.push(hover_overlay);
                    }
                    children
                },
            );

            if let Some(cfg) = autoplay_cfg {
                let runtime_for_timer = runtime_model.clone();
                let snaps_for_timer = snaps_model.clone();
                let index_for_timer = index_model.clone();
                let offset_for_timer = offset_model.clone();
                let loop_for_timer = options.loop_enabled;
                cx.timer_on_timer_for(
                    root.id,
                    Arc::new(move |host, action_cx, token| {
                        let runtime = host
                            .models_mut()
                            .read(&runtime_for_timer, |st| *st)
                            .ok()
                            .unwrap_or_default();
                        if runtime.autoplay_token.as_ref() != Some(&token) {
                            return false;
                        }
                        if runtime.autoplay_paused
                            || (cfg.pause_on_hover && runtime.autoplay_hovered)
                        {
                            let _ = host.models_mut().update(&runtime_for_timer, |st| {
                                st.autoplay_token = None;
                            });
                            return true;
                        }

                        let snaps: Arc<[Px]> = host
                            .models_mut()
                            .read(&snaps_for_timer, |v| v.clone())
                            .ok()
                            .unwrap_or_else(|| Arc::from(Vec::<Px>::new()));
                        if snaps.len() <= 1 {
                            let _ = host.models_mut().update(&runtime_for_timer, |st| {
                                st.autoplay_token = None;
                            });
                            return true;
                        }

                        let index: usize = host
                            .models_mut()
                            .read(&index_for_timer, |v| *v)
                            .ok()
                            .unwrap_or(0);
                        let target_index = if index + 1 < snaps.len() {
                            index + 1
                        } else if loop_for_timer {
                            0
                        } else {
                            let _ = host.models_mut().update(&runtime_for_timer, |st| {
                                st.autoplay_token = None;
                            });
                            return true;
                        };

                        let target = snaps[target_index];
                        let cur = host
                            .models_mut()
                            .read(&offset_for_timer, |v| *v)
                            .ok()
                            .unwrap_or(Px(0.0));
                        let _ = host
                            .models_mut()
                            .update(&index_for_timer, |v| *v = target_index);
                        let _ = host.models_mut().update(&runtime_for_timer, |st| {
                            st.drag = headless_carousel::CarouselDragState::default();
                            st.settling = true;
                            st.settle_from = cur;
                            st.settle_to = target;
                            st.settle_generation = st.settle_generation.saturating_add(1);
                        });
                        host.request_redraw(action_cx.window);

                        host.push_effect(Effect::SetTimer {
                            window: Some(action_cx.window),
                            token,
                            after: cfg.delay,
                            repeat: None,
                        });
                        true
                    }),
                );
            }

            cx.key_add_on_key_down_capture_for(root.id, on_key_down);

            root.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(root_test_id),
            )
        })
    }
}
