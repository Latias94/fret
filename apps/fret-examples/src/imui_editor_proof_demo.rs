use std::collections::HashMap;
use std::sync::Arc;

use fret::interop::embedded_viewport as embedded;
use fret::prelude::*;
use fret_app::{CreateWindowKind, CreateWindowRequest, WindowRequest};
use fret_core::{Axis, Color, Edges, Px};
use fret_docking::{
    DockManager, DockPanel, DockPanelRegistry, DockPanelRegistryService, ViewportPanel,
    runtime as dock_runtime,
};
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::{
    ActivationPolicy, FrameId, Model, PlatformCapabilities, TickId, WindowHoverDetectionQuality,
    WindowRole, WindowStyleRequest,
};
use fret_ui::element::{CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle};
use fret_ui_editor::composites::{PropertyGrid, PropertyGroup, PropertyRow, PropertyRowReset};
use fret_ui_editor::controls::{
    Checkbox, DragValue, EnumSelect, EnumSelectItem, EnumSelectOptions, FieldStatus,
    FieldStatusBadge, MiniSearchBox, NumericFormatFn, NumericParseFn, NumericValidateFn,
};

const VIEWPORT_PX_SIZE: (u32, u32) = (960, 540);
const AUX_LOGICAL_WINDOW_ID: &str = "aux";
const ENV_SINGLE_WINDOW: &str = "FRET_IMUI_EDITOR_PROOF_SINGLE_WINDOW";

fn diag_enabled() -> bool {
    std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty() && v != "0")
}

struct ImUiEditorProofState {
    embedded: embedded::EmbeddedViewportSurface,
}

impl embedded::EmbeddedViewportRecord for ImUiEditorProofState {
    fn embedded_viewport_surface(&mut self) -> &mut embedded::EmbeddedViewportSurface {
        &mut self.embedded
    }

    fn embedded_viewport_label(&self) -> Option<&'static str> {
        Some("imui-editor-proof viewport")
    }

    fn record_embedded_viewport(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        _context: &WgpuContext,
        _renderer: &mut Renderer,
        _scale_factor: f32,
        _tick_id: TickId,
        frame_id: FrameId,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let t = (frame_id.0 as f32 * 0.02).sin() * 0.5 + 0.5;
        let clear = wgpu::Color {
            r: (0.08 + 0.30 * t) as f64,
            g: (0.08 + 0.25 * (1.0 - t)) as f64,
            b: (0.10 + 0.35 * (0.5 - (t - 0.5).abs())) as f64,
            a: 1.0,
        };
        embedded::clear_pass(encoder, view, Some("imui-editor-proof clear"), clear);
    }
}

pub fn run() -> anyhow::Result<()> {
    fret::app_with_hooks("imui-editor-proof-demo", init_window, view, |d| {
        d.drive_embedded_viewport()
            .dock_op(on_dock_op)
            .window_create_spec(window_create_spec)
            .window_created(window_created)
            .before_close_window(before_close_window)
    })?
    .with_main_window("imui_editor_proof_demo", (1120.0, 720.0))
    .init_app(|app| {
        configure_single_window_caps_if_requested(app);
        shadcn::shadcn_themes::apply_shadcn_new_york_v4(
            app,
            shadcn::shadcn_themes::ShadcnBaseColor::Slate,
            shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        );
        install_dock_panel_registry(app);
    })
    .run()?;
    Ok(())
}

fn single_window_mode_enabled() -> bool {
    std::env::var_os(ENV_SINGLE_WINDOW).is_some_and(|v| !v.is_empty() && v != "0")
}

fn configure_single_window_caps_if_requested(app: &mut App) {
    if !single_window_mode_enabled() {
        return;
    }

    // Simulate wasm/mobile-like constraints:
    // - no OS multi-window tear-off
    // - no reliable hover detection across windows
    app.with_global_mut(PlatformCapabilities::default, |caps, _app| {
        caps.ui.multi_window = false;
        caps.ui.window_tear_off = false;
        caps.ui.window_hover_detection = WindowHoverDetectionQuality::None;
    });
}

fn init_window(app: &mut App, window: AppWindowId) -> ImUiEditorProofState {
    embedded::ensure_models(app, window);
    if !single_window_mode_enabled() {
        ensure_aux_window_requested(app, window);
    }

    ImUiEditorProofState {
        embedded: embedded::EmbeddedViewportSurface::new(
            wgpu::TextureFormat::Bgra8UnormSrgb,
            RenderTargetColorSpace::Srgb,
            VIEWPORT_PX_SIZE,
        ),
    }
}

