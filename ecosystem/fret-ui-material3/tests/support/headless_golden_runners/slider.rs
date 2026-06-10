use std::collections::BTreeMap;

use fret_core::{AppWindowId, KeyCode, Point, PointerId, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;

use super::{MATERIAL3_HEADLESS_SCALE_FACTORS_V1, MATERIAL3_HEADLESS_SCHEMES_V1, scale_segment};
use crate::support::{
    events::{key_down, key_up, pointer_down, pointer_move},
    goldens::{
        Material3HeadlessGoldenV1, Material3HeadlessSuiteV1, settle_material3_scene_snapshot_v1,
        write_or_assert_material3_suite_for_test_v1,
    },
    headless_interactions::{dispatch_idle_pointer, focus_test_id, hover_test_id},
    headless_slider_cases::{
        Material3SliderKeyboardInteractionV1, Material3SliderPointerInteractionV1,
        load_material3_slider_golden_suite_v1,
    },
    host::{FakeUiServices, TestHost},
    layout::with_padding,
    theme::{apply_material_theme, apply_material_theme_rtl},
};

pub(crate) fn run_material3_headless_slider_suite_goldens_v1() {
    use fret_ui::element::FlexProps;

    let slider_suite = load_material3_slider_golden_suite_v1();

    for scale_factor in MATERIAL3_HEADLESS_SCALE_FACTORS_V1 {
        let scale = scale_segment(scale_factor);

        for scheme in MATERIAL3_HEADLESS_SCHEMES_V1 {
            let mode = scheme.mode;
            let variant = scheme.variant;
            let label = scheme.label;
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(520.0), Px(320.0)),
            );

            let mut cases: BTreeMap<String, Material3HeadlessGoldenV1> = BTreeMap::new();

            for case in slider_suite.cases() {
                let case_name = case.id();
                let window = AppWindowId::default();
                let mut app = TestHost::default();
                app.set_global(PlatformCapabilities::default());
                if case.is_rtl() {
                    apply_material_theme_rtl(&mut app, mode, variant);
                } else {
                    apply_material_theme(&mut app, mode, variant);
                }

                let mut services = FakeUiServices;
                let mut ui: UiTree<TestHost> = UiTree::new();
                ui.set_window(window);

                let single_value_models = slider_suite
                    .single_value_models()
                    .iter()
                    .map(|definition| {
                        (
                            definition.id().to_string(),
                            app.models_mut().insert(definition.value()),
                        )
                    })
                    .collect::<BTreeMap<_, _>>();
                let range_value_models = slider_suite
                    .range_value_models()
                    .iter()
                    .map(|definition| {
                        (
                            definition.id().to_string(),
                            app.models_mut().insert(definition.values()),
                        )
                    })
                    .collect::<BTreeMap<_, _>>();
                let render_config = slider_suite.render_config_for(case);

                let render = |ui: &mut UiTree<TestHost>,
                              app: &mut TestHost,
                              services: &mut dyn UiServices| {
                    fret_ui::declarative::render_root(
                        ui,
                        app,
                        services,
                        window,
                        bounds,
                        "root",
                        |cx| {
                            let mut props = FlexProps::default();
                            props.direction = fret_core::Axis::Vertical;
                            props.gap = fret_ui::element::SpacingLength::Px(Px(24.0));
                            let content = cx.flex(props, |cx| {
                                let mut elements = Vec::new();

                                for slider in slider_suite.sliders() {
                                    let model = single_value_models
                                        .get(slider.model_id())
                                        .unwrap_or_else(|| {
                                            panic!(
                                                "expected single slider model {} ({label}, {scale}, {case_name})",
                                                slider.model_id()
                                            )
                                        })
                                        .clone();
                                    elements.push(slider.slider(model, render_config).into_element(cx));
                                }

                                for slider in slider_suite.range_sliders() {
                                    let model = range_value_models
                                        .get(slider.model_id())
                                        .unwrap_or_else(|| {
                                            panic!(
                                                "expected range slider model {} ({label}, {scale}, {case_name})",
                                                slider.model_id()
                                            )
                                        })
                                        .clone();
                                    elements.push(
                                        slider.range_slider(model, render_config).into_element(cx),
                                    );
                                }

                                elements
                            });

                            vec![with_padding(cx, Px(24.0), content)]
                        },
                    )
                };

                let root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.request_semantics_snapshot();
                ui.layout_all(&mut app, &mut services, bounds, scale_factor);

                let interaction_context = format!("{label}, {scale}, {case_name}");
                dispatch_idle_pointer(&mut ui, &mut app, &mut services);

                if let Some(test_id) = case.hover_test_id() {
                    let hover_target = hover_test_id(
                        &mut ui,
                        &mut app,
                        &mut services,
                        test_id,
                        &interaction_context,
                    );
                    let node_bounds = hover_target.bounds;
                    let hover_at = hover_target.center;

                    match case.pointer_interaction() {
                        Some(Material3SliderPointerInteractionV1::Pressed) => {
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &pointer_down(PointerId(1), hover_at),
                            );
                        }
                        Some(Material3SliderPointerInteractionV1::Dragging) => {
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &pointer_down(PointerId(1), hover_at),
                            );
                            let drag_to = Point::new(
                                Px(node_bounds.origin.x.0 + node_bounds.size.width.0 * 0.8),
                                Px(node_bounds.origin.y.0 + node_bounds.size.height.0 * 0.5),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &pointer_move(PointerId(1), drag_to),
                            );
                        }
                        Some(Material3SliderPointerInteractionV1::RangeDragging) => {
                            let start_at = Point::new(
                                Px(node_bounds.origin.x.0 + node_bounds.size.width.0 * 0.85),
                                Px(node_bounds.origin.y.0 + node_bounds.size.height.0 * 0.5),
                            );
                            let drag_to = Point::new(
                                Px(node_bounds.origin.x.0 + node_bounds.size.width.0 * 0.95),
                                Px(node_bounds.origin.y.0 + node_bounds.size.height.0 * 0.5),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &pointer_move(PointerId(1), start_at),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &pointer_down(PointerId(1), start_at),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &pointer_move(PointerId(1), drag_to),
                            );
                        }
                        None => {}
                    }
                }

                if let Some(test_id) = case.focus_test_id() {
                    focus_test_id(&mut ui, test_id, &interaction_context);

                    match case.keyboard_interaction() {
                        Some(Material3SliderKeyboardInteractionV1::SingleArrowCycle) => {
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowLeft),
                            );
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));
                        }
                        Some(Material3SliderKeyboardInteractionV1::SinglePageHomeEnd) => {
                            let model_id = case.assert_model_id().unwrap_or_else(|| {
                                panic!(
                                    "expected single slider assert model id ({label}, {scale}, {case_name})"
                                )
                            });
                            let value_model =
                                single_value_models.get(model_id).unwrap_or_else(|| {
                                    panic!(
                                        "expected single slider assert model {model_id} ({label}, {scale}, {case_name})"
                                    )
                                });

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::PageUp));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageUp));
                            let after_page_up = app
                                .models_mut()
                                .read(value_model, |v| *v)
                                .ok()
                                .unwrap_or(0.0);
                            assert!(
                                (after_page_up - 0.4).abs() <= 1e-6,
                                "expected slider PageUp to increment by a page (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::PageDown),
                            );
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageDown));
                            let after_page_down = app
                                .models_mut()
                                .read(value_model, |v| *v)
                                .ok()
                                .unwrap_or(0.0);
                            assert!(
                                (after_page_down - 0.3).abs() <= 1e-6,
                                "expected slider PageDown to decrement by a page (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));
                            let after_home = app
                                .models_mut()
                                .read(value_model, |v| *v)
                                .ok()
                                .unwrap_or(0.0);
                            assert!(
                                after_home.abs() <= 1e-6,
                                "expected slider Home to snap to min (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::End));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::End));
                            let after_end = app
                                .models_mut()
                                .read(value_model, |v| *v)
                                .ok()
                                .unwrap_or(0.0);
                            assert!(
                                (after_end - 1.0).abs() <= 1e-6,
                                "expected slider End to snap to max (case={case_name}, {label}, {scale})"
                            );
                        }
                        Some(Material3SliderKeyboardInteractionV1::SingleRtlArrowCycle) => {
                            let model_id = case.assert_model_id().unwrap_or_else(|| {
                                panic!(
                                    "expected single slider assert model id ({label}, {scale}, {case_name})"
                                )
                            });
                            let value_model =
                                single_value_models.get(model_id).unwrap_or_else(|| {
                                    panic!(
                                        "expected single slider assert model {model_id} ({label}, {scale}, {case_name})"
                                    )
                                });

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );
                            let after_right = app
                                .models_mut()
                                .read(value_model, |v| *v)
                                .ok()
                                .unwrap_or(0.0);
                            assert!(
                                (after_right - 0.29).abs() <= 1e-6,
                                "expected slider ArrowRight to decrement under RTL (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowLeft),
                            );
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));
                            let after_left = app
                                .models_mut()
                                .read(value_model, |v| *v)
                                .ok()
                                .unwrap_or(0.0);
                            assert!(
                                (after_left - 0.30).abs() <= 1e-6,
                                "expected slider ArrowLeft to increment under RTL (case={case_name}, {label}, {scale})"
                            );
                        }
                        Some(Material3SliderKeyboardInteractionV1::RangeThumbSwitch) => {
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );

                            let end_test_id = case.secondary_focus_test_id().unwrap_or_else(|| {
                                panic!(
                                    "expected range slider secondary focus test id ({label}, {scale}, {case_name})"
                                )
                            });
                            focus_test_id(&mut ui, end_test_id, &interaction_context);

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );
                        }
                        Some(Material3SliderKeyboardInteractionV1::RangePageHomeEnd) => {
                            let model_id = case.assert_model_id().unwrap_or_else(|| {
                                panic!(
                                    "expected range slider assert model id ({label}, {scale}, {case_name})"
                                )
                            });
                            let range_model =
                                range_value_models.get(model_id).unwrap_or_else(|| {
                                    panic!(
                                        "expected range slider assert model {model_id} ({label}, {scale}, {case_name})"
                                    )
                                });

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::PageUp));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageUp));
                            let after_page_up = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                (after_page_up[0] - 0.4).abs() <= 1e-6
                                    && (after_page_up[1] - 0.7).abs() <= 1e-6,
                                "expected range slider start PageUp to increment start by a page (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::PageDown),
                            );
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageDown));
                            let after_page_down = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                (after_page_down[0] - 0.3).abs() <= 1e-6
                                    && (after_page_down[1] - 0.7).abs() <= 1e-6,
                                "expected range slider start PageDown to decrement start by a page (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));
                            let after_home = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                after_home[0].abs() <= 1e-6 && (after_home[1] - 0.7).abs() <= 1e-6,
                                "expected range slider start Home to snap to min (case={case_name}, {label}, {scale})"
                            );

                            let end_test_id = case.secondary_focus_test_id().unwrap_or_else(|| {
                                panic!(
                                    "expected range slider secondary focus test id ({label}, {scale}, {case_name})"
                                )
                            });
                            focus_test_id(&mut ui, end_test_id, &interaction_context);

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::PageDown),
                            );
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageDown));
                            let after_end_page_down = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                after_end_page_down[0].abs() <= 1e-6
                                    && (after_end_page_down[1] - 0.6).abs() <= 1e-6,
                                "expected range slider end PageDown to decrement end by a page (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::PageUp));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::PageUp));
                            let after_end_page_up = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                after_end_page_up[0].abs() <= 1e-6
                                    && (after_end_page_up[1] - 0.7).abs() <= 1e-6,
                                "expected range slider end PageUp to increment end by a page (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));
                            let after_end_home = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                after_end_home[0].abs() <= 1e-6 && after_end_home[1].abs() <= 1e-6,
                                "expected range slider end Home to snap to start value (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::End));
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::End));
                            let after_end_end = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                after_end_end[0].abs() <= 1e-6
                                    && (after_end_end[1] - 1.0).abs() <= 1e-6,
                                "expected range slider end End to snap to max (case={case_name}, {label}, {scale})"
                            );
                        }
                        Some(Material3SliderKeyboardInteractionV1::RangeRtlArrowCycle) => {
                            let model_id = case.assert_model_id().unwrap_or_else(|| {
                                panic!(
                                    "expected range slider assert model id ({label}, {scale}, {case_name})"
                                )
                            });
                            let range_model =
                                range_value_models.get(model_id).unwrap_or_else(|| {
                                    panic!(
                                        "expected range slider assert model {model_id} ({label}, {scale}, {case_name})"
                                    )
                                });

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowRight),
                            );
                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_up(KeyCode::ArrowRight),
                            );
                            let after_right = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                (after_right[0] - 0.29).abs() <= 1e-6
                                    && (after_right[1] - 0.7).abs() <= 1e-6,
                                "expected range slider start ArrowRight to decrement under RTL (case={case_name}, {label}, {scale})"
                            );

                            ui.dispatch_event(
                                &mut app,
                                &mut services,
                                &key_down(KeyCode::ArrowLeft),
                            );
                            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));
                            let after_left = app
                                .models_mut()
                                .read(range_model, |v| *v)
                                .ok()
                                .unwrap_or([0.0, 0.0]);
                            assert!(
                                (after_left[0] - 0.30).abs() <= 1e-6
                                    && (after_left[1] - 0.7).abs() <= 1e-6,
                                "expected range slider start ArrowLeft to increment under RTL (case={case_name}, {label}, {scale})"
                            );
                        }
                        None => {}
                    }
                }

                let message = format!(
                    "expected the Material3 slider scene to be stable after animations settle ({label}, {scale}, {case_name})"
                );
                cases.insert(
                    case.id().to_string(),
                    settle_material3_scene_snapshot_v1(
                        &mut app,
                        &mut ui,
                        &mut services,
                        bounds,
                        scale_factor,
                        case.settle_from_frame(),
                        case.total_frames(),
                        &message,
                        &render,
                    ),
                );
            }

            let suite = Material3HeadlessSuiteV1 { cases };
            write_or_assert_material3_suite_for_test_v1(
                &format!("material3-slider.{scale}.{label}"),
                "material3_headless_slider_suite_goldens_v1",
                &suite,
            );
        }
    }
}
