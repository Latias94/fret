//! Immediate-mode slider helpers.

use std::sync::Arc;

use fret_core::{Corners, KeyCode, MouseButton, Px, SemanticsOrientation, SemanticsRole};
use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{ContainerProps, Length, MainAlign, PressableA11y, PressableProps};

use super::label_identity::parse_label_identity;
use super::{ResponseExt, SliderOptions, UiWriterImUiFacadeExt};
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

pub(super) fn slider_f32_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<f32>,
    options: SliderOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("slider-label", identity), |ui| {
        slider_f32_model_with_options_inner(ui, visible_label, model, options)
    })
}

fn slider_f32_model_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<f32>,
    options: SliderOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();

    let min = options.min;
    let max = options.max;
    let step = options.step;

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::imui_is_disabled(cx);
        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled && options.focusable;
        props.layout.size.width = Length::Fill;
        props.layout.size.min_height = Some(Length::Px(super::control_chrome::FIELD_MIN_HEIGHT));

        props.a11y = PressableA11y {
            role: Some(SemanticsRole::Slider),
            label: options.a11y_label.clone().or_else(|| Some(label.clone())),
            test_id: options.test_id.clone(),
            ..Default::default()
        };

        let a11y_current = cx
            .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| {
                super::slider_clamp_and_snap(*v, min, max, step)
            })
            .unwrap_or_else(|_| super::slider_clamp_and_snap(min, min, max, step));
        let (a11y_min, a11y_max) = super::slider_normalize_range(min, max);
        let a11y_step = super::slider_step_or_default(step);

        let mut a11y = fret_ui::element::SemanticsDecoration::default()
            .role(SemanticsRole::Slider)
            .orientation(SemanticsOrientation::Horizontal)
            .value(crate::headless::slider::format_semantics_value(
                a11y_current,
            ));

        if a11y_current.is_finite() {
            a11y = a11y.numeric_value(a11y_current as f64);
        }
        if a11y_min.is_finite() && a11y_max.is_finite() {
            a11y = a11y.numeric_range(a11y_min as f64, a11y_max as f64);
        }
        if a11y_step.is_finite() && a11y_step > 0.0 {
            a11y = a11y
                .numeric_step(a11y_step as f64)
                .numeric_jump((a11y_step * 10.0) as f64);
        }

        let label_for_visuals = label.clone();
        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            cx.pressable_clear_on_pointer_down();
            cx.pressable_clear_on_pointer_move();
            cx.pressable_clear_on_pointer_up();
            cx.key_clear_on_key_down_for(id);

            let active_item_model = super::active_item_model_for_window(cx);
            let active_item_model_for_down = active_item_model.clone();
            let active_item_model_for_move = active_item_model.clone();
            let active_item_model_for_up = active_item_model.clone();
            let lifecycle_model = super::lifecycle_session_model_for(cx, id);
            let lifecycle_model_for_down = lifecycle_model.clone();
            let lifecycle_model_for_move = lifecycle_model.clone();
            let lifecycle_model_for_up = lifecycle_model.clone();
            let lifecycle_model_for_key = lifecycle_model.clone();

            let model_for_down = model.clone();
            cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
                if down.button != MouseButton::Left {
                    return PressablePointerDownResult::Continue;
                }

                super::mark_lifecycle_activated_on_left_pointer_down(
                    host,
                    acx,
                    down.button,
                    &lifecycle_model_for_down,
                );
                let _ = host.update_model(&active_item_model_for_down, |st| {
                    st.active = Some(acx.target);
                });
                host.capture_pointer();
                host.request_focus(acx.target);

                let next =
                    super::slider_value_from_pointer(host.bounds(), down.position, min, max, step);
                let mut changed = false;
                let _ = host.update_model(&model_for_down, |value: &mut f32| {
                    let current = super::slider_clamp_and_snap(*value, min, max, step);
                    if (current - next).abs() > f32::EPSILON {
                        *value = next;
                        changed = true;
                    }
                });
                if changed {
                    super::mark_lifecycle_edit(host, acx, &lifecycle_model_for_down);
                    host.record_transient_event(acx, super::KEY_CHANGED);
                    host.notify(acx);
                }

                PressablePointerDownResult::Continue
            }));

            let model_for_move = model.clone();
            cx.pressable_on_pointer_move(Arc::new(move |host, acx, mv| {
                if !mv.buttons.left {
                    super::mark_lifecycle_deactivated_on_left_pointer_up(
                        host,
                        acx,
                        MouseButton::Left,
                        &lifecycle_model_for_move,
                    );
                    host.release_pointer_capture();
                    let _ = host.update_model(&active_item_model_for_move, |st| {
                        if st.active == Some(acx.target) {
                            st.active = None;
                        }
                    });
                    return false;
                }

                let next =
                    super::slider_value_from_pointer(host.bounds(), mv.position, min, max, step);
                let mut changed = false;
                let _ = host.update_model(&model_for_move, |value: &mut f32| {
                    let current = super::slider_clamp_and_snap(*value, min, max, step);
                    if (current - next).abs() > f32::EPSILON {
                        *value = next;
                        changed = true;
                    }
                });
                if changed {
                    super::mark_lifecycle_edit(host, acx, &lifecycle_model_for_move);
                    host.record_transient_event(acx, super::KEY_CHANGED);
                    host.notify(acx);
                }
                changed
            }));

            cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
                if up.button == MouseButton::Left {
                    super::mark_lifecycle_deactivated_on_left_pointer_up(
                        host,
                        acx,
                        up.button,
                        &lifecycle_model_for_up,
                    );
                    host.release_pointer_capture();
                    let _ = host.update_model(&active_item_model_for_up, |st| {
                        if st.active == Some(id) {
                            st.active = None;
                        }
                    });
                }
                PressablePointerUpResult::Continue
            }));

            if enabled {
                let model_for_key = model.clone();
                cx.key_on_key_down_for(
                    id,
                    Arc::new(move |host, acx, down| {
                        let (min, max) = super::slider_normalize_range(min, max);
                        let step = super::slider_step_or_default(step);
                        let delta = match down.key {
                            KeyCode::ArrowLeft | KeyCode::ArrowDown => Some(-step),
                            KeyCode::ArrowRight | KeyCode::ArrowUp => Some(step),
                            KeyCode::PageDown => Some(-step * 10.0),
                            KeyCode::PageUp => Some(step * 10.0),
                            _ => None,
                        };

                        let mut changed = false;
                        let _ = host.update_model(&model_for_key, |value: &mut f32| {
                            let current = super::slider_clamp_and_snap(*value, min, max, step);
                            let next = match down.key {
                                KeyCode::Home => min,
                                KeyCode::End => max,
                                _ => {
                                    let Some(delta) = delta else {
                                        return;
                                    };
                                    super::slider_clamp_and_snap(current + delta, min, max, step)
                                }
                            };
                            if (current - next).abs() > f32::EPSILON {
                                *value = next;
                                changed = true;
                            }
                        });

                        if changed {
                            super::mark_lifecycle_edit(host, acx, &lifecycle_model_for_key);
                            host.record_transient_event(acx, super::KEY_CHANGED);
                            host.notify(acx);
                        }

                        changed
                    }),
                );
            }

            let current = cx
                .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| {
                    super::slider_clamp_and_snap(*v, min, max, step)
                })
                .unwrap_or_else(|_| super::slider_clamp_and_snap(min, min, max, step));
            let progress = slider_progress(current, a11y_min, a11y_max);

            response.core.hovered = state.hovered;
            response.core.pressed = state.pressed;
            response.core.focused = state.focused;
            response.nav_highlighted =
                state.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
            response.id = Some(id);
            response.core.changed = cx.take_transient_for(id, super::KEY_CHANGED);
            response.core.rect = cx.last_bounds_for_element(id);
            let hover_delay =
                super::install_hover_query_hooks_for_pressable(cx, id, state.hovered_raw, None);
            response.pointer_hovered_raw = state.hovered_raw;
            response.pointer_hovered_raw_below_barrier = state.hovered_raw_below_barrier;
            response.hover_stationary_met = hover_delay.stationary_met;
            response.hover_delay_short_met = hover_delay.delay_short_met;
            response.hover_delay_normal_met = hover_delay.delay_normal_met;
            response.hover_delay_short_shared_met = hover_delay.shared_delay_short_met;
            response.hover_delay_normal_shared_met = hover_delay.shared_delay_normal_met;
            response.hover_blocked_by_active_item =
                super::hover_blocked_by_active_item_for(cx, id, &active_item_model);
            super::populate_response_lifecycle_transients(cx, id, response);
            super::populate_response_lifecycle_from_active_state(
                cx,
                id,
                state.pressed,
                response.core.changed,
                response,
            );
            super::sanitize_response_for_enabled(enabled, response);

            let (palette, chrome) = super::control_chrome::field_chrome(cx, enabled, state);
            let mut track = ContainerProps::default();
            track.layout.size.width = Length::Fill;
            track.layout.size.height = Length::Px(super::control_chrome::SLIDER_TRACK_HEIGHT);
            track.background = Some(palette.subtle_background);
            track.corner_radii =
                Corners::all(Px(super::control_chrome::SLIDER_TRACK_HEIGHT.0 / 2.0));

            let mut fill = ContainerProps::default();
            fill.layout.size.width = Length::Fraction(progress);
            fill.layout.size.height = Length::Fill;
            fill.background = Some(palette.accent_background);
            fill.corner_radii = track.corner_radii;

            let value_badge = super::control_chrome::pill(
                cx,
                Arc::from(format!("{current:.2}")),
                palette.accent_background,
                palette.accent_foreground,
            );

            (props, chrome, move |cx| {
                vec![
                    cx.flex(super::control_chrome::fill_stack_props(), move |cx| {
                        let mut out = Vec::new();
                        out.push(cx.flex(
                            super::control_chrome::fill_row_props(MainAlign::SpaceBetween),
                            move |cx| {
                                let mut row = Vec::new();
                                if !label_for_visuals.is_empty() {
                                    row.push(super::control_chrome::caption_text(
                                        cx,
                                        label_for_visuals.clone(),
                                        palette,
                                    ));
                                }
                                row.push(value_badge);
                                row
                            },
                        ));
                        out.push(
                            cx.container(track, move |cx| vec![cx.container(fill, |_cx| vec![])]),
                        );
                        out
                    }),
                ]
            })
        })
        .attach_semantics(a11y)
    });

    ui.add(element);
    response
}

fn slider_progress(current: f32, min: f32, max: f32) -> f32 {
    let range = max - min;
    if !range.is_finite() || range.abs() <= f32::EPSILON {
        return 1.0;
    }

    ((current - min) / range).clamp(0.0, 1.0)
}
