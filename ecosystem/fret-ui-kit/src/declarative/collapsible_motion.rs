//! Collapsible-style height transitions.
//!
//! Upstream Radix Collapsible/Accordion coordinate mount/unmount via `Presence` and expose measured
//! content dimensions for CSS keyframe animations (e.g. `--radix-collapsible-content-height`).
//!
//! Fret does not use CSS variables. Instead, we cache the last known open height and drive a
//! clipped wrapper height using a `TransitionTimeline` progress value.

use fret_core::{Px, Size};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::headless::transition::TransitionOutput;
use crate::{LayoutRefinement, Space};

/// Output describing how to render a collapsible-style measured-height wrapper for the current
/// element root.
///
/// This helper is usable by any "open/close with height animation" component (Collapsible,
/// Accordion items, etc.). It does not build elements; it only describes the wrapper/refinement and
/// how to update cached measurements once an element id is known.
#[derive(Debug, Clone)]
pub struct MeasuredHeightMotionOutput {
    pub state_id: GlobalElementId,
    /// The requested open state (source of truth).
    pub open: bool,
    /// Whether the transition should be driven as open this frame.
    ///
    /// When there is no cached measurement, this is `false` even if `open=true` so that the first
    /// open can mount an off-flow measurement wrapper before animating.
    pub open_for_motion: bool,
    /// Whether an off-flow measurement pass is required this frame.
    pub wants_measurement: bool,
    /// Transition timeline output for `open_for_motion`.
    pub transition: TransitionOutput,
    /// Whether the content wrapper should be present in the element tree.
    pub should_render: bool,
    /// Layout refinement for the wrapper (either a clipped-height wrapper or a measurement wrapper).
    pub wrapper_refinement: LayoutRefinement,
    /// Opacity to apply to the wrapper subtree (0.0 for measurement; 1.0 for visible content).
    pub wrapper_opacity: f32,
}

#[derive(Debug, Clone, Copy)]
struct MeasuredSizeState {
    last: Size,
}

impl Default for MeasuredSizeState {
    fn default() -> Self {
        Self {
            last: Size::new(Px(0.0), Px(0.0)),
        }
    }
}

/// Read the last cached open height for a collapsible content subtree.
pub fn last_measured_height_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state_id: GlobalElementId,
) -> Px {
    cx.with_state_for(state_id, MeasuredSizeState::default, |st| st.last.height)
}

/// Read the last cached open size for a collapsible content subtree.
pub fn last_measured_size_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state_id: GlobalElementId,
) -> Size {
    cx.with_state_for(state_id, MeasuredSizeState::default, |st| st.last)
}

/// Update the cached open height from the previously-laid-out bounds of `wrapper_element_id`.
///
/// This should be called from the same element scope that renders the wrapper (so the wrapper ID
/// is stable), but the cached value can be stored on a separate `state_id` (typically the root).
pub fn update_measured_height_if_open_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state_id: GlobalElementId,
    wrapper_element_id: GlobalElementId,
    open: bool,
    animating: bool,
) -> Px {
    let last = last_measured_size_for(cx, state_id);
    let last_height = last.height;

    if !open || animating {
        return last_height;
    }

    let Some(bounds) = cx.last_bounds_for_element(wrapper_element_id) else {
        return last_height;
    };

    let h = bounds.size.height;
    if h.0 <= 0.0 || (h.0 - last_height.0).abs() <= 0.5 {
        return last_height;
    }

    cx.with_state_for(state_id, MeasuredSizeState::default, |st| {
        st.last = bounds.size;
    });
    h
}

/// Update the cached open size from a "measurement element" that is laid out off-flow.
///
/// This is intended to support the first open animation: callers can render a hidden, absolutely
/// positioned copy of the content that does not affect layout, then read its last-frame bounds.
pub fn update_measured_size_from_element_if_open_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    state_id: GlobalElementId,
    measure_element_id: GlobalElementId,
    open: bool,
) -> Size {
    let last = last_measured_size_for(cx, state_id);
    if !open {
        return last;
    }

    let Some(bounds) = cx.last_bounds_for_element(measure_element_id) else {
        return last;
    };

    if bounds.size.height.0 <= 0.0 {
        return last;
    }

    // Avoid churn from layout rounding noise.
    let dw = (bounds.size.width.0 - last.width.0).abs();
    let dh = (bounds.size.height.0 - last.height.0).abs();
    if dw <= 0.5 && dh <= 0.5 {
        return last;
    }

    cx.with_state_for(state_id, MeasuredSizeState::default, |st| {
        st.last = bounds.size;
    });

    bounds.size
}

/// Layout refinement for an off-flow measurement wrapper.
///
/// Call sites should typically wrap this in an opacity gate so it does not paint.
pub fn collapsible_measurement_wrapper_refinement() -> LayoutRefinement {
    LayoutRefinement::default()
        .absolute()
        .top(Space::N0)
        .left(Space::N0)
        .right(Space::N0)
        .overflow_visible()
}