fn view(cx: &mut ElementContext<'_, App>, _st: &mut ImUiEditorProofState) -> ViewElements {
    let window = cx.window;
    let last_input: Arc<str> = embedded::models(&*cx.app, window)
        .and_then(|models| cx.watch_model(&models.last_input).paint().cloned())
        .unwrap_or_else(|| Arc::from("<embedded viewport models missing>"));

    let caps = cx
        .app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();
    let window_size = cx
        .app
        .global::<fret_core::WindowMetricsService>()
        .and_then(|svc| svc.inner_size(window));
    let single = single_window_mode_enabled();
    let logical_window_id = cx
        .app
        .global::<WindowBootstrapService>()
        .and_then(|svc| svc.logical_by_window.get(&window).cloned());
    let dock_test_id = if logical_window_id.as_deref() == Some("main") {
        Some("imui-editor-proof.main.dock")
    } else if logical_window_id.as_deref() == Some(AUX_LOGICAL_WINDOW_ID) {
        Some("imui-editor-proof.aux.dock")
    } else {
        None
    };
    let tab_drag_anchor_test_id = (diag_enabled() && logical_window_id.as_deref() == Some("main"))
        .then_some("imui-editor-proof.main.tab-drag-anchor");

    let editor_value_model = editor_demo_value_model(cx);
    let editor_roughness_model = editor_demo_roughness_model(cx);
    let editor_metallic_model = editor_demo_metallic_model(cx);
    let editor_alpha_clip_model = editor_demo_alpha_clip_model(cx);
    let editor_cast_shadows_model = editor_demo_cast_shadows_model(cx);
    let editor_shading_model = editor_demo_shading_model(cx);
    let editor_iterations_model = editor_demo_iterations_model(cx);
    let editor_search_model = editor_demo_search_model(cx);

    fret_imui::imui(cx, |ui| {
        use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
        use fret_ui_kit::imui::UiWriterUiKitExt as _;

        let root = fret_ui_kit::ui::v_flex_build(ui.cx_mut(), move |cx, out| {
            fret_imui::imui_build(cx, out, |ui| {
                let headline = fret_ui_kit::ui::text(
                    ui.cx_mut(),
                    format!(
                        "imui editor-grade proof (M7): docking + multi-window + viewport surfaces (window={window:?})"
                    ),
                )
                .font_semibold();
                ui.add_ui(headline);

                if single {
                    let hint = fret_ui_kit::ui::text(
                        ui.cx_mut(),
                        format!(
                            "single-window mode enabled ({ENV_SINGLE_WINDOW}=1): dock tear-off should degrade to in-window floating"
                        ),
                    )
                    .text_xs();
                    ui.add_ui(hint);
                }

                let caps_line = fret_ui_kit::ui::text(
                    ui.cx_mut(),
                    format!(
                        "caps: multi_window={} window_tear_off={} window_hover_detection={:?} window_inner_size={window_size:?}",
                        caps.ui.multi_window, caps.ui.window_tear_off, caps.ui.window_hover_detection,
                    ),
                )
                .text_xs();
                ui.add_ui(caps_line);

                let controls = fret_ui_kit::ui::h_flex_build(ui.cx_mut(), move |cx, out| {
                    fret_imui::imui_build(cx, out, |ui| {
                        if ui.button("Reset layout").clicked() {
                            reset_dock_graph(ui.cx_mut().app, window);
                            dock_runtime::request_dock_invalidation(ui.cx_mut().app, [window]);
                        }
                        if ui.button("Center floatings").clicked() {
                            dock_runtime::recenter_in_window_floatings(ui.cx_mut().app, window);
                        }
                    });
                })
                .gap(fret_ui_kit::Space::N2);
                ui.add_ui(controls);

                let last_input_line = fret_ui_kit::ui::text(
                    ui.cx_mut(),
                    format!("last viewport input: {last_input}"),
                )
                .text_xs();
                ui.add_ui(last_input_line);
                ui.separator();

                let editor_label =
                    fret_ui_kit::ui::text(ui.cx_mut(), "fret-ui-editor (M2): PropertyGroup + PropertyGrid + MiniSearchBox (filter)")
                        .text_xs();
                ui.add_ui(editor_label);
                ui.mount(|cx| {
                    let fmt: NumericFormatFn<f64> =
                        Arc::new(|v| Arc::from(format!("{v:.3}")));
                    let parse: NumericParseFn<f64> = Arc::new(|s| s.trim().parse::<f64>().ok());
                    let validate: NumericValidateFn<f64> = Arc::new(|v| {
                        if (0.0..=1.0).contains(&v) {
                            None
                        } else {
                            Some(Arc::from("Expected 0.0..=1.0"))
                        }
                    });

                    let query = cx
                        .get_model_cloned(&editor_search_model, fret_ui::Invalidation::Layout)
                        .unwrap_or_default();
                    let q = query.trim().to_lowercase();
                    let matches = |s: &str| q.is_empty() || s.to_lowercase().contains(&q);

                    let material_show_all = q.is_empty() || matches("material");
                    let show_opacity = material_show_all || matches("opacity");
                    let show_roughness = material_show_all || matches("roughness");
                    let show_metallic = material_show_all || matches("metallic");
                    let show_shading_model =
                        material_show_all || matches("shading") || matches("model");
                    let show_alpha_clip =
                        material_show_all || matches("alpha") || matches("clip");
                    let show_cast_shadows =
                        material_show_all || matches("shadow") || matches("shadows");

                    let advanced_show_all = q.is_empty() || matches("advanced");
                    let show_iterations = advanced_show_all || matches("iterations");

                    let any_match =
                        show_opacity
                            || show_roughness
                            || show_metallic
                            || show_shading_model
                            || show_alpha_clip
                            || show_cast_shadows
                            || show_iterations;

                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: Axis::Vertical,
                            gap: Px(8.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                        },
                        move |cx| {
                            let mut out = Vec::new();

                            out.push(
                                MiniSearchBox::new(editor_search_model.clone())
                                    .options(fret_ui_editor::controls::MiniSearchBoxOptions {
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.search",
                                        )),
                                        clear_test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.search.clear",
                                        )),
                                        ..Default::default()
                                    })
                                    .into_element(cx),
                            );

                            out.push(
                                PropertyGroup::new("Material")
                                    .options(fret_ui_editor::composites::PropertyGroupOptions {
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.group.material",
                                        )),
                                        header_test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.group.material.header",
                                        )),
                                        content_test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.group.material.content",
                                        )),
                                        ..Default::default()
                                    })
                                    .into_element(
                                        cx,
                                        |_cx| None,
                                        move |cx| {
                                            let fmt = fmt.clone();
                                            let parse = parse.clone();
                                            let validate = validate.clone();
                                            vec![PropertyGrid::new().into_element(
                                                cx,
                                                move |cx, row_cx| {
                                                    let mut rows = Vec::new();

                                                    if show_opacity {
                                                        let model_for_reset =
                                                            editor_value_model.clone();
                                                        let on_reset = Arc::new(
                                                            move |host: &mut dyn fret_ui::action::UiActionHost,
                                                                  action_cx: fret_ui::action::ActionCx| {
                                                                let _ = host.models_mut().update(
                                                                    &model_for_reset,
                                                                    |v| *v = 1.0,
                                                                );
                                                                host.request_redraw(action_cx.window);
                                                            },
                                                        );

                                                        rows.push(row_cx.row_with(
                                                            cx,
                                                            PropertyRow::new().reset(Some(
                                                                PropertyRowReset::new(on_reset)
                                                                    .options(
                                                                        fret_ui_editor::composites::PropertyRowResetOptions {
                                                                            test_id: Some(Arc::from("imui-editor-proof.editor.drag-value-reset")),
                                                                            ..Default::default()
                                                                        },
                                                                    ),
                                                            )),
                                                            |cx| cx.text("Opacity"),
                                                            |cx| {
                                                                DragValue::new(
                                                                    editor_value_model.clone(),
                                                                    fmt.clone(),
                                                                    parse.clone(),
                                                                )
                                                                .validate(Some(validate.clone()))
                                                                .into_element(cx)
                                                                .test_id("imui-editor-proof.editor.drag-value-demo")
                                                            },
                                                            |cx| {
                                                                Some(
                                                                    FieldStatusBadge::new(
                                                                        FieldStatus::Dirty,
                                                                    )
                                                                    .into_element(cx),
                                                                )
                                                            },
                                                        ));
                                                    }

                                                    if show_roughness {
                                                        let model_for_reset =
                                                            editor_roughness_model.clone();
                                                        let on_reset = Arc::new(
                                                            move |host: &mut dyn fret_ui::action::UiActionHost,
                                                                  action_cx: fret_ui::action::ActionCx| {
                                                                let _ = host.models_mut().update(
                                                                    &model_for_reset,
                                                                    |v| *v = 0.5,
                                                                );
                                                                host.request_redraw(action_cx.window);
                                                            },
                                                        );

                                                        rows.push(row_cx.row_with(
                                                            cx,
                                                            PropertyRow::new().reset(Some(
                                                                PropertyRowReset::new(on_reset)
                                                                    .options(
                                                                        fret_ui_editor::composites::PropertyRowResetOptions {
                                                                            test_id: Some(Arc::from("imui-editor-proof.editor.material.roughness.reset")),
                                                                            ..Default::default()
                                                                        },
                                                                    ),
                                                            )),
                                                            |cx| cx.text("Roughness"),
                                                            |cx| {
                                                                DragValue::new(
                                                                    editor_roughness_model.clone(),
                                                                    fmt.clone(),
                                                                    parse.clone(),
                                                                )
                                                                .validate(Some(validate.clone()))
                                                                .into_element(cx)
                                                                .test_id("imui-editor-proof.editor.material.roughness")
                                                            },
                                                            |cx| {
                                                                Some(
                                                                    FieldStatusBadge::new(
                                                                        FieldStatus::Mixed,
                                                                    )
                                                                    .into_element(cx),
                                                                )
                                                            },
                                                        ));
                                                    }

                                                    if show_metallic {
                                                        let model_for_reset =
                                                            editor_metallic_model.clone();
                                                        let on_reset = Arc::new(
                                                            move |host: &mut dyn fret_ui::action::UiActionHost,
                                                                  action_cx: fret_ui::action::ActionCx| {
                                                                let _ = host.models_mut().update(
                                                                    &model_for_reset,
                                                                    |v| *v = 0.0,
                                                                );
                                                                host.request_redraw(action_cx.window);
                                                            },
                                                        );

                                                        rows.push(row_cx.row_with(
                                                            cx,
                                                            PropertyRow::new().reset(Some(
                                                                PropertyRowReset::new(on_reset)
                                                                    .options(
                                                                        fret_ui_editor::composites::PropertyRowResetOptions {
                                                                            test_id: Some(Arc::from("imui-editor-proof.editor.material.metallic.reset")),
                                                                            ..Default::default()
                                                                        },
                                                                    ),
                                                            )),
                                                            |cx| cx.text("Metallic"),
                                                            |cx| {
                                                                DragValue::new(
                                                                    editor_metallic_model.clone(),
                                                                    fmt.clone(),
                                                                    parse.clone(),
                                                                )
                                                                .validate(Some(validate.clone()))
                                                                .into_element(cx)
                                                                .test_id("imui-editor-proof.editor.material.metallic")
                                                            },
                                                            |cx| {
                                                                Some(
                                                                    FieldStatusBadge::new(
                                                                        FieldStatus::Loading,
                                                                    )
                                                                    .into_element(cx),
                                                                )
                                                            },
                                                        ));
                                                    }

                                                    if show_shading_model {
                                                        let items: Arc<[EnumSelectItem]> = vec![
                                                            EnumSelectItem::new("lit", "Lit"),
                                                            EnumSelectItem::new("unlit", "Unlit"),
                                                            EnumSelectItem::new(
                                                                "subsurface",
                                                                "Subsurface",
                                                            ),
                                                        ]
                                                        .into();

                                                        rows.push(row_cx.row(
                                                            cx,
                                                            |cx| cx.text("Shading model"),
                                                            |cx| {
                                                                EnumSelect::new(
                                                                    editor_shading_model.clone(),
                                                                    items,
                                                                )
                                                                .options(EnumSelectOptions {
                                                                    a11y_label: Some(Arc::from(
                                                                        "Shading model",
                                                                    )),
                                                                    test_id: Some(Arc::from(
                                                                        "imui-editor-proof.editor.material.shading-model",
                                                                    )),
                                                                    list_test_id: Some(Arc::from(
                                                                        "imui-editor-proof.editor.material.shading-model.list",
                                                                    )),
                                                                    search_test_id: Some(Arc::from(
                                                                        "imui-editor-proof.editor.material.shading-model.search",
                                                                    )),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(cx)
                                                            },
                                                        ));
                                                    }

                                                    if show_alpha_clip {
                                                        rows.push(row_cx.row(
                                                            cx,
                                                            |cx| cx.text("Alpha clip"),
                                                            |cx| {
                                                                Checkbox::new(
                                                                    editor_alpha_clip_model.clone(),
                                                                )
                                                                .options(
                                                                    fret_ui_editor::controls::CheckboxOptions {
                                                                        a11y_label: Some(
                                                                            Arc::from("Alpha clip"),
                                                                        ),
                                                                        ..Default::default()
                                                                    },
                                                                )
                                                                .into_element(cx)
                                                                .test_id(
                                                                    "imui-editor-proof.editor.material.alpha-clip",
                                                                )
                                                            },
                                                        ));
                                                    }

                                                    if show_cast_shadows {
                                                        rows.push(row_cx.row(
                                                            cx,
                                                            |cx| cx.text("Cast shadows"),
                                                            |cx| {
                                                                Checkbox::new_optional(
                                                                    editor_cast_shadows_model.clone(),
                                                                )
                                                                .options(
                                                                    fret_ui_editor::controls::CheckboxOptions {
                                                                        a11y_label: Some(Arc::from(
                                                                            "Cast shadows",
                                                                        )),
                                                                        ..Default::default()
                                                                    },
                                                                )
                                                                .into_element(cx)
                                                                .test_id(
                                                                    "imui-editor-proof.editor.material.cast-shadows",
                                                                )
                                                            },
                                                        ));
                                                    }

                                                    if rows.is_empty() {
                                                        rows.push(
                                                            cx.text("No matches").test_id(
                                                                "imui-editor-proof.editor.material.no-matches",
                                                            ),
                                                        );
                                                    }

                                                    rows
                                                },
                                            )]
                                        },
                                    ),
                            );

                            out.push(
                                PropertyGroup::new("Advanced")
                                    .options(fret_ui_editor::composites::PropertyGroupOptions {
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.group.advanced",
                                        )),
                                        header_test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.group.advanced.header",
                                        )),
                                        content_test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.group.advanced.content",
                                        )),
                                        ..Default::default()
                                    })
                                    .into_element(
                                        cx,
                                        |_cx| None,
                                        move |cx| {
                                            let fmt_i32: fret_ui_editor::controls::NumericFormatFn<i32> =
                                                Arc::new(|v| Arc::from(format!("{v}")));
                                            let parse_i32: fret_ui_editor::controls::NumericParseFn<i32> =
                                                Arc::new(|s| s.trim().parse::<i32>().ok());

                                            vec![PropertyGrid::new().into_element(
                                                cx,
                                                move |cx, row_cx| {
                                                    let mut rows = Vec::new();

                                                    if show_iterations {
                                                        let model_for_reset =
                                                            editor_iterations_model.clone();
                                                        let on_reset = Arc::new(
                                                            move |host: &mut dyn fret_ui::action::UiActionHost,
                                                                  action_cx: fret_ui::action::ActionCx| {
                                                                let _ = host.models_mut().update(
                                                                    &model_for_reset,
                                                                    |v| *v = 8,
                                                                );
                                                                host.request_redraw(action_cx.window);
                                                            },
                                                        );

                                                        rows.push(
                                                            PropertyRow::new()
                                                                .options(row_cx.row_options.clone())
                                                                .reset(Some(
                                                                    PropertyRowReset::new(on_reset).options(
                                                                        fret_ui_editor::composites::PropertyRowResetOptions {
                                                                            test_id: Some(Arc::from("imui-editor-proof.editor.advanced.iterations.reset")),
                                                                            ..Default::default()
                                                                        },
                                                                    ),
                                                                ))
                                                                .into_element(
                                                                    cx,
                                                                    |cx| cx.text("Iterations"),
                                                                    |cx| {
                                                                        DragValue::new(
                                                                            editor_iterations_model.clone(),
                                                                            fmt_i32.clone(),
                                                                            parse_i32.clone(),
                                                                        )
                                                                        .into_element(cx)
                                                                        .test_id("imui-editor-proof.editor.advanced.iterations")
                                                                    },
                                                                    |cx| {
                                                                        Some(
                                                                            FieldStatusBadge::new(FieldStatus::Error(
                                                                                Arc::from("stub"),
                                                                            ))
                                                                            .into_element(cx),
                                                                        )
                                                                    },
                                                                ),
                                                        );
                                                    }

                                                    if rows.is_empty() {
                                                        rows.push(
                                                            cx.text("No matches").test_id(
                                                                "imui-editor-proof.editor.advanced.no-matches",
                                                            ),
                                                        );
                                                    }

                                                    rows
                                                },
                                            )]
                                        },
                                    ),
                            );

                            if !q.is_empty() && !any_match {
                                out.push(
                                    cx.text("No matches")
                                        .test_id("imui-editor-proof.editor.no-matches"),
                                );
                            }

                            out
                        },
                    )]
                });
                ui.separator();

                fret_docking::imui::dock_space_with(
                    ui,
                    fret_docking::imui::DockSpaceImUiOptions {
                        test_id: dock_test_id,
                        tab_drag_anchor_test_id,
                        ..Default::default()
                    },
                    move |app, window| ensure_dock_graph(app, window),
                );
            });
        })
        .size_full();
        ui.add_ui(root);
    })
}

