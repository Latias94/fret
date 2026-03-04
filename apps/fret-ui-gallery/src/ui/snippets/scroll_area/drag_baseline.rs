pub const SOURCE: &str = include_str!("drag_baseline.rs");

// region: example
use fret_core::{Point, Px, SemanticsRole, TimerToken};
use fret_runtime::Effect;
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::SemanticsDecoration;
use fret_ui::element::SemanticsProps;
use fret_ui::scroll::ScrollHandle;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;
use std::time::Duration;

#[derive(Default)]
struct DemoModels {
    arm_grow: Option<Model<bool>>,
    grew: Option<Model<bool>>,
    timer_token: Option<Model<Option<TimerToken>>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.named("ui-gallery.scroll_area.drag_baseline", |cx| {
        let scroll_handle = cx.with_state(ScrollHandle::default, |h| h.clone());

        let arm_grow = cx.with_state(DemoModels::default, |st| st.arm_grow.clone());
        let arm_grow = match arm_grow {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(false);
                cx.with_state(DemoModels::default, |st| st.arm_grow = Some(model.clone()));
                model
            }
        };

        let grew = cx.with_state(DemoModels::default, |st| st.grew.clone());
        let grew = match grew {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(false);
                cx.with_state(DemoModels::default, |st| st.grew = Some(model.clone()));
                model
            }
        };