/// Compute wrapper mounting and layout patches for a collapsible content subtree.
///
/// When a measurement exists, the wrapper height is driven using `transition.progress` as an eased
/// 0..1 progress value. Without a measurement, call sites should avoid "close presence" to prevent
/// hidden content from affecting layout.
pub fn collapsible_height_wrapper_refinement(
    open: bool,
    force_mount: bool,
    require_measurement_for_close: bool,
    transition: TransitionOutput,
    measured_height: Px,
) -> (bool, LayoutRefinement) {
    let has_measurement = measured_height.0 > 0.0;
    let progress = transition.progress.clamp(0.0, 1.0);

    let keep_mounted_for_close =
        transition.present && (!require_measurement_for_close || has_measurement);
    let should_render = force_mount || open || keep_mounted_for_close;

    let wants_height_animation = has_measurement && (transition.animating || !open);

    let mut wrapper = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .min_h(Px(0.0))
        .overflow_hidden();
    if wants_height_animation {
        wrapper = wrapper.h_px(Px(measured_height.0 * progress));
    } else if !open && force_mount {
        wrapper = wrapper.h_px(Px(0.0));
    }

    (should_render, wrapper)
}

/// Computes a measured-height motion plan for the current element root.
///
/// Call sites should:
/// 1. Call this during rendering to obtain the wrapper refinement.
/// 2. Render a wrapper element with a stable id (e.g. using `cx.keyed(...)`).
/// 3. Call `update_measured_for_motion(...)` with the wrapper element id to update cached size.
#[allow(clippy::too_many_arguments)]
pub fn measured_height_motion_for_root<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    force_mount: bool,
    require_measurement_for_close: bool,
    open_ticks: u64,
    close_ticks: u64,
    ease: fn(f32) -> f32,
) -> MeasuredHeightMotionOutput {
    let state_id = cx.root_id();
    let last_height = last_measured_height_for(cx, state_id);
    let has_measurement = last_height.0 > 0.0;
    let wants_measurement = open && !has_measurement;
    let open_for_motion = open && has_measurement;

    let transition = crate::declarative::transition::drive_transition_with_durations_and_easing(
        cx,
        open_for_motion,
        open_ticks,
        close_ticks,
        ease,
    );

    if wants_measurement {
        cx.request_frame();
        return MeasuredHeightMotionOutput {
            state_id,
            open,
            open_for_motion,
            wants_measurement,
            transition,
            should_render: true,
            wrapper_refinement: collapsible_measurement_wrapper_refinement(),
            wrapper_opacity: 0.0,
        };
    }

    let (should_render, wrapper_refinement) = collapsible_height_wrapper_refinement(
        open_for_motion,
        force_mount,
        require_measurement_for_close,
        transition,
        last_height,
    );

    MeasuredHeightMotionOutput {
        state_id,
        open,
        open_for_motion,
        wants_measurement,
        transition,
        should_render,
        wrapper_refinement,
        wrapper_opacity: 1.0,
    }
}