fn editor_demo_value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    let model = cx.with_state(|| None::<Model<f64>>, |st| st.clone());
    match model {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(0.8_f64);
            cx.with_state(
                || None::<Model<f64>>,
                |st| {
                    if st.is_none() {
                        *st = Some(model.clone());
                    }
                },
            );
            model
        }
    }
}

fn editor_demo_roughness_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    let model = cx.with_state(|| None::<Model<f64>>, |st| st.clone());
    match model {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(0.35_f64);
            cx.with_state(
                || None::<Model<f64>>,
                |st| {
                    if st.is_none() {
                        *st = Some(model.clone());
                    }
                },
            );
            model
        }
    }
}

fn editor_demo_metallic_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    let model = cx.with_state(|| None::<Model<f64>>, |st| st.clone());
    match model {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(0.1_f64);
            cx.with_state(
                || None::<Model<f64>>,
                |st| {
                    if st.is_none() {
                        *st = Some(model.clone());
                    }
                },
            );
            model
        }
    }
}

fn editor_demo_alpha_clip_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let model = cx.with_state(|| None::<Model<bool>>, |st| st.clone());
    match model {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(
                || None::<Model<bool>>,
                |st| {
                    if st.is_none() {
                        *st = Some(model.clone());
                    }
                },
            );
            model
        }
    }
}

