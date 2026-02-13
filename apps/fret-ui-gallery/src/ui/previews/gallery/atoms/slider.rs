use super::super::super::super::*;

pub(in crate::ui) fn preview_slider(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.slider_page", |cx| {
        #[derive(Default)]
        struct SliderPageState {
            last_commit: Option<Model<Vec<f32>>>,
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

        let demo = cx.keyed("ui_gallery.slider.demo", |cx| {
            let last_commit_for_cb = last_commit.clone();
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![75.0])
                .range(0.0, 100.0)
                .test_id("ui-gallery-slider-single")
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

            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
                |_cx| vec![slider, meta],
            );
            let body = centered(cx, body);
            section(cx, "Demo", body)
        });

        let range = cx.keyed("ui_gallery.slider.range", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![25.0, 50.0])
                .range(0.0, 100.0)
                .step(5.0)
                .test_id("ui-gallery-slider-range")
                .a11y_label("Range slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "Range", body)
        });

        let multiple = cx.keyed("ui_gallery.slider.multiple", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![10.0, 20.0, 70.0])
                .range(0.0, 100.0)
                .step(10.0)
                .test_id("ui-gallery-slider-multiple")
                .a11y_label("Multiple thumbs slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "Multiple Thumbs", body)
        });

        let vertical = cx.keyed("ui_gallery.slider.vertical", |cx| {
            let a = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
                .range(0.0, 100.0)
                .step(1.0)
                .orientation(fret_ui_kit::primitives::slider::SliderOrientation::Vertical)
                .refine_layout(LayoutRefinement::default().h_px(Px(160.0)))
                .test_id("ui-gallery-slider-vertical")
                .a11y_label("Vertical slider")
                .into_element(cx);

            let b = shadcn::Slider::new_controllable(cx, None, || vec![25.0])
                .range(0.0, 100.0)
                .step(1.0)
                .orientation(fret_ui_kit::primitives::slider::SliderOrientation::Vertical)
                .refine_layout(LayoutRefinement::default().h_px(Px(160.0)))
                .a11y_label("Vertical slider")
                .into_element(cx);

            let body = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N6)
                    .items_center()
                    .justify_center()
                    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
                |_cx| vec![a, b],
            );

            section(cx, "Vertical", body)
        });

        let controlled = cx.keyed("ui_gallery.slider.controlled", |cx| {
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

            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N3)
                    .layout(LayoutRefinement::default().w_full().max_w(Px(320.0))),
                |_cx| vec![header, slider],
            );

            let body = centered(cx, body);
            section(cx, "Controlled", body)
        });

        let disabled = cx.keyed("ui_gallery.slider.disabled", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
                .range(0.0, 100.0)
                .step(1.0)
                .disabled(true)
                .test_id("ui-gallery-slider-disabled")
                .a11y_label("Disabled slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "Disabled", body)
        });

        let rtl = cx.keyed("ui_gallery.slider.rtl", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![75.0])
                .range(0.0, 100.0)
                .step(1.0)
                .dir(fret_ui_kit::primitives::direction::LayoutDirection::Rtl)
                .test_id("ui-gallery-slider-rtl")
                .a11y_label("RTL slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "RTL", body)
        });

        let inverted = cx.keyed("ui_gallery.slider.inverted", |cx| {
            let slider = shadcn::Slider::new_controllable(cx, None, || vec![25.0])
                .range(0.0, 100.0)
                .step(1.0)
                .inverted(true)
                .test_id("ui-gallery-slider-inverted")
                .a11y_label("Inverted slider")
                .refine_layout(max_width_xs.clone())
                .into_element(cx);
            let body = centered(cx, slider);
            section(cx, "Extras: Inverted", body)
        });

        vec![
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N6)
                    .items_start(),
                |_cx| vec![
                    demo,
                    range,
                    multiple,
                    vertical,
                    controlled,
                    disabled,
                    rtl,
                    inverted,
                ],
            ),
            shadcn::typography::muted(
                cx,
                "Note: demo/range/multiple/vertical/disabled/RTL are uncontrolled (element state). Controlled uses a shared model."
                    .to_string(),
            ),
        ]
    })
}
