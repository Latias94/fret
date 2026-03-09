use std::collections::HashMap;
use std::sync::Arc;

use fret::interop::embedded_viewport as embedded;
use fret::prelude::*;
use fret_app::{CreateWindowKind, CreateWindowRequest, WindowRequest};
use fret_core::{Color, Px};
use fret_docking::{
    runtime as dock_runtime, DockManager, DockPanel, DockPanelRegistry, DockPanelRegistryService,
    ViewportPanel,
};
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::{
    ActivationPolicy, FrameId, Model, PlatformCapabilities, TickId, WindowHoverDetectionQuality,
    WindowRole, WindowStyleRequest,
};
use fret_ui_editor::composites::{
    GradientEditor, GradientEditorOptions, GradientStopBinding, InspectorPanel,
    InspectorPanelOptions, PropertyGrid, PropertyGroup, PropertyRow, PropertyRowReset,
};
use fret_ui_editor::controls::{
    Checkbox, ColorEdit, ColorEditOptions, DragValue, EnumSelect, EnumSelectItem,
    EnumSelectOptions, FieldStatus, FieldStatusBadge, NumericFormatFn, NumericParseFn,
    NumericValidateFn, Slider, SliderOptions, TextField, TextFieldOptions, TransformEdit,
    TransformEditOptions, Vec3Edit,
};
use fret_ui_editor::primitives::{percent_0_1_format, percent_0_1_parse};

const VIEWPORT_PX_SIZE: (u32, u32) = (960, 540);
const AUX_LOGICAL_WINDOW_ID: &str = "aux";
const ENV_SINGLE_WINDOW: &str = "FRET_IMUI_EDITOR_PROOF_SINGLE_WINDOW";

fn diag_enabled() -> bool {
    std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty() && v != "0")
}

struct ImUiEditorProofView {
    embedded: embedded::EmbeddedViewportSurface,
}