fn editor_demo_cast_shadows_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<bool>> {
    let model = cx.with_state(|| None::<Model<Option<bool>>>, |st| st.clone());
    match model {
        Some(model) => model,
        None => {
            // Start in "mixed/indeterminate" to exercise tri-state checkbox rendering.
            let model = cx.app.models_mut().insert(None::<bool>);
            cx.with_state(
                || None::<Model<Option<bool>>>,
                |st| {
                    if st.is_none() {
                        *st = Some(model.clone());
                    }
                },
            );
            model
        }
    }
}

fn editor_demo_shading_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    let model = cx.with_state(|| None::<Model<Option<Arc<str>>>>, |st| st.clone());
    match model {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(Some::<Arc<str>>(Arc::from("lit")));
            cx.with_state(
                || None::<Model<Option<Arc<str>>>>,
                |st| {
                    if st.is_none() {
                        *st = Some(model.clone());
                    }
                },
            );
            model
        }
    }
}

fn editor_demo_iterations_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<i32> {
    let model = cx.with_state(|| None::<Model<i32>>, |st| st.clone());
    match model {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(16_i32);
            cx.with_state(
                || None::<Model<i32>>,
                |st| {
                    if st.is_none() {
                        *st = Some(model.clone());
                    }
                },
            );
            model
        }
    }
}

