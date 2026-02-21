use super::super::super::super::*;

pub(in crate::ui) fn preview_progress(
    cx: &mut ElementContext<'_, App>,
    _progress: Model<f32>,
) -> Vec<AnyElement> {
    use std::time::Duration;

    use crate::ui::doc_layout::{self, DocSection};
    use fret_core::{SemanticsRole, TimerToken};
    use fret_runtime::Effect;
    use fret_ui::Invalidation;
    use fret_ui::element::SemanticsProps;

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

        body.test_id("ui-gallery-progress-demo")
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

        centered(cx, field).test_id("ui-gallery-progress-label")
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
        centered_body.test_id("ui-gallery-progress-controlled")
    });

    let rtl = cx.keyed("ui_gallery.progress.rtl", |cx| {
        doc_layout::rtl(cx, |cx| {
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
        })
        .test_id("ui-gallery-progress-rtl")
    });

    let extras = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Extras are Fret-specific recipes and regression gates (not part of upstream shadcn ProgressDemo).",
                ),
                label,
                controlled,
            ]
        },
    )
    .test_id("ui-gallery-progress-extras");

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Progress demo (new-york-v4).",
            "The demo uses a one-shot timer (500ms) to update the progress value from 13 → 66.",
            "For labeled progress, prefer composing `FieldLabel` + `Progress` instead of adding one-off widget APIs.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Progress demo: value update after 500ms."),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-progress-demo")
                .code(
                    "rust",
                    r#"let progress = cx.app.models_mut().insert(13.0);

// After 500ms, update progress to 66.
cx.app.push_effect(Effect::SetTimer {
    window: Some(cx.window),
    token,
    after: Duration::from_millis(500),
    repeat: None,
});

shadcn::Progress::new(progress)
    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
    .into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-progress-rtl")
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::Progress::new(value).mirror_in_rtl(true).into_element(cx)
});"#,
                ),
            DocSection::new("Extras", extras)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-progress-extras")
                .code(
                    "rust",
                    r#"// Label
let field = shadcn::Field::new([label_row, shadcn::Progress::new(value).into_element(cx)])
    .into_element(cx);

// Controlled
let values = cx.app.models_mut().insert(vec![50.0]);
shadcn::Progress::new_values_first(values.clone()).into_element(cx);"#,
                ),
            DocSection::new("Notes", notes).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-progress")]
}
