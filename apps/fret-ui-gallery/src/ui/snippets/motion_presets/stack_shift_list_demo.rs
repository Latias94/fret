pub const SOURCE: &str = include_str!("stack_shift_list_demo.rs");

// region: example
use fret::UiCx;
use fret_ui::Theme;
use fret_ui::element::LayoutStyle;
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Default)]
struct StackShiftListShiftState {
    generation: u64,
    active: bool,
    last_targets_y: HashMap<u64, Px>,
    last_visual_y: HashMap<u64, Px>,
    deltas_y: HashMap<u64, Px>,
}

#[derive(Debug, Clone)]
struct StackShiftListItem {
    id: u64,
    exiting: bool,
    exit_slot: usize,
}

#[derive(Default, Clone)]
struct Models {
    stack_shift_list: Option<Model<Vec<StackShiftListItem>>>,
    stack_shift_next_id: Option<Model<u64>>,
}

fn ensure_models(cx: &mut UiCx<'_>) -> (Model<Vec<StackShiftListItem>>, Model<u64>) {
    let state = cx.with_state(Models::default, |st| st.clone());

    let list = match state.stack_shift_list {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![
                StackShiftListItem {
                    id: 1,
                    exiting: false,
                    exit_slot: 0,
                },
                StackShiftListItem {
                    id: 2,
                    exiting: false,
                    exit_slot: 0,
                },
                StackShiftListItem {
                    id: 3,
                    exiting: false,
                    exit_slot: 0,
                },
                StackShiftListItem {
                    id: 4,
                    exiting: false,
                    exit_slot: 0,
                },
            ]);
            cx.with_state(Models::default, |st| {
                st.stack_shift_list = Some(model.clone())
            });
            model
        }
    };

    let next_id = match state.stack_shift_next_id {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(5u64);
            cx.with_state(Models::default, |st| {
                st.stack_shift_next_id = Some(model.clone())
            });
            model
        }
    };

    (list, next_id)
}