fn editor_demo_search_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let model = cx.with_state(|| None::<Model<String>>, |st| st.clone());
    match model {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(
                || None::<Model<String>>,
                |st| {
                    if st.is_none() {
                        *st = Some(model.clone());
                    }
                },
            );
            model
        }
    }
}

fn install_dock_panel_registry(app: &mut App) {
    let registry: Arc<dyn DockPanelRegistry<App>> = Arc::new(ImUiEditorProofPanelRegistry);
    app.with_global_mut(DockPanelRegistryService::<App>::default, |svc, _app| {
        svc.set(registry);
    });
}

struct ImUiEditorProofPanelRegistry;

impl DockPanelRegistry<App> for ImUiEditorProofPanelRegistry {
    fn render_panel(
        &self,
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: fret_core::Rect,
        panel: &fret_core::PanelKey,
    ) -> Option<fret_core::NodeId> {
        if panel.kind.0 != "demo.controls" {
            return None;
        }

        let root_name = match panel.instance.as_deref() {
            Some(instance) => format!("imui_editor_proof.panel.{}:{}", panel.kind.0, instance),
            None => format!("imui_editor_proof.panel.{}", panel.kind.0),
        };
        let panel_key = panel.clone();
        Some(fret_docking::render_cached_panel_root(
            ui,
            app,
            services,
            window,
            bounds,
            &root_name,
            move |cx| {
                let target = embedded::models(&*cx.app, window)
                    .and_then(|m| cx.watch_model(&m.target).paint().copied())
                    .unwrap_or_default();

                vec![
                    fret_ui_kit::ui::container_build(cx, move |cx, out| {
                        out.extend(
                            fret_imui::imui_vstack(cx, move |ui| {
                                use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;

                                ui.id(&panel_key, |ui| {
                                    ui.text("Controls panel (declarative root inside docking)");
                                    ui.text(format!("embedded viewport target: {target:?}"));
                                    ui.text(
                                        "Wasm/mobile note: multi-window should degrade to in-window floatings.",
                                    );
                                });
                            })
                            .into_vec(),
                        );
                    })
                    .size_full()
                    .p_3()
                    .bg(fret_ui_kit::ColorRef::Token {
                        key: "background",
                        fallback: fret_ui_kit::ColorFallback::ThemeSurfaceBackground,
                    })
                    .into_element(cx),
                ]
            },
        ))
    }
}

