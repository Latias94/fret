use super::super::super::super::*;

pub(in crate::ui) fn preview_progress(
    cx: &mut ElementContext<'_, App>,
    _progress: Model<f32>,
) -> Vec<AnyElement> {
    use std::time::Duration;

    use fret_core::{SemanticsRole, TimerToken};
    use fret_runtime::Effect;
    use fret_ui::Invalidation;
    use fret_ui::element::SemanticsProps;
    use fret_ui_kit::primitives::direction as direction_prim;

    #[derive(Default, Clone)]
    struct ProgressModels {
        demo_value: Option<Model<f32>>,
        demo_token: Option<Model<Option<TimerToken>>>,
        label_value: Option<Model<f32>>,
        controlled_values: Option<Model<Vec<f32>>>,
    }

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let state = cx.with_state(ProgressModels::default, |st| st.clone());

    let demo_value = match state.demo_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(13.0);
            cx.with_state(ProgressModels::default, |st| {
                st.demo_value = Some(model.clone())
            });
            model
        }
    };

    let demo_token = match state.demo_token {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<TimerToken>);
            cx.with_state(ProgressModels::default, |st| {
                st.demo_token = Some(model.clone())
            });
            model
        }
    };

    let label_value = match state.label_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(66.0);
            cx.with_state(ProgressModels::default, |st| {
                st.label_value = Some(model.clone())
            });
            model
        }
    };

    let controlled_values = match state.controlled_values {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![50.0]);
            cx.with_state(ProgressModels::default, |st| {
                st.controlled_values = Some(model.clone())
            });
            model
        }
    };

    let demo = cx.keyed("ui_gallery.progress.demo", |cx| {
        let demo_value_for_timer = demo_value.clone();
        let demo_token_for_timer = demo_token.clone();

        let body = cx.semantics_with_id(
            SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("ui-gallery-progress-demo")),
                ..Default::default()
            },
            move |cx, id| {
                cx.timer_on_timer_for(
                    id,
                    Arc::new(move |host, action_cx, token| {
                        let expected = host
                            .models_mut()
                            .read(&demo_token_for_timer, Clone::clone)
                            .ok()
                            .flatten();
                        if expected != Some(token) {
                            return false;
                        }
                        let _ = host
                            .models_mut()
                            .update(&demo_value_for_timer, |v| *v = 66.0);
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    }),
                );

                let armed = cx
                    .get_model_copied(&demo_token, Invalidation::Paint)
                    .unwrap_or(None)
                    .is_some();
                if !armed {
                    let token = cx.app.next_timer_token();
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&demo_token, |v| *v = Some(token));
                    let _ = cx.app.models_mut().update(&demo_value, |v| *v = 13.0);
                    cx.app.push_effect(Effect::SetTimer {
                        window: Some(cx.window),
                        token,
                        after: Duration::from_millis(500),
                        repeat: None,
                    });
                }

                let bar = shadcn::Progress::new(demo_value.clone())
                    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                    .into_element(cx);

                vec![centered(cx, bar)]
            },
        );

        section(cx, "Demo", body)
    });

    let label = cx.keyed("ui_gallery.progress.label", |cx| {
        let label_row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center(),
            |cx| {
                vec![
                    shadcn::FieldLabel::new("Upload progress").into_element(cx),
                    shadcn::FieldLabel::new("66%")
                        .refine_layout(LayoutRefinement::default().ml_auto())
                        .into_element(cx),
                ]
            },
        );

        let field = shadcn::Field::new(vec![
            label_row,
            shadcn::Progress::new(label_value.clone()).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx);

        let body = centered(cx, field);
        section(cx, "Label", body)
    });

    let controlled = cx.keyed("ui_gallery.progress.controlled", |cx| {
        let values = controlled_values.clone();
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |cx| {
                vec![
                    shadcn::Progress::new_values_first(values.clone()).into_element(cx),
                    shadcn::Slider::new(values)
                        .range(0.0, 100.0)
                        .step(1.0)
                        .a11y_label("Progress value")
                        .into_element(cx),
                ]
            },
        );

        let centered_body = centered(cx, body);
        section(cx, "Controlled", centered_body)
    });

    let rtl = cx.keyed("ui_gallery.progress.rtl", |cx| {
        let body = direction_prim::with_direction_provider(
            cx,
            direction_prim::LayoutDirection::Rtl,
            |cx| {
                let label_row = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .items_center(),
                    |cx| {
                        vec![
                            shadcn::FieldLabel::new("٦٦%").into_element(cx),
                            shadcn::FieldLabel::new("تقدم الرفع")
                                .refine_layout(LayoutRefinement::default().ml_auto())
                                .into_element(cx),
                        ]
                    },
                );

                let field = shadcn::Field::new(vec![
                    label_row,
                    shadcn::Progress::new(label_value.clone())
                        .mirror_in_rtl(true)
                        .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
                .into_element(cx);

                centered(cx, field)
            },
        );

        section(cx, "RTL", body)
    });

    let examples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![label, controlled, rtl],
    );

    vec![demo, examples]
}