        let timer_token = cx.with_state(DemoModels::default, |st| st.timer_token.clone());
        let timer_token = match timer_token {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(None::<TimerToken>);
                cx.with_state(DemoModels::default, |st| {
                    st.timer_token = Some(model.clone())
                });
                model
            }
        };

        let arm_grow_for_timer = arm_grow.clone();
        let grew_for_timer = grew.clone();
        let timer_token_for_timer = timer_token.clone();

        cx.semantics_with_id(
            SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("ui-gallery-scroll-area-drag-baseline-harness")),
                ..Default::default()
            },
            move |cx, id| {
                cx.timer_on_timer_for(
                    id,
                    Arc::new(move |host, action_cx, token| {
                        let expected = host
                            .models_mut()
                            .read(&timer_token_for_timer, Clone::clone)
                            .ok()
                            .flatten();
                        if expected != Some(token) {
                            return false;
                        }
                        let _ = host
                            .models_mut()
                            .update(&timer_token_for_timer, |v| *v = None);
                        let _ = host
                            .models_mut()
                            .update(&arm_grow_for_timer, |v| *v = false);
                        let _ = host.models_mut().update(&grew_for_timer, |v| *v = true);
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    }),
                );

                let is_armed = cx
                    .get_model_copied(&arm_grow, Invalidation::Paint)
                    .unwrap_or(false);
                let did_grow = cx
                    .get_model_copied(&grew, Invalidation::Paint)
                    .unwrap_or(false);

                let reset = {
                    let arm_grow = arm_grow.clone();
                    let grew = grew.clone();
                    let timer_token = timer_token.clone();
                    let scroll_handle = scroll_handle.clone();
                    shadcn::Button::new("Reset")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .on_activate(Arc::new(move |host, action_cx, _reason| {
                            let _ = host.models_mut().update(&arm_grow, |v| *v = false);
                            let _ = host.models_mut().update(&grew, |v| *v = false);
                            let _ = host.models_mut().update(&timer_token, |v| *v = None);
                            scroll_handle.scroll_to_offset(Point::new(Px(0.0), Px(0.0)));
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                        }))
                        .test_id("ui-gallery-scroll-area-drag-baseline-reset")
                        .into_element(cx)
                };

                let arm = {
                    let arm_grow = arm_grow.clone();
                    let grew = grew.clone();
                    let timer_token = timer_token.clone();
                    shadcn::Button::new("Arm content growth")
                        .on_activate(Arc::new(move |host, action_cx, _reason| {
                            let token = host.next_timer_token();
                            let _ = host.models_mut().update(&arm_grow, |v| *v = true);
                            let _ = host.models_mut().update(&grew, |v| *v = false);
                            let _ = host.models_mut().update(&timer_token, |v| *v = Some(token));
                            host.push_effect(Effect::SetTimer {
                                window: Some(action_cx.window),
                                token,
                                after: Duration::from_millis(120),
                                repeat: None,
                            });
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                        }))
                        .test_id("ui-gallery-scroll-area-drag-baseline-arm-grow")
                        .into_element(cx)
                };

                let status = if did_grow {
                    shadcn::Badge::new("Grown")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx)
                        .test_id("ui-gallery-scroll-area-drag-baseline-grown")
                } else if is_armed {
                    shadcn::Badge::new("Armed")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx)
                        .test_id("ui-gallery-scroll-area-drag-baseline-armed")
                } else {
                    shadcn::Badge::new("Idle")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx)
                        .test_id("ui-gallery-scroll-area-drag-baseline-idle")
                };

                let controls = ui::h_flex(cx, |_cx| [reset, arm, status])
                    .gap(Space::N2)
                    .into_element(cx)
                    .test_id("ui-gallery-scroll-area-drag-baseline-controls");

                let instructions = shadcn::typography::muted(
                    cx,
                    "Drag the thumb, then click “Arm content growth”. Content will grow after ~120ms; the thumb should remain stable.",
                )
                .test_id("ui-gallery-scroll-area-drag-baseline-instructions");

                // Deterministic sizing for diagnostics:
                // - viewport height: 200px
                // - baseline content height: 220px (11 * 20px), so max_offset_y = 20px
                // - when the timer fires, append many rows to simulate "extents grow mid-drag".
                let row_h = Px(20.0);
                let baseline_rows = 11;

                let mut row_layout = fret_ui::element::LayoutStyle::default();
                row_layout.size.width = fret_ui::element::Length::Fill;
                row_layout.size.height = fret_ui::element::Length::Px(row_h);

                let row_props = fret_ui::element::ContainerProps {
                    layout: row_layout,
                    ..Default::default()
                };

                let content = cx.column(
                    fret_ui::element::ColumnProps {
                        gap: Px(0.0).into(),
                        ..Default::default()
                    },
                    move |cx| {
                        let mut rows: Vec<AnyElement> = Vec::new();
                        for _ in 0..baseline_rows {
                            rows.push(cx.container(row_props, |_cx| Vec::new()));
                        }
                        if did_grow {
                            for _ in 0..200 {
                                rows.push(cx.container(row_props, |_cx| Vec::new()));
                            }
                        }
                        rows
                    },
                );

                let mut scroll_layout = fret_ui::element::LayoutStyle::default();
                scroll_layout.size.width = fret_ui::element::Length::Px(Px(240.0));
                scroll_layout.size.height = fret_ui::element::Length::Px(Px(200.0));
                scroll_layout.overflow = fret_ui::element::Overflow::Clip;

                let scroll = cx
                    .scroll(
                        fret_ui::element::ScrollProps {
                            layout: scroll_layout,
                            axis: fret_ui::element::ScrollAxis::Y,
                            scroll_handle: Some(scroll_handle.clone()),
                            ..Default::default()
                        },
                        |_cx| vec![content],
                    )
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .test_id("ui-gallery-scroll-area-drag-baseline-viewport"),
                    );

                let scrollbar_layout = fret_ui::element::LayoutStyle {
                    position: fret_ui::element::PositionStyle::Absolute,
                    inset: fret_ui::element::InsetStyle {
                        top: Some(Px(0.0)).into(),
                        right: Some(Px(0.0)).into(),
                        bottom: Some(Px(0.0)).into(),
                        left: None.into(),
                    },
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Px(Px(12.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let mut scrollbar_fill = fret_ui::element::LayoutStyle::default();
                scrollbar_fill.size.width = fret_ui::element::Length::Fill;
                scrollbar_fill.size.height = fret_ui::element::Length::Fill;

                let scrollbar = cx
                    .scrollbar(fret_ui::element::ScrollbarProps {
                        layout: scrollbar_fill,
                        axis: fret_ui::element::ScrollbarAxis::Vertical,
                        scroll_target: Some(scroll.id),
                        scroll_handle: scroll_handle.clone(),
                        style: fret_ui::element::ScrollbarStyle::default(),
                    })
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .test_id("ui-gallery-scroll-area-drag-baseline-y-scrollbar"),
                    );

                let mut stack_layout = fret_ui::element::LayoutStyle::default();
                stack_layout.size.width = fret_ui::element::Length::Px(Px(240.0));
                stack_layout.size.height = fret_ui::element::Length::Px(Px(200.0));

                let scroll_group = scroll.attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Group)
                        .test_id("ui-gallery-scroll-area-drag-baseline"),
                );

                let body = cx.stack_props(
                    fret_ui::element::StackProps {
                        layout: stack_layout,
                    },
                    move |_cx| {
                        vec![
                            scroll_group,
                            _cx.interactivity_gate_props(
                                fret_ui::element::InteractivityGateProps {
                                    layout: scrollbar_layout,
                                    present: true,
                                    interactive: true,
                                },
                                |_cx| vec![scrollbar],
                            ),
                        ]
                    },
                );

                vec![
                    ui::v_flex(cx, |_cx| [controls, instructions, body])
                        .gap(Space::N2)
                        .into_element(cx),
                ]
            },
        )
    })
}
// endregion: example