fn ensure_dock_graph(app: &mut App, window: AppWindowId) {
    ensure_dock_graph_inner(app, window, false);
}

fn reset_dock_graph(app: &mut App, window: AppWindowId) {
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.graph.remove_window_root(window);
        dock.graph.floating_windows_mut(window).clear();
    });
    ensure_dock_graph_inner(app, window, true);
}

fn ensure_dock_graph_inner(app: &mut App, window: AppWindowId, force: bool) {
    app.with_global_mut(DockManager::default, |dock, app| {
        let logical_window_id = app
            .global::<WindowBootstrapService>()
            .and_then(|svc| svc.logical_by_window.get(&window).cloned())
            .unwrap_or_else(|| format!("{window:?}"));

        let viewport_panel =
            fret_core::PanelKey::with_instance("demo.viewport", logical_window_id.clone());
        let controls_panel = fret_core::PanelKey::with_instance("demo.controls", logical_window_id);

        let target = embedded::models(app, window)
            .and_then(|m| app.models().read(&m.target, |v| *v).ok())
            .unwrap_or_default();

        dock.ensure_panel(&viewport_panel, || DockPanel {
            title: "Viewport".to_string(),
            color: Color::TRANSPARENT,
            viewport: None,
        });
        dock.ensure_panel(&controls_panel, || DockPanel {
            title: "Controls".to_string(),
            color: Color::TRANSPARENT,
            viewport: None,
        });

        if let Some(panel) = dock.panels.get_mut(&viewport_panel) {
            panel.viewport = if target == fret_core::RenderTargetId::default() {
                None
            } else {
                Some(ViewportPanel {
                    target,
                    target_px_size: VIEWPORT_PX_SIZE,
                    fit: fret_core::ViewportFit::Stretch,
                    context_menu_enabled: true,
                })
            };
        }

        if !force && dock.graph.window_root(window).is_some() {
            return;
        }

        use fret_core::{Axis, DockFloatingWindow, DockNode, Point, Px, Rect, Size};

        if single_window_mode_enabled() {
            // In single-window mode we want the "floating window" affordance to be immediately
            // visible without requiring the user to discover the float zone gesture.
            let tabs_viewport = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![viewport_panel],
                active: 0,
            });
            dock.graph.set_window_root(window, tabs_viewport);

            let tabs_controls = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![controls_panel],
                active: 0,
            });
            let floating = dock.graph.insert_node(DockNode::Floating {
                child: tabs_controls,
            });
            dock.graph
                .floating_windows_mut(window)
                .push(DockFloatingWindow {
                    floating,
                    rect: Rect::new(
                        Point::new(Px(24.0), Px(48.0)),
                        Size::new(Px(420.0), Px(240.0)),
                    ),
                });
        } else {
            let tabs_viewport = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![viewport_panel],
                active: 0,
            });
            let tabs_controls = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![controls_panel],
                active: 0,
            });
            let root = dock.graph.insert_node(DockNode::Split {
                axis: Axis::Vertical,
                children: vec![tabs_viewport, tabs_controls],
                fractions: vec![0.7, 0.3],
            });
            dock.graph.set_window_root(window, root);
        }

        dock_runtime::request_dock_invalidation(app, [window]);
    });
}