impl embedded::EmbeddedViewportView for ImUiEditorProofView {
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
    FretApp::new("imui-editor-proof-demo")
        .window("imui_editor_proof_demo", (1120.0, 720.0))
        .view_with_hooks::<ImUiEditorProofView>(|d| {
            d.drive_embedded_viewport()
                .dock_op(on_dock_op)
                .window_create_spec(window_create_spec)
                .window_created(window_created)
                .before_close_window(before_close_window)
        })?
        .init_app(|app| {
            configure_single_window_caps_if_requested(app);
            shadcn::shadcn_themes::apply_shadcn_new_york(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            );
            fret_ui_editor::theme::apply_editor_theme_patch_v1(app);
            fret_icons_lucide::install_app(app);
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

impl View for ImUiEditorProofView {
    fn init(app: &mut App, window: AppWindowId) -> Self {
        embedded::ensure_models(app, window);
        if !single_window_mode_enabled() {
            ensure_aux_window_requested(app, window);
        }

        Self {
            embedded: embedded::EmbeddedViewportSurface::new(
                wgpu::TextureFormat::Bgra8UnormSrgb,
                RenderTargetColorSpace::Srgb,
                VIEWPORT_PX_SIZE,
            ),
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        render_view(cx.elements())
    }
}

fn render_view(cx: &mut ElementContext<'_, App>) -> ViewElements {
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
    let editor_base_color_model = editor_demo_base_color_model(cx);
    let editor_name_model = editor_demo_name_model(cx);
    let editor_notes_model = editor_demo_notes_model(cx);
    let (editor_pos_x, editor_pos_y, editor_pos_z) = editor_demo_position_models(cx);
    let (editor_rot_x, editor_rot_y, editor_rot_z) = editor_demo_rotation_models(cx);
    let (editor_scl_x, editor_scl_y, editor_scl_z) = editor_demo_scale_models(cx);
    let editor_iterations_model = editor_demo_iterations_model(cx);
    let editor_search_model = editor_demo_search_model(cx);
    let editor_gradient_angle_model = editor_demo_gradient_angle_model(cx);
    let editor_gradient_stops_model = editor_demo_gradient_stops_model(cx);
    let editor_gradient_next_id_model = editor_demo_gradient_next_id_model(cx);

    #[cfg(debug_assertions)]
    {
        debug_assert_ne!(
            editor_roughness_model.id(),
            editor_metallic_model.id(),
            "Roughness/Metallic models must be distinct; otherwise sliders will sync unintentionally."
        );
    }

    fret_imui::imui(cx, |ui| {
        use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
        use fret_ui_kit::imui::UiWriterUiKitExt as _;

        let root = fret_ui_kit::ui::v_flex_build(move |cx, out| {
            fret_imui::imui_build(cx, out, |ui| {
                let headline = fret_ui_kit::ui::text(format!(
                        "imui editor-grade proof (M7): docking + multi-window + viewport surfaces (window={window:?})"
                    ),
                )
                .font_semibold();
                ui.add_ui(headline);

                if single {
                    let hint = fret_ui_kit::ui::text(format!(
                            "single-window mode enabled ({ENV_SINGLE_WINDOW}=1): dock tear-off should degrade to in-window floating"
                        ),
                    )
                    .text_xs();
                    ui.add_ui(hint);
                }

                let caps_line = fret_ui_kit::ui::text(format!(
                        "caps: multi_window={} window_tear_off={} window_hover_detection={:?} window_inner_size={window_size:?}",
                        caps.ui.multi_window, caps.ui.window_tear_off, caps.ui.window_hover_detection,
                    ),
                )
                .text_xs();
                ui.add_ui(caps_line);

                let controls = fret_ui_kit::ui::h_flex_build(move |cx, out| {
                    fret_imui::imui_build(cx, out, |ui| {
                        if <_ as fret_ui_kit::imui::UiWriterImUiFacadeExt<App>>::button(
                            ui,
                            "Reset layout",
                        )
                        .clicked()
                        {
                            reset_dock_graph(ui.cx_mut().app, window);
                            dock_runtime::request_dock_invalidation(ui.cx_mut().app, [window]);
                        }
                        if <_ as fret_ui_kit::imui::UiWriterImUiFacadeExt<App>>::button(
                            ui,
                            "Center floatings",
                        )
                        .clicked()
                        {
                            dock_runtime::recenter_in_window_floatings(ui.cx_mut().app, window);
                        }
                    });
                })
                .gap(fret_ui_kit::Space::N2);
                ui.add_ui(controls);

                let last_input_line =
                    fret_ui_kit::ui::text(format!("last_input: {last_input}")).text_xs();
                ui.add_ui(last_input_line);
                ui.separator();

                let editor_label =
                    fret_ui_kit::ui::text("fret-ui-editor (M2): PropertyGroup + PropertyGrid + MiniSearchBox (filter)")
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

                    vec![InspectorPanel::new(Some(editor_search_model.clone()))
                        .options(InspectorPanelOptions {
                            test_id: Some(Arc::from("imui-editor-proof.editor.inspector")),
                            header_test_id: Some(Arc::from(
                                "imui-editor-proof.editor.inspector.header",
                            )),
                            search_test_id: Some(Arc::from("imui-editor-proof.editor.search")),
                            search_clear_test_id: Some(Arc::from(
                                "imui-editor-proof.editor.search.clear",
                            )),
                            content_test_id: Some(Arc::from(
                                "imui-editor-proof.editor.inspector.content",
                            )),
                            ..Default::default()
                        })
                        .into_element(
                            cx,
                            |_cx, _panel_cx| Vec::new(),
                            move |cx, panel_cx| {
                                let matches = |s: &str| panel_cx.matches(s);

                                let material_show_all = matches("material");
                                let show_opacity = material_show_all || matches("opacity");
                                let show_roughness = material_show_all || matches("roughness");
                                let show_metallic = material_show_all || matches("metallic");
                                let show_base_color =
                                    material_show_all || matches("base") || matches("color");
                                let show_shading_model =
                                    material_show_all || matches("shading") || matches("model");
                                let show_alpha_clip =
                                    material_show_all || matches("alpha") || matches("clip");
                                let show_cast_shadows =
                                    material_show_all || matches("shadow") || matches("shadows");

                                let advanced_show_all = matches("advanced");
                                let show_iterations = advanced_show_all || matches("iterations");
                                let show_position =
                                    advanced_show_all || matches("position") || matches("pos");
                                let show_transform = advanced_show_all
                                    || matches("transform")
                                    || matches("xform")
                                    || matches("rotation")
                                    || matches("rot")
                                    || matches("scale");

                                let any_match = show_opacity
                                    || show_roughness
                                    || show_metallic
                                    || show_base_color
                                    || show_shading_model
                                    || show_alpha_clip
                                    || show_cast_shadows
                                    || show_iterations
                                    || show_position
                                    || show_transform;

                                let mut out = Vec::new();

                            out.push(
                                PropertyGroup::new("Object")
                                    .options(fret_ui_editor::composites::PropertyGroupOptions {
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.group.object",
                                        )),
                                        header_test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.group.object.header",
                                        )),
                                        content_test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.group.object.content",
                                        )),
                                        ..Default::default()
                                    })
                                    .into_element(
                                        cx,
                                        |_cx| None,
                                        move |cx| {
                                            vec![PropertyGrid::new().into_element(
                                                cx,
                                                move |cx, row_cx| {
                                                    let mut rows = Vec::new();

                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Name"),
                                                        |cx| {
                                                            TextField::new(
                                                                editor_name_model.clone(),
                                                            )
                                                            .options(TextFieldOptions {
                                                                placeholder: Some(Arc::from(
                                                                    "Untitled",
                                                                )),
                                                                clear_button: true,
                                                                test_id: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.name",
                                                                )),
                                                                clear_test_id: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.name.clear",
                                                                )),
                                                                ..Default::default()
                                                            })
                                                            .into_element(cx)
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Notes"),
                                                        |cx| {
                                                            TextField::new(
                                                                editor_notes_model.clone(),
                                                            )
                                                            .options(TextFieldOptions {
                                                                multiline: true,
                                                                min_height: Some(Px(96.0)),
                                                                clear_button: true,
                                                                test_id: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.notes",
                                                                )),
                                                                clear_test_id: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.notes.clear",
                                                                )),
                                                                ..Default::default()
                                                            })
                                                            .into_element(cx)
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    rows
                                                },
                                            )]
                                        },
                                    ),
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
                                                        let roughness_fmt: NumericFormatFn<f64> =
                                                            percent_0_1_format(0);
                                                        let roughness_parse: NumericParseFn<f64> =
                                                            percent_0_1_parse();

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
                                                                Slider::new(
                                                                    editor_roughness_model.clone(),
                                                                    0.0,
                                                                    1.0,
                                                                )
                                                                .format(roughness_fmt.clone())
                                                                .parse(roughness_parse.clone())
                                                                .options(SliderOptions {
                                                                    a11y_label: Some(Arc::from(
                                                                        "Roughness",
                                                                    )),
                                                                    step: Some(0.01),
                                                                    test_id: Some(Arc::from(
                                                                        "imui-editor-proof.editor.material.roughness",
                                                                    )),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(cx)
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
                                                        let metallic_fmt: NumericFormatFn<f64> =
                                                            percent_0_1_format(0);
                                                        let metallic_parse: NumericParseFn<f64> =
                                                            percent_0_1_parse();

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
                                                                Slider::new(
                                                                    editor_metallic_model.clone(),
                                                                    0.0,
                                                                    1.0,
                                                                )
                                                                .format(metallic_fmt.clone())
                                                                .parse(metallic_parse.clone())
                                                                .options(SliderOptions {
                                                                    a11y_label: Some(Arc::from(
                                                                        "Metallic",
                                                                    )),
                                                                    step: Some(0.01),
                                                                    test_id: Some(Arc::from(
                                                                        "imui-editor-proof.editor.material.metallic",
                                                                    )),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(cx)
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

                                                    if show_base_color {
                                                        rows.push(row_cx.row(
                                                            cx,
                                                            |cx| cx.text("Base color"),
                                                            |cx| {
                                                                ColorEdit::new(
                                                                    editor_base_color_model
                                                                        .clone(),
                                                                )
                                                                .options(ColorEditOptions {
                                                                    test_id: Some(Arc::from(
                                                                        "imui-editor-proof.editor.material.base-color",
                                                                    )),
                                                                    swatch_test_id: Some(
                                                                        Arc::from("imui-editor-proof.editor.material.base-color.swatch"),
                                                                    ),
                                                                    input_test_id: Some(
                                                                        Arc::from("imui-editor-proof.editor.material.base-color.hex"),
                                                                    ),
                                                                    popup_test_id: Some(
                                                                        Arc::from("imui-editor-proof.editor.material.base-color.popup"),
                                                                    ),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(cx)
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
                                PropertyGroup::new("Gradient")
                                    .options(fret_ui_editor::composites::PropertyGroupOptions {
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.group.gradient",
                                        )),
                                        header_test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.group.gradient.header",
                                        )),
                                        content_test_id: Some(Arc::from(
                                            "imui-editor-proof.editor.group.gradient.content",
                                        )),
                                        ..Default::default()
                                    })
                                    .into_element(
                                        cx,
                                        |_cx| None,
                                        move |cx| {
                                            let stops: Vec<GradientDemoStop> = cx
                                                .watch_model(&editor_gradient_stops_model)
                                                .paint()
                                                .cloned()
                                                .unwrap_or_default();

                                            let on_remove: fret_ui_editor::composites::OnGradientStopAction =
                                                Arc::new({
                                                    let stops_model = editor_gradient_stops_model.clone();
                                                    move |host, action_cx, stop_id| {
                                                        let _ = host.models_mut().update(
                                                            &stops_model,
                                                            |stops| stops.retain(|s| s.id != stop_id),
                                                        );
                                                        host.request_redraw(action_cx.window);
                                                    }
                                                });

                                            let on_add: fret_ui_editor::composites::OnGradientAction =
                                                Arc::new({
                                                    let stops_model = editor_gradient_stops_model.clone();
                                                    let next_id_model = editor_gradient_next_id_model.clone();
                                                    move |host, action_cx| {
                                                        let id = host
                                                            .models_mut()
                                                            .update(&next_id_model, |v| {
                                                                let out = *v;
                                                                *v = v.saturating_add(1);
                                                                out
                                                            })
                                                            .unwrap_or(1);

                                                        let position = host.models_mut().insert(0.5_f64);
                                                        let color = host.models_mut().insert(Color {
                                                            r: 0.85,
                                                            g: 0.85,
                                                            b: 0.85,
                                                            a: 1.0,
                                                        });
                                                        let stop = GradientDemoStop {
                                                            id,
                                                            position,
                                                            color,
                                                        };

                                                        let _ = host
                                                            .models_mut()
                                                            .update(&stops_model, |stops| stops.push(stop));
                                                        host.request_redraw(action_cx.window);
                                                    }
                                                });

                                            let bindings: Arc<[GradientStopBinding]> = stops
                                                .into_iter()
                                                .map(|s| GradientStopBinding {
                                                    id: s.id,
                                                    position: s.position,
                                                    color: s.color,
                                                    remove: Some(on_remove.clone()),
                                                })
                                                .collect::<Vec<_>>()
                                                .into();

                                            vec![GradientEditor::new(bindings)
                                                .angle_degrees(Some(
                                                    editor_gradient_angle_model.clone(),
                                                ))
                                                .on_add_stop(Some(on_add))
                                                .options(GradientEditorOptions {
                                                    id_source: Some(Arc::from(
                                                        "imui_editor_proof_demo.gradient",
                                                    )),
                                                    test_id: Some(Arc::from(
                                                        "imui-editor-proof.editor.gradient",
                                                    )),
                                                    preview_test_id: Some(Arc::from(
                                                        "imui-editor-proof.editor.gradient.preview",
                                                    )),
                                                    stops_test_id: Some(Arc::from(
                                                        "imui-editor-proof.editor.gradient.stops",
                                                    )),
                                                    add_stop_test_id: Some(Arc::from(
                                                        "imui-editor-proof.editor.gradient.add-stop",
                                                    )),
                                                    ..Default::default()
                                                })
                                                .into_element(cx)]
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
                                            let fmt_f64: fret_ui_editor::controls::NumericFormatFn<f64> =
                                                Arc::new(|v| Arc::from(format!("{v:.3}")));
                                            let parse_f64: fret_ui_editor::controls::NumericParseFn<f64> =
                                                Arc::new(|s| s.trim().parse::<f64>().ok());
                                            let fmt_i32: fret_ui_editor::controls::NumericFormatFn<i32> =
                                                Arc::new(|v| Arc::from(format!("{v}")));
                                            let parse_i32: fret_ui_editor::controls::NumericParseFn<i32> =
                                                Arc::new(|s| s.trim().parse::<i32>().ok());

                                            vec![PropertyGrid::new().into_element(
                                                cx,
                                                move |cx, row_cx| {
                                                    let mut rows = Vec::new();

                                                    if show_position {
                                                        let x_for_reset = editor_pos_x.clone();
                                                        let y_for_reset = editor_pos_y.clone();
                                                        let z_for_reset = editor_pos_z.clone();
                                                        let on_reset = Arc::new(
                                                            move |host: &mut dyn fret_ui::action::UiActionHost,
                                                                  action_cx: fret_ui::action::ActionCx| {
                                                                let _ = host
                                                                    .models_mut()
                                                                    .update(&x_for_reset, |v| *v = 0.0);
                                                                let _ = host
                                                                    .models_mut()
                                                                    .update(&y_for_reset, |v| *v = 0.0);
                                                                let _ = host
                                                                    .models_mut()
                                                                    .update(&z_for_reset, |v| *v = 0.0);
                                                                host.request_redraw(action_cx.window);
                                                            },
                                                        );

                                                        rows.push(row_cx.row_with(
                                                            cx,
                                                            PropertyRow::new().reset(Some(
                                                                PropertyRowReset::new(on_reset)
                                                                    .options(
                                                                        fret_ui_editor::composites::PropertyRowResetOptions {
                                                                            test_id: Some(Arc::from("imui-editor-proof.editor.advanced.position.reset")),
                                                                            ..Default::default()
                                                                        },
                                                                    ),
                                                            )),
                                                            |cx| cx.text("Position"),
                                                            |cx| {
                                                                Vec3Edit::new(
                                                                    editor_pos_x.clone(),
                                                                    editor_pos_y.clone(),
                                                                    editor_pos_z.clone(),
                                                                    fmt_f64.clone(),
                                                                    parse_f64.clone(),
                                                                )
                                                                .into_element(cx)
                                                                .test_id(
                                                                    "imui-editor-proof.editor.advanced.position",
                                                                )
                                                            },
                                                            |_cx| None,
                                                        ));
                                                    }

                                                    if show_transform {
                                                        let pos_x = editor_pos_x.clone();
                                                        let pos_y = editor_pos_y.clone();
                                                        let pos_z = editor_pos_z.clone();
                                                        let rot_x = editor_rot_x.clone();
                                                        let rot_y = editor_rot_y.clone();
                                                        let rot_z = editor_rot_z.clone();
                                                        let scl_x = editor_scl_x.clone();
                                                        let scl_y = editor_scl_y.clone();
                                                        let scl_z = editor_scl_z.clone();

                                                        let on_reset = Arc::new(
                                                            move |host: &mut dyn fret_ui::action::UiActionHost,
                                                                  action_cx: fret_ui::action::ActionCx| {
                                                                let _ = host
                                                                    .models_mut()
                                                                    .update(&pos_x, |v| *v = 0.0);
                                                                let _ = host
                                                                    .models_mut()
                                                                    .update(&pos_y, |v| *v = 0.0);
                                                                let _ = host
                                                                    .models_mut()
                                                                    .update(&pos_z, |v| *v = 0.0);
                                                                let _ = host
                                                                    .models_mut()
                                                                    .update(&rot_x, |v| *v = 0.0);
                                                                let _ = host
                                                                    .models_mut()
                                                                    .update(&rot_y, |v| *v = 0.0);
                                                                let _ = host
                                                                    .models_mut()
                                                                    .update(&rot_z, |v| *v = 0.0);
                                                                let _ = host
                                                                    .models_mut()
                                                                    .update(&scl_x, |v| *v = 1.0);
                                                                let _ = host
                                                                    .models_mut()
                                                                    .update(&scl_y, |v| *v = 1.0);
                                                                let _ = host
                                                                    .models_mut()
                                                                    .update(&scl_z, |v| *v = 1.0);
                                                                host.request_redraw(action_cx.window);
                                                            },
                                                        );

                                                        rows.push(row_cx.row_with(
                                                            cx,
                                                            PropertyRow::new()
                                                                .options(row_cx.row_options.clone())
                                                                .reset(Some(
                                                                    PropertyRowReset::new(on_reset)
                                                                        .options(
                                                                            fret_ui_editor::composites::PropertyRowResetOptions {
                                                                                test_id: Some(Arc::from("imui-editor-proof.editor.advanced.transform.reset")),
                                                                                ..Default::default()
                                                                            },
                                                                        ),
                                                                )),
                                                            |cx| cx.text("Transform"),
                                                            |cx| {
                                                                TransformEdit::new(
                                                                    (
                                                                        editor_pos_x.clone(),
                                                                        editor_pos_y.clone(),
                                                                        editor_pos_z.clone(),
                                                                    ),
                                                                    (
                                                                        editor_rot_x.clone(),
                                                                        editor_rot_y.clone(),
                                                                        editor_rot_z.clone(),
                                                                    ),
                                                                    (
                                                                        editor_scl_x.clone(),
                                                                        editor_scl_y.clone(),
                                                                        editor_scl_z.clone(),
                                                                    ),
                                                                    fmt_f64.clone(),
                                                                    parse_f64.clone(),
                                                                )
                                                                .options(TransformEditOptions {
                                                                    test_id: Some(Arc::from("imui-editor-proof.editor.advanced.transform")),
                                                                    link_test_id: Some(Arc::from("imui-editor-proof.editor.advanced.transform.link-scale")),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(cx)
                                                            },
                                                            |_cx| None,
                                                        ));
                                                    }

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

                            if !panel_cx.is_query_empty() && !any_match {
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

fn named_demo_state<H: UiHost, S: Clone + 'static>(
    cx: &mut ElementContext<'_, H>,
    name: &'static str,
    init: impl FnOnce(&mut ElementContext<'_, H>) -> S,
) -> S {
    cx.named(name, |cx| {
        let existing = cx.with_state(|| None::<S>, |st| st.clone());
        match existing {
            Some(v) => v,
            None => {
                let v = init(cx);
                cx.with_state(
                    || None::<S>,
                    |st| {
                        if st.is_none() {
                            *st = Some(v.clone());
                        }
                    },
                );
                v
            }
        }
    })
}

fn editor_demo_value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(cx, "imui_editor_proof_demo.model.value", |cx| {
        cx.app.models_mut().insert(0.8_f64)
    })
}

fn editor_demo_roughness_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(cx, "imui_editor_proof_demo.model.roughness", |cx| {
        cx.app.models_mut().insert(0.35_f64)
    })
}

fn editor_demo_metallic_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(cx, "imui_editor_proof_demo.model.metallic", |cx| {
        cx.app.models_mut().insert(0.1_f64)
    })
}

#[derive(Clone)]
struct GradientDemoStop {
    id: fret_ui::ItemKey,
    position: Model<f64>,
    color: Model<Color>,
}

fn editor_demo_gradient_angle_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(cx, "imui_editor_proof_demo.model.gradient_angle", |cx| {
        cx.app.models_mut().insert(45.0_f64)
    })
}

fn editor_demo_gradient_stops_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Vec<GradientDemoStop>> {
    named_demo_state(cx, "imui_editor_proof_demo.model.gradient_stops", |cx| {
        let stop_0_pos = cx.app.models_mut().insert(0.0_f64);
        let stop_0_color = cx.app.models_mut().insert(Color {
            a: 1.0,
            ..Color::from_srgb_hex_rgb(0xf2_59_33)
        });
        let stop_1_pos = cx.app.models_mut().insert(1.0_f64);
        let stop_1_color = cx.app.models_mut().insert(Color {
            a: 1.0,
            ..Color::from_srgb_hex_rgb(0x33_73_f2)
        });
        cx.app.models_mut().insert(vec![
            GradientDemoStop {
                id: 1,
                position: stop_0_pos,
                color: stop_0_color,
            },
            GradientDemoStop {
                id: 2,
                position: stop_1_pos,
                color: stop_1_color,
            },
        ])
    })
}

fn editor_demo_gradient_next_id_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<u64> {
    named_demo_state(cx, "imui_editor_proof_demo.model.gradient_next_id", |cx| {
        cx.app.models_mut().insert(3_u64)
    })
}

fn editor_demo_base_color_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Color> {
    named_demo_state(cx, "imui_editor_proof_demo.model.base_color", |cx| {
        cx.app.models_mut().insert(Color {
            r: 0.9,
            g: 0.2,
            b: 0.2,
            a: 1.0,
        })
    })
}

fn editor_demo_position_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<f64>, Model<f64>, Model<f64>) {
    named_demo_state(cx, "imui_editor_proof_demo.model.position", |cx| {
        let x = cx.app.models_mut().insert(0.0_f64);
        let y = cx.app.models_mut().insert(1.0_f64);
        let z = cx.app.models_mut().insert(0.0_f64);
        (x, y, z)
    })
}

fn editor_demo_rotation_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<f64>, Model<f64>, Model<f64>) {
    named_demo_state(cx, "imui_editor_proof_demo.model.rotation", |cx| {
        let x = cx.app.models_mut().insert(0.0_f64);
        let y = cx.app.models_mut().insert(0.0_f64);
        let z = cx.app.models_mut().insert(0.0_f64);
        (x, y, z)
    })
}

fn editor_demo_scale_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<f64>, Model<f64>, Model<f64>) {
    named_demo_state(cx, "imui_editor_proof_demo.model.scale", |cx| {
        let x = cx.app.models_mut().insert(1.0_f64);
        let y = cx.app.models_mut().insert(1.0_f64);
        let z = cx.app.models_mut().insert(1.0_f64);
        (x, y, z)
    })
}

fn editor_demo_alpha_clip_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    named_demo_state(cx, "imui_editor_proof_demo.model.alpha_clip", |cx| {
        cx.app.models_mut().insert(false)
    })
}

fn editor_demo_cast_shadows_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<bool>> {
    named_demo_state(cx, "imui_editor_proof_demo.model.cast_shadows", |cx| {
        // Start in "mixed/indeterminate" to exercise tri-state checkbox rendering.
        cx.app.models_mut().insert(None::<bool>)
    })
}

fn editor_demo_shading_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    named_demo_state(cx, "imui_editor_proof_demo.model.shading_model", |cx| {
        cx.app
            .models_mut()
            .insert(Some::<Arc<str>>(Arc::from("lit")))
    })
}

fn editor_demo_iterations_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<i32> {
    named_demo_state(cx, "imui_editor_proof_demo.model.iterations", |cx| {
        cx.app.models_mut().insert(16_i32)
    })
}

fn editor_demo_search_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.search", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

fn editor_demo_name_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.name", |cx| {
        cx.app.models_mut().insert("Cube".to_string())
    })
}

fn editor_demo_notes_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.notes", |cx| {
        cx.app
            .models_mut()
            .insert("Multiline TextField (v1)\n- uses TextArea\n- clear affordance\n".to_string())
    })
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
                    fret_ui_kit::ui::container_build( move |cx, out| {
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
            fret_launch::WindowLogicalSize::new(720.0, 520.0),
        )),
        CreateWindowKind::DockRestore { logical_window_id } => {
            Some(fret_launch::WindowCreateSpec::new(
                format!("fret-demo imui_editor_proof_demo — {logical_window_id}"),
                fret_launch::WindowLogicalSize::new(980.0, 720.0),
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