/// Updates the cached measured size/height based on the wrapper element id.
///
/// When `motion.wants_measurement=true`, the wrapper is expected to be an off-flow measurement
/// wrapper.
pub fn update_measured_for_motion<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    motion: MeasuredHeightMotionOutput,
    wrapper_element_id: GlobalElementId,
) -> Size {
    if motion.wants_measurement {
        return update_measured_size_from_element_if_open_for(
            cx,
            motion.state_id,
            wrapper_element_id,
            motion.open,
        );
    }

    let _ = update_measured_height_if_open_for(
        cx,
        motion.state_id,
        wrapper_element_id,
        motion.open,
        motion.transition.animating,
    );
    last_measured_size_for(cx, motion.state_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, SvgId, SvgService, TextBlobId, TextConstraints, TextInput,
        TextMetrics, TextService,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Point, Px, Rect};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::element::{ContainerProps, LayoutStyle, Length, OpacityProps};
    use fret_ui::elements::GlobalElementId;
    use fret_ui::{Theme, UiTree};

    use crate::declarative::model_watch::ModelWatchExt as _;
    use crate::declarative::style as decl_style;
    use crate::declarative::transition;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    #[test]
    fn collapsible_can_measure_off_flow_then_animate_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );
        let mut services = FakeServices::default();

        let wrapper_id_out: std::cell::Cell<Option<GlobalElementId>> = std::cell::Cell::new(None);

        let bump_frame = |app: &mut App| {
            app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
            app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        };

        let render = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            bump_frame(app);
            let wrapper_id_out = &wrapper_id_out;

            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "collapsible-motion",
                |cx| {
                    let state_id = cx.root_id();
                    let is_open = cx.watch_model(&open).copied_or_default();

                    let measured = last_measured_height_for(cx, state_id);
                    let has_measurement = measured.0 > 0.0;
                    let wants_measure = is_open && !has_measurement;

                    let mut probe_layout = LayoutStyle::default();
                    probe_layout.size.width = Length::Fill;
                    probe_layout.size.height = Length::Px(Px(1.0));

                    let mut content_layout = LayoutStyle::default();
                    content_layout.size.width = Length::Fill;
                    content_layout.size.height = Length::Px(Px(80.0));

                    // Delay the opening transition until we have a non-zero measurement, so the
                    // animation starts from 0 -> measured height (Radix-like).
                    let open_for_motion = is_open && has_measurement;
                    let motion = transition::drive_transition_with_durations_and_easing(
                        cx,
                        open_for_motion,
                        8,
                        8,
                        |t| t,
                    );

                    let (should_render_wrapper, wrapper) = collapsible_height_wrapper_refinement(
                        open_for_motion,
                        false,
                        true,
                        motion,
                        measured,
                    );

                    let mut children = Vec::new();

                    // Ensure the root has a non-zero width so absolutely positioned measurement
                    // wrappers can resolve `left/right` insets.
                    children.push(cx.container(
                        ContainerProps {
                            layout: probe_layout,
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    ));

                    if wants_measure {
                        let theme = Theme::global(&*cx.app);
                        let measure_layout = decl_style::layout_style(
                            theme,
                            collapsible_measurement_wrapper_refinement(),
                        );
                        let measurer = cx.keyed("collapsible-measure", |cx| {
                            cx.container(
                                ContainerProps {
                                    layout: measure_layout,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![cx.opacity_props(
                                        OpacityProps {
                                            layout: LayoutStyle::default(),
                                            opacity: 0.0,
                                        },
                                        |cx| {
                                            vec![cx.container(
                                                ContainerProps {
                                                    layout: content_layout,
                                                    ..Default::default()
                                                },
                                                |_cx| Vec::new(),
                                            )]
                                        },
                                    )]
                                },
                            )
                        });
                        let measurer_id = measurer.id;
                        children.push(measurer);

                        // Update from last-frame bounds of the measurement element.
                        let _ = update_measured_size_from_element_if_open_for(
                            cx,
                            state_id,
                            measurer_id,
                            is_open,
                        );
                    }

                    if should_render_wrapper {
                        let theme = Theme::global(&*cx.app);
                        let wrapper_layout = decl_style::layout_style(theme, wrapper);
                        let wrapper_el = cx.keyed("collapsible-wrapper", |cx| {
                            cx.container(
                                ContainerProps {
                                    layout: wrapper_layout,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![cx.container(
                                        ContainerProps {
                                            layout: content_layout,
                                            ..Default::default()
                                        },
                                        |_cx| Vec::new(),
                                    )]
                                },
                            )
                        });
                        wrapper_id_out.set(Some(wrapper_el.id));
                        children.push(wrapper_el);
                    }

                    children
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
        };

        // Closed frame.
        render(&mut ui, &mut app, &mut services);

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Measurement + early animation frames.
        let mut wrapper_id = None;
        let mut saw_partial_height = false;
        for _ in 0..8 {
            render(&mut ui, &mut app, &mut services);
            wrapper_id = wrapper_id.or_else(|| wrapper_id_out.get());
            let Some(wrapper_id) = wrapper_id else {
                continue;
            };
            let Some(wrapper_bounds) =
                fret_ui::elements::bounds_for_element(&mut app, window, wrapper_id)
            else {
                continue;
            };

            if wrapper_bounds.size.height.0 > 0.0 && wrapper_bounds.size.height.0 < 80.0 {
                saw_partial_height = true;
                break;
            }
        }
        assert!(
            saw_partial_height,
            "expected an intermediate animated height"
        );
        let wrapper_id = wrapper_id.expect("wrapper id");

        // Advance frames until the wrapper reaches its final height.
        //
        // Note: `bounds_for_element` intentionally returns the *previous* frame's bounds. If we
        // keep producing frames after the animation settles (without any invalidations), the
        // runtime may stop recording bounds and this query can return `None`. Real apps typically
        // stop producing frames once the transition settles, so this test stops as soon as it
        // observes the final height.
        let mut settled = false;
        for _ in 0..16 {
            render(&mut ui, &mut app, &mut services);
            let Some(wrapper_bounds) =
                fret_ui::elements::bounds_for_element(&mut app, window, wrapper_id)
            else {
                continue;
            };
            if (wrapper_bounds.size.height.0 - 80.0).abs() <= 0.5 {
                settled = true;
                break;
            }
        }
        assert!(settled, "expected wrapper to reach its final height");
    }
}