#[derive(Default)]
struct WindowBootstrapService {
    main_window: Option<AppWindowId>,
    aux_requested: bool,
    logical_by_window: HashMap<AppWindowId, String>,
}

fn ensure_aux_window_requested(app: &mut App, window: AppWindowId) {
    app.with_global_mut(WindowBootstrapService::default, |svc, app| {
        if svc.main_window.is_none() {
            svc.main_window = Some(window);
            svc.logical_by_window.insert(window, "main".to_string());
        }
        if svc.aux_requested {
            return;
        }
        if svc.main_window != Some(window) {
            return;
        }

        svc.aux_requested = true;
        let anchor = diag_enabled().then_some(fret_core::WindowAnchor {
            window,
            position: fret_core::Point::new(fret_core::Px(120.0), fret_core::Px(24.0)),
        });
        app.push_effect(Effect::Window(WindowRequest::Create(CreateWindowRequest {
            kind: CreateWindowKind::DockRestore {
                logical_window_id: AUX_LOGICAL_WINDOW_ID.to_string(),
            },
            anchor,
            role: WindowRole::Auxiliary,
            style: WindowStyleRequest {
                activation: diag_enabled().then_some(ActivationPolicy::NonActivating),
                ..Default::default()
            },
        })));
    });
}

fn on_dock_op(app: &mut App, op: fret_core::DockOp) {
    let _ = dock_runtime::handle_dock_op(app, op);
}