pub fn render(cx: &mut UiCx<'_>, theme: &Theme) -> AnyElement {
    let shell_layout = LayoutRefinement::default()
        .w_full()
        .max_w(Px(760.0))
        .min_w_0();

    let (stack_shift_list, stack_shift_next_id) = ensure_models(cx);

    let shift_duration_ms = theme.duration_ms_token("duration.motion.stack.shift");
    let shift_duration = Duration::from_millis(shift_duration_ms as u64);
    let shift_each_delay_ms = theme.duration_ms_token("duration.motion.stack.shift.stagger");
    let shift_each_delay = Duration::from_millis(shift_each_delay_ms as u64);
    let shift_easing = theme.easing_token("easing.motion.stack.shift");
    let shift_easing_headless = fret_ui_headless::easing::CubicBezier::new(
        shift_easing.x1,
        shift_easing.y1,
        shift_easing.x2,
        shift_easing.y2,
    );

    let items = cx.watch_model(&stack_shift_list).paint().value_or_default();
    let active_count = items.iter().filter(|i| !i.exiting).count();
    let exiting_count = items.iter().filter(|i| i.exiting).count();

    let add = {
        let list = stack_shift_list.clone();
        let next_id = stack_shift_next_id.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let id = host
                .models_mut()
                .update(&next_id, |v| {
                    let id = *v;
                    *v = id.saturating_add(1);
                    id
                })
                .unwrap_or(0);

            let _ = host.models_mut().update(&list, |items| {
                items.insert(
                    0,
                    StackShiftListItem {
                        id,
                        exiting: false,
                        exit_slot: 0,
                    },
                );
            });
            host.request_redraw(action_cx.window);
            host.push_effect(fret_runtime::Effect::RequestAnimationFrame(
                action_cx.window,
            ));
        });

        shadcn::Button::new("Insert at top")
            .variant(shadcn::ButtonVariant::Secondary)
            .on_activate(on_activate)
            .test_id("ui-gallery-motion-presets-stack-shift-insert")
            .into_element(cx)
    };

    let remove = {
        let list = stack_shift_list.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&list, |items| {
                let Some(pos) = items.iter().position(|i| !i.exiting) else {
                    return;
                };
                let mut item = items.remove(pos);
                item.exiting = true;
                item.exit_slot = 0;
                items.push(item);
            });
            host.request_redraw(action_cx.window);
            host.push_effect(fret_runtime::Effect::RequestAnimationFrame(
                action_cx.window,
            ));
        });

        shadcn::Button::new("Remove from top")
            .variant(shadcn::ButtonVariant::Outline)
            .on_activate(on_activate)
            .test_id("ui-gallery-motion-presets-stack-shift-remove")
            .into_element(cx)
    };

    let controls = ui::h_flex(move |cx| {
        vec![
            add,
            remove,
            shadcn::Badge::new(format!("active: {active_count}"))
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
            shadcn::Badge::new(format!("exiting: {exiting_count}"))
                .variant(shadcn::BadgeVariant::Outline)
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N3)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-motion-presets-stack-shift-controls");

    let list_panel = cx.keyed("stack_shift_list_demo", |cx| {
        let items = cx.watch_model(&stack_shift_list).paint().value_or_default();

        let row_h = Px(34.0);
        let gap = Px(8.0);

        let active: Vec<&StackShiftListItem> = items.iter().filter(|i| !i.exiting).collect();
        let exiting: Vec<&StackShiftListItem> = items.iter().filter(|i| i.exiting).collect();

        let mut targets_y: HashMap<u64, Px> = HashMap::new();
        for (idx, item) in active.iter().enumerate() {
            let y = (row_h.0 + gap.0) * (idx as f32);
            targets_y.insert(item.id, Px(y));
        }
        for item in &exiting {
            let y = (row_h.0 + gap.0) * (item.exit_slot as f32);
            targets_y.insert(item.id, Px(y));
        }

        let (shift_active, generation, deltas_y) =
            cx.with_state(StackShiftListShiftState::default, |st| {
                let mut changed = st.last_targets_y.len() != targets_y.len();
                if !changed {
                    for (id, curr) in &targets_y {
                        if let Some(prev) = st.last_targets_y.get(id)
                            && (prev.0 - curr.0).abs() > 0.5
                        {
                            changed = true;
                            break;
                        }
                    }
                }

                if changed {
                    st.active = true;
                    st.generation = st.generation.wrapping_add(1);
                    st.deltas_y.clear();

                    for (id, target) in &targets_y {
                        let from = st
                            .last_visual_y
                            .get(id)
                            .copied()
                            .or_else(|| st.last_targets_y.get(id).copied())
                            .unwrap_or(*target);
                        st.deltas_y.insert(*id, Px(from.0 - target.0));
                    }
                }

                st.last_targets_y.clone_from(&targets_y);
                (st.active, st.generation, st.deltas_y.clone())
            });

        let shift = cx.keyed(("stack_shift_list_demo_shift", generation), |cx| {
            fret_ui_kit::primitives::transition::drive_transition_with_durations_and_cubic_bezier_duration_with_mount_behavior(
                cx,
                shift_active,
                shift_duration,
                shift_duration,
                shift_easing,
                true,
            )
        });

        let mut out_y: HashMap<u64, Px> = HashMap::new();
        if shift_active {
            let active_count = active.len().max(1);
            for (idx, item) in active.iter().enumerate() {
                let id = item.id;
                let target = targets_y.get(&id).copied().unwrap_or(Px(0.0));
                let delta = deltas_y.get(&id).copied().unwrap_or(Px(0.0));
                let local_linear = fret_ui_headless::motion::stagger::staggered_progress_for_duration(
                    shift.linear,
                    idx,
                    active_count,
                    shift_each_delay,
                    shift_duration,
                    fret_ui_headless::motion::stagger::StaggerFrom::First,
                );
                let local = shift_easing_headless.sample(local_linear);
                out_y.insert(id, Px(target.0 + delta.0 * (1.0 - local)));
            }

            for item in &exiting {
                let id = item.id;
                let target = targets_y.get(&id).copied().unwrap_or(Px(0.0));
                let delta = deltas_y.get(&id).copied().unwrap_or(Px(0.0));
                let local = shift_easing_headless.sample(shift.linear);
                out_y.insert(id, Px(target.0 + delta.0 * (1.0 - local)));
            }
        } else {
            out_y.clone_from(&targets_y);
        }

        cx.with_state(StackShiftListShiftState::default, |st| {
            st.last_visual_y.clone_from(&out_y);

            if shift_active && !shift.animating && (shift.progress - 1.0).abs() <= f32::EPSILON {
                st.active = false;
                st.deltas_y.clear();
                st.last_visual_y.clone_from(&targets_y);
            }
        });

        let total_rows = active.len() + exiting.len();
        let total_h = if total_rows == 0 {
            Px(0.0)
        } else {
            Px(row_h.0 * (total_rows as f32) + gap.0 * (total_rows.saturating_sub(1) as f32))
        };

        let container_props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .p(Space::N3),
            LayoutRefinement::default().w_full(),
        );

        let mut stage_layout = LayoutStyle::default();
        stage_layout.size.width = fret_ui::element::Length::Fill;
        stage_layout.size.height = fret_ui::element::Length::Px(total_h);

        let mut to_prune: Vec<u64> = Vec::new();
        let stage = cx.stack_props(
            fret_ui::element::StackProps { layout: stage_layout },
            |cx| {
                let mut out: Vec<AnyElement> = Vec::new();

                for item in items.clone() {
                    let id = item.id;
                    let y = out_y.get(&id).copied().unwrap_or(Px(0.0));
                    let open = !item.exiting;

                    let presence = cx.keyed((id, "presence"), |cx| {
                        fret_ui_kit::primitives::presence::fade_presence_with_durations_and_cubic_bezier_duration(
                            cx,
                            open,
                            shift_duration,
                            shift_duration,
                            shift_easing,
                        )
                    });

                    if item.exiting && !presence.present {
                        to_prune.push(id);
                        continue;
                    }

                    let mut row_layout = LayoutStyle::default();
                    row_layout.position = fret_ui::element::PositionStyle::Absolute;
                    row_layout.inset.left = Some(Px(0.0)).into();
                    row_layout.inset.right = Some(Px(0.0)).into();
                    row_layout.inset.top = Some(Px(0.0)).into();
                    row_layout.size.height = fret_ui::element::Length::Px(row_h);

                    let transform = fret_core::Transform2D::translation(fret_core::Point::new(
                        Px(0.0),
                        y,
                    ));

                    let row = cx.opacity_props(
                        fret_ui::element::OpacityProps {
                            layout: LayoutStyle::default(),
                            opacity: presence.opacity,
                        },
                        move |cx| {
                            let props = decl_style::container_props(
                                theme,
                                ChromeRefinement::default()
                                    .border_1()
                                    .rounded(Radius::Sm)
                                    .bg(ColorRef::Token {
                                        key: "card",
                                        fallback: fret_ui_kit::ColorFallback::ThemePanelBackground,
                                    })
                                    .p(Space::N2),
                                LayoutRefinement::default()
                                    .w_full()
                                    .h_px(row_h)
                                    .min_w_0(),
                            );

                            let body = ui::h_flex(move |cx| {
                                    let left = cx.text(format!("Item {id}"));
                                    let right = if item.exiting {
                                        shadcn::Badge::new("Exiting")
                                            .variant(shadcn::BadgeVariant::Outline)
                                            .into_element(cx)
                                    } else {
                                        shadcn::Badge::new("Active")
                                            .variant(shadcn::BadgeVariant::Secondary)
                                            .into_element(cx)
                                    };
                                    vec![left, right]
                                })
                                    .layout(LayoutRefinement::default().w_full())
                                    .justify_between()
                                    .items_center().into_element(cx);

                            vec![cx.container(props, move |_cx| [body])]
                        },
                    );

                    out.push(cx.visual_transform_props(
                        fret_ui::element::VisualTransformProps {
                            layout: row_layout,
                            transform,
                        },
                        |_cx| vec![row],
                    ));
                }

                out
            },
        );

        if !to_prune.is_empty() {
            let ids = to_prune.clone();
            let _ = cx.app.models_mut().update(&stack_shift_list, |items| {
                items.retain(|i| !ids.contains(&i.id));
            });
            cx.app.request_redraw(cx.window);
        }

        cx.container(container_props, move |_cx| [stage])
            .test_id("ui-gallery-motion-presets-stack-shift-stage")
    });

    let content = ui::v_flex(move |_cx| vec![controls, list_panel])
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N3)
        .items_start()
        .into_element(cx);

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Stack shift list demo").into_element(cx),
            shadcn::CardDescription::new(
                "A list insert/remove choreography driven by semantic `stack.shift` tokens (duration + stagger + easing).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([content]).into_element(cx),
    ])
    .refine_layout(shell_layout)
    .into_element(cx)
    .test_id("ui-gallery-motion-presets-stack-shift-list-demo")
}
// endregion: example
