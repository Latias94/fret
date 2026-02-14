use super::super::*;

use std::collections::HashMap;
use std::time::Duration;

use fret_ui::element::LayoutStyle;

fn fmt_bezier(b: fret_ui::theme::CubicBezier) -> String {
    format!("{:.2}, {:.2}, {:.2}, {:.2}", b.x1, b.y1, b.x2, b.y2)
}

pub(super) fn preview_motion_presets(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    motion_preset: Model<Option<Arc<str>>>,
    motion_preset_open: Model<bool>,
    dialog_open: Model<bool>,
) -> Vec<AnyElement> {
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

    #[derive(Default)]
    struct MotionPresetDemoModels {
        stagger_open: Option<Model<bool>>,
        stack_shift_list: Option<Model<Vec<StackShiftListItem>>>,
        stack_shift_next_id: Option<Model<u64>>,
    }

    let stagger_open = cx.with_state(MotionPresetDemoModels::default, |st| {
        st.stagger_open.clone()
    });
    let stagger_open = match stagger_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(MotionPresetDemoModels::default, |st| {
                st.stagger_open = Some(model.clone());
            });
            model
        }
    };

    let stack_shift_list = cx.with_state(MotionPresetDemoModels::default, |st| {
        st.stack_shift_list.clone()
    });
    let stack_shift_list = match stack_shift_list {
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
            cx.with_state(MotionPresetDemoModels::default, |st| {
                st.stack_shift_list = Some(model.clone());
            });
            model
        }
    };

    let stack_shift_next_id = cx.with_state(MotionPresetDemoModels::default, |st| {
        st.stack_shift_next_id.clone()
    });
    let stack_shift_next_id = match stack_shift_next_id {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(5u64);
            cx.with_state(MotionPresetDemoModels::default, |st| {
                st.stack_shift_next_id = Some(model.clone());
            });
            model
        }
    };

    let preset = cx
        .watch_model(&motion_preset)
        .paint()
        .cloned_or_default()
        .unwrap_or_else(|| Arc::from("theme"));

    let shell_layout = LayoutRefinement::default()
        .w_full()
        .max_w(Px(760.0))
        .min_w_0();

    let preset_select = {
        let select = shadcn::Select::new(motion_preset, motion_preset_open)
            .placeholder("Motion preset")
            .trigger_test_id("ui-gallery-motion-preset-trigger")
            .items([
                shadcn::SelectItem::new("theme", "Theme (baseline)")
                    .test_id("ui-gallery-motion-preset-item-theme"),
                shadcn::SelectItem::new("snappy", "Snappy")
                    .test_id("ui-gallery-motion-preset-item-snappy"),
                shadcn::SelectItem::new("bouncy", "Bouncy")
                    .test_id("ui-gallery-motion-preset-item-bouncy"),
                shadcn::SelectItem::new("gentle", "Gentle")
                    .test_id("ui-gallery-motion-preset-item-gentle"),
            ])
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx);

        shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Preset selector").into_element(cx),
                shadcn::CardDescription::new(
                    "Applies a ThemeConfig patch (durations/easings/spring params) on top of the current theme preset.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .gap(Space::N4)
                        .items_center(),
                    move |cx| {
                        vec![
                            select,
                            shadcn::Badge::new(format!("active: {}", preset.as_ref()))
                                .variant(shadcn::BadgeVariant::Secondary)
                                .into_element(cx),
                        ]
                    },
                ),
            ])
            .into_element(cx),
        ])
        .refine_layout(shell_layout.clone())
        .into_element(cx)
        .test_id("ui-gallery-motion-presets-selector-card")
    };

    let token_snapshot = {
        let rows = [
            (
                "duration.shadcn.motion.100",
                theme
                    .duration_ms_token("duration.shadcn.motion.100")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.200",
                theme
                    .duration_ms_token("duration.shadcn.motion.200")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.300",
                theme
                    .duration_ms_token("duration.shadcn.motion.300")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.500",
                theme
                    .duration_ms_token("duration.shadcn.motion.500")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.overlay.open",
                theme
                    .duration_ms_token("duration.shadcn.motion.overlay.open")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.overlay.close",
                theme
                    .duration_ms_token("duration.shadcn.motion.overlay.close")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.collapsible.toggle",
                theme
                    .duration_ms_token("duration.shadcn.motion.collapsible.toggle")
                    .to_string(),
            ),
            (
                "easing.shadcn.motion.overlay",
                fmt_bezier(theme.easing_token("easing.shadcn.motion.overlay")),
            ),
            (
                "easing.shadcn.motion.collapsible.toggle",
                fmt_bezier(theme.easing_token("easing.shadcn.motion.collapsible.toggle")),
            ),
            (
                "duration.shadcn.motion.toast.enter",
                theme
                    .duration_ms_token("duration.shadcn.motion.toast.enter")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.toast.exit",
                theme
                    .duration_ms_token("duration.shadcn.motion.toast.exit")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.toast.stack.shift",
                theme
                    .duration_ms_token("duration.shadcn.motion.toast.stack.shift")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.toast.stack.shift.stagger",
                theme
                    .duration_ms_token("duration.shadcn.motion.toast.stack.shift.stagger")
                    .to_string(),
            ),
            (
                "easing.shadcn.motion.toast.stack.shift",
                fmt_bezier(theme.easing_token("easing.shadcn.motion.toast.stack.shift")),
            ),
            (
                "duration.shadcn.motion.spring.drawer.settle",
                theme
                    .duration_ms_token("duration.shadcn.motion.spring.drawer.settle")
                    .to_string(),
            ),
            (
                "number.shadcn.motion.spring.drawer.settle.bounce",
                format!(
                    "{:.2}",
                    theme.number_token("number.shadcn.motion.spring.drawer.settle.bounce")
                ),
            ),
            (
                "duration.motion.presence.enter",
                theme
                    .duration_ms_token("duration.motion.presence.enter")
                    .to_string(),
            ),
            (
                "duration.motion.collapsible.toggle",
                theme
                    .duration_ms_token("duration.motion.collapsible.toggle")
                    .to_string(),
            ),
            (
                "easing.motion.collapsible.toggle",
                fmt_bezier(theme.easing_token("easing.motion.collapsible.toggle")),
            ),
            (
                "duration.motion.layout.expand",
                theme
                    .duration_ms_token("duration.motion.layout.expand")
                    .to_string(),
            ),
            (
                "duration.motion.stack.shift",
                theme
                    .duration_ms_token("duration.motion.stack.shift")
                    .to_string(),
            ),
            (
                "duration.motion.stack.shift.stagger",
                theme
                    .duration_ms_token("duration.motion.stack.shift.stagger")
                    .to_string(),
            ),
            (
                "easing.motion.stack.shift",
                fmt_bezier(theme.easing_token("easing.motion.stack.shift")),
            ),
            (
                "easing.motion.standard",
                fmt_bezier(theme.easing_token("easing.motion.standard")),
            ),
        ];

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N3)
                .items_start(),
            move |cx| {
                rows.into_iter()
                    .map(|(key, value)| {
                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full())
                                .justify_between()
                                .items_center()
                                .gap(Space::N4),
                            move |cx| {
                                vec![
                                    cx.text(key),
                                    shadcn::Badge::new(value)
                                        .variant(shadcn::BadgeVariant::Outline)
                                        .into_element(cx),
                                ]
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            },
        );

        shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Token snapshot").into_element(cx),
                shadcn::CardDescription::new(
                    "Current effective values for a small, shared set of motion tokens.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([content]).into_element(cx),
        ])
        .refine_layout(shell_layout.clone())
        .into_element(cx)
        .test_id("ui-gallery-motion-presets-token-snapshot")
    };

    let overlay_demo = {
        let open_for_trigger = dialog_open.clone();
        let open_for_close = dialog_open.clone();

        let dialog = shadcn::Dialog::new(dialog_open)
            .into_element(
                cx,
                move |cx| {
                    shadcn::Button::new("Open dialog (presence)")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(open_for_trigger.clone())
                        .test_id("ui-gallery-motion-presets-dialog-trigger")
                        .into_element(cx)
                },
                move |cx| {
                    shadcn::DialogContent::new([
                        shadcn::DialogHeader::new([
                            shadcn::DialogTitle::new("Motion preset demo").into_element(cx),
                            shadcn::DialogDescription::new(
                                "Switch motion presets to compare presence timing + easing under fixed frame delta gates.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::DialogFooter::new([shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(open_for_close.clone())
                            .test_id("ui-gallery-motion-presets-dialog-close")
                            .into_element(cx)])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .test_id("ui-gallery-motion-presets-dialog-content")
                },
            )
            .test_id("ui-gallery-motion-presets-dialog");

        shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Overlay demo").into_element(cx),
                shadcn::CardDescription::new(
                    "Presence motion is token-driven and should feel consistent across refresh rates.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([dialog]).into_element(cx),
        ])
        .refine_layout(shell_layout)
        .into_element(cx)
        .test_id("ui-gallery-motion-presets-overlay-demo")
    };

    let stagger_demo = {
        let is_open = cx
            .watch_model(&stagger_open)
            .paint()
            .copied()
            .unwrap_or(false);

        let duration_ms = theme
            .duration_ms_by_key("duration.motion.stack.shift")
            .unwrap_or_else(|| theme.duration_ms_token("duration.motion.presence.enter"));
        let duration = Duration::from_millis(duration_ms as u64);
        let each_delay_ms = theme
            .duration_ms_by_key("duration.motion.stack.shift.stagger")
            .unwrap_or(24);
        let each_delay = Duration::from_millis(each_delay_ms as u64);
        let easing = theme
            .easing_by_key("easing.motion.stack.shift")
            .unwrap_or_else(|| theme.easing_token("easing.motion.standard"));
        let easing_headless =
            fret_ui_headless::easing::CubicBezier::new(easing.x1, easing.y1, easing.x2, easing.y2);

        let global = fret_ui_kit::primitives::transition::drive_transition_with_durations_and_cubic_bezier_duration_with_mount_behavior(
            cx,
            is_open,
            duration,
            duration,
            easing,
            false,
        );

        // Keep this intentionally small and semantic-first. Component ecosystems should be able
        // to share this "sequence feel" without depending on DOM/Framer Motion implementation
        // details.
        let count = 6usize;
        let from = if is_open {
            fret_ui_headless::motion::stagger::StaggerFrom::First
        } else {
            fret_ui_headless::motion::stagger::StaggerFrom::Last
        };

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2)
                .items_center(),
            move |cx| {
                (0..count)
                    .map(|i| {
                        let local_linear =
                            fret_ui_headless::motion::stagger::staggered_progress_for_duration(
                                global.linear,
                                i,
                                count,
                                each_delay,
                                duration,
                                from,
                            );
                        let local = easing_headless.sample(local_linear);
                        let dy_px = (1.0 - local) * 10.0;
                        let transform = fret_core::Transform2D::translation(fret_core::Point::new(
                            Px(0.0),
                            Px(dy_px),
                        ));

                        cx.opacity_props(
                            fret_ui::element::OpacityProps {
                                layout: LayoutStyle::default(),
                                opacity: local,
                            },
                            move |cx| {
                                let badge = shadcn::Badge::new(format!("Item {}", i + 1))
                                    .variant(shadcn::BadgeVariant::Secondary)
                                    .into_element(cx)
                                    .test_id(format!("ui-gallery-motion-presets-stagger-item-{i}"));

                                vec![cx.visual_transform_props(
                                    fret_ui::element::VisualTransformProps {
                                        layout: LayoutStyle::default(),
                                        transform,
                                    },
                                    |_cx| vec![badge],
                                )]
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            },
        );

        let toggle = shadcn::Button::new("Toggle sequence")
            .variant(shadcn::ButtonVariant::Secondary)
            .toggle_model(stagger_open.clone())
            .test_id("ui-gallery-motion-presets-stagger-toggle")
            .into_element(cx);

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N3)
                .items_start(),
            move |_cx| vec![toggle, row],
        );

        shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Stagger / sequence demo").into_element(cx),
                shadcn::CardDescription::new(
                    "One shared timeline mapped into per-item progress via a small headless stagger helper.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([content]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(760.0)).min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-motion-presets-stagger-demo")
    };

    let stack_shift_list_demo = {
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

        let items = cx
            .watch_model(&stack_shift_list)
            .paint()
            .cloned_or_default();
        let active_count = items.iter().filter(|i| !i.exiting).count();
        let exiting_count = items.iter().filter(|i| i.exiting).count();

        let add = {
            let list = stack_shift_list.clone();
            let next_id = stack_shift_next_id.clone();
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
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
            let on_activate: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
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

        let controls = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N3)
                .items_center(),
            move |cx| {
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
            },
        );

        let list_panel = cx.keyed("stack_shift_list_demo", |cx| {
            let items = cx
                .watch_model(&stack_shift_list)
                .paint()
                .cloned_or_default()
                ;

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
                    let local_linear =
                        fret_ui_headless::motion::stagger::staggered_progress_for_duration(
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

                if shift_active
                    && !shift.animating
                    && (shift.progress - 1.0).abs() <= f32::EPSILON
                {
                    st.active = false;
                    st.deltas_y.clear();
                    st.last_visual_y.clone_from(&targets_y);
                }
            });

            let total_rows = active.len() + exiting.len();
            let total_h = if total_rows == 0 {
                Px(0.0)
            } else {
                Px(
                    row_h.0 * (total_rows as f32)
                        + gap.0 * (total_rows.saturating_sub(1) as f32),
                )
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
                        row_layout.inset.left = Some(Px(0.0));
                        row_layout.inset.right = Some(Px(0.0));
                        row_layout.inset.top = Some(Px(0.0));
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

                                let body = stack::hstack(
                                    cx,
                                    stack::HStackProps::default()
                                        .layout(LayoutRefinement::default().w_full())
                                        .justify_between()
                                        .items_center(),
                                    move |cx| {
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
                                    },
                                );

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
                let list = stack_shift_list.clone();
                let ids = to_prune.clone();
                let _ = cx.app.models_mut().update(&list, |items| {
                    items.retain(|i| !ids.contains(&i.id));
                });
                cx.app.request_redraw(cx.window);
            }

            cx.container(container_props, move |_cx| [stage])
                .test_id("ui-gallery-motion-presets-stack-shift-stage")
        });

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N3)
                .items_start(),
            move |_cx| vec![controls, list_panel],
        );

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
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(760.0)).min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-motion-presets-stack-shift-list-demo")
    };

    vec![
        preset_select,
        token_snapshot,
        overlay_demo,
        stagger_demo,
        stack_shift_list_demo,
    ]
}
