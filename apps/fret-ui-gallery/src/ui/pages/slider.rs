use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_slider(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.slider_page", |cx| {
        let theme = Theme::global(&*cx.app).snapshot();

        #[derive(Default)]
        struct SliderPageState {
            last_commit: Option<Model<Vec<f32>>>,
            controlled_values: Option<Model<Vec<f32>>>,
        }

        let max_width_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

        let last_commit = cx.with_state(SliderPageState::default, |st| st.last_commit.clone());
        let last_commit = match last_commit {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(Vec::<f32>::new());
                cx.with_state(SliderPageState::default, |st| {
                    st.last_commit = Some(model.clone());
                });
                model
            }
        };

        let controlled_values =
            cx.with_state(SliderPageState::default, |st| st.controlled_values.clone());
        let controlled_values = match controlled_values {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(vec![0.3, 0.7]);
                cx.with_state(SliderPageState::default, |st| {
                    st.controlled_values = Some(model.clone());
                });
                model
            }
        };

        let controlled = |cx: &mut ElementContext<'_, App>| {
            let values_snapshot = cx
                .watch_model(&controlled_values)
                .layout()
                .cloned()
                .unwrap_or_default();
            let values_text = values_snapshot
                .iter()
                .map(|v| format!("{v:.1}"))
                .collect::<Vec<_>>()
                .join(", ");

            let header = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .items_center()
                    .justify_between(),
                |cx| {
                    vec![
                        shadcn::Label::new("Temperature").into_element(cx),
                        shadcn::typography::muted(cx, values_text),
                    ]
                },
            );
            let slider = shadcn::Slider::new(controlled_values.clone())
                .range(0.0, 1.0)
                .step(0.1)
                .test_id("ui-gallery-slider-controlled")
                .a11y_label("Temperature")
                .into_element(cx);

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
                |_cx| vec![header, slider],
            )
        };

        let demo = cx.keyed("ui_gallery.slider.demo", |cx| {
            let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));

            doc_layout::wrap_row_snapshot(
                cx,
                &theme,
                Space::N6,
                fret_ui::element::CrossAlign::Start,
                |cx| {
                    let single = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
                        .range(0.0, 100.0)
                        .step(1.0)
                        .test_id("ui-gallery-slider-demo-single")
                        .a11y_label("Slider")
                        .refine_layout(max_w_sm.clone())
                        .into_element(cx);

                    let range = shadcn::Slider::new_controllable(cx, None, || vec![25.0, 50.0])
                        .range(0.0, 100.0)
                        .step(1.0)
                        .test_id("ui-gallery-slider-demo-range")
                        .a11y_label("Range slider")
                        .refine_layout(max_w_sm.clone())
                        .into_element(cx);

                    let multiple = shadcn::Slider::new_controllable(cx, None, || vec![10.0, 20.0])
                        .range(0.0, 100.0)
                        .step(10.0)
                        .test_id("ui-gallery-slider-demo-multiple")
                        .a11y_label("Multiple thumbs slider")
                        .refine_layout(max_w_sm.clone())
                        .into_element(cx);

                    let vertical = {
                        let a = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
                            .range(0.0, 100.0)
                            .step(1.0)
                            .orientation(fret_ui_kit::primitives::slider::SliderOrientation::Vertical)
                            .refine_layout(LayoutRefinement::default().h_px(Px(160.0)))
                            .test_id("ui-gallery-slider-demo-vertical-a")
                            .a11y_label("Vertical slider")
                            .into_element(cx);

                        let b = shadcn::Slider::new_controllable(cx, None, || vec![25.0])
                            .range(0.0, 100.0)
                            .step(1.0)
                            .orientation(fret_ui_kit::primitives::slider::SliderOrientation::Vertical)
                            .refine_layout(LayoutRefinement::default().h_px(Px(160.0)))
                            .test_id("ui-gallery-slider-demo-vertical-b")
                            .a11y_label("Vertical slider")
                            .into_element(cx);

                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .gap(Space::N6)
                                .items_center()
                                .layout(LayoutRefinement::default().w_full().min_w_0()),
                            |_cx| vec![a, b],
                        )
                        .test_id("ui-gallery-slider-demo-vertical")
                    };

                    let controlled = controlled(cx).test_id("ui-gallery-slider-demo-controlled");

                    vec![single, range, multiple, vertical, controlled]
                },
            )
            .test_id("ui-gallery-slider-demo")
        });

        let on_value_commit = cx.keyed("ui_gallery.slider.on_value_commit", |cx| {
            let last_commit_for_cb = last_commit.clone();
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![75.0])
                .range(0.0, 100.0)
                .test_id("ui-gallery-slider-on-value-commit")
                .a11y_label("Slider")
                .refine_layout(max_width_xs.clone())
                .on_value_commit(move |host, _cx, values| {
                    let _ = host.models_mut().update(&last_commit_for_cb, |v| {
                        *v = values;
                    });
                })
                .into_element(cx);

            let last_commit_values = cx
                .watch_model(&last_commit)
                .layout()
                .cloned()
                .unwrap_or_default();
            let last_commit_text = if last_commit_values.is_empty() {
                "<none>".to_string()
            } else {
                format!("{last_commit_values:?}")
            };
            let meta = shadcn::typography::muted(cx, format!("onValueCommit: {last_commit_text}"));

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
                |_cx| vec![slider, meta],
            )
        });

        let disabled = cx.keyed("ui_gallery.slider.disabled", |cx| {
            shadcn::Slider::new_controllable(cx, None, || vec![50.0])
                .range(0.0, 100.0)
                .step(1.0)
                .disabled(true)
                .test_id("ui-gallery-slider-disabled")
                .a11y_label("Disabled slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx)
        });

        let rtl = cx.keyed("ui_gallery.slider.rtl", |cx| {
            shadcn::Slider::new_controllable(cx, None, || vec![75.0])
                .range(0.0, 100.0)
                .step(1.0)
                .dir(fret_ui_kit::primitives::direction::LayoutDirection::Rtl)
                .test_id("ui-gallery-slider-rtl")
                .a11y_label("RTL slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx)
        });

        let inverted = cx.keyed("ui_gallery.slider.inverted", |cx| {
            shadcn::Slider::new_controllable(cx, None, || vec![25.0])
                .range(0.0, 100.0)
                .step(1.0)
                .inverted(true)
                .test_id("ui-gallery-slider-inverted")
                .a11y_label("Inverted slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx)
        });

        let notes = doc_layout::notes(
            cx,
            [
                "Uncontrolled sliders store their values in element state; controlled sliders store values in a shared model.",
                "Prefer `on_value_commit` for expensive reactions (e.g. save, fetch) and use live updates for lightweight UI.",
                "Vertical sliders should have an explicit height to avoid zero-size layouts.",
            ],
        );

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
                        "Extras are Fret-specific demos and regression gates (not part of upstream shadcn SliderDemo).",
                    ),
                    on_value_commit,
                    disabled,
                    rtl,
                    inverted,
                ]
            },
        )
        .test_id("ui-gallery-slider-extras");

        let body = doc_layout::render_doc_page(
            cx,
            Some("Demo matches shadcn `SliderDemo` (new-york-v4). Extras cover Fret-specific variants."),
            vec![
                DocSection::new("Demo", demo)
                    .description("shadcn demo: single, range, multiple, vertical, and controlled.")
                    .max_w(Px(520.0))
                    .code(
                        "rust",
                        r#"shadcn::Slider::new_controllable(cx, None, || vec![50.0]).into_element(cx);
shadcn::Slider::new_controllable(cx, None, || vec![25.0, 50.0]).into_element(cx);
shadcn::Slider::new_controllable(cx, None, || vec![10.0, 20.0])
    .step(10.0)
    .into_element(cx);"#,
                    ),
                DocSection::new("Extras", extras)
                    .description("Fret extras: disabled, RTL, inverted, and onValueCommit.")
                    .max_w(Px(520.0))
                    .code(
                        "rust",
                        r#"shadcn::Slider::new_controllable(cx, None, || vec![50.0])
    .disabled(true)
    .into_element(cx);

shadcn::Slider::new_controllable(cx, None, || vec![75.0])
    .dir(fret_ui_kit::primitives::direction::LayoutDirection::Rtl)
    .into_element(cx);

shadcn::Slider::new_controllable(cx, None, || vec![25.0])
    .inverted(true)
    .into_element(cx);

shadcn::Slider::new_controllable(cx, None, || vec![75.0])
    .on_value_commit(|host, _cx, values| { /* ... */ })
    .into_element(cx);"#,
                    ),
                DocSection::new("Notes", notes).description("Behavior notes."),
            ],
        );

        vec![body.test_id("ui-gallery-slider")]
    })
}