fn window_create_spec(
    _app: &mut App,
    request: &fret_app::CreateWindowRequest,
) -> Option<fret_launch::WindowCreateSpec> {
    match &request.kind {
        CreateWindowKind::DockFloating { panel, .. } => Some(fret_launch::WindowCreateSpec::new(
            format!("fret-demo imui_editor_proof_demo — {}", panel.kind.0),
            winit::dpi::LogicalSize::new(720.0, 520.0),
        )),
        CreateWindowKind::DockRestore { logical_window_id } => {
            Some(fret_launch::WindowCreateSpec::new(
                format!("fret-demo imui_editor_proof_demo — {logical_window_id}"),
                winit::dpi::LogicalSize::new(980.0, 720.0),
            ))
        }
    }
}

fn window_created(app: &mut App, request: &fret_app::CreateWindowRequest, new_window: AppWindowId) {
    if let CreateWindowKind::DockRestore { logical_window_id } = &request.kind {
        app.with_global_mut(WindowBootstrapService::default, |svc, _app| {
            svc.logical_by_window
                .insert(new_window, logical_window_id.clone());
        });
        if diag_enabled() && logical_window_id == AUX_LOGICAL_WINDOW_ID {
            let sender = app
                .global::<WindowBootstrapService>()
                .and_then(|svc| svc.main_window);
            app.push_effect(Effect::Window(WindowRequest::Raise {
                window: new_window,
                sender,
            }));
        }
        if diag_enabled() {
            app.request_redraw(new_window);
            app.push_effect(Effect::RequestAnimationFrame(new_window));
        }
    }
    let _ = dock_runtime::handle_dock_window_created(app, request, new_window);
}

fn before_close_window(app: &mut App, closing_window: AppWindowId) -> bool {
    let target_window = app
        .global::<WindowBootstrapService>()
        .and_then(|svc| svc.main_window)
        .unwrap_or(closing_window);
    let _ = dock_runtime::handle_dock_before_close_window(app, closing_window, target_window);
    true
}
