use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret::advanced::interop::embedded_viewport as embedded;
use fret::advanced::view::{AppRenderDataExt as _, ViewWindowState};
use fret::imui::{kit::ImUiMultiSelectState, prelude::*};
use fret::{Defaults, FretApp, advanced::prelude::*, component::prelude::*, shadcn};
use fret_core::{Color, KeyCode, Modifiers, PanelKind, Point, PointerId, Px, Rect, Size};
use fret_docking::{DockSpaceElementOptions, runtime as dock_runtime};
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::{
    FrameId, Model, PlatformCapabilities, TickId, TimerToken, WindowHoverDetectionQuality,
};
use fret_ui::GlobalElementId;
use fret_ui::action::{UiActionHostExt as _, UiFocusActionHost};
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle};
use fret_ui::scroll::ScrollHandle;
use fret_ui_editor::composites::{
    GradientEditor, GradientEditorOptions, GradientStopBinding, InspectorPanel,
    InspectorPanelOptions, InspectorPanelSearchAssistOptions, PropertyGrid, PropertyGroup,
    PropertyRow, PropertyRowReset,
};
use fret_ui_editor::controls::{
    Checkbox, ColorEdit, ColorEditOptions, DragValue, DragValueOutcome,
    EditorTextSelectionBehavior, EnumSelect, EnumSelectItem, EnumSelectOptions, FieldStatus,
    FieldStatusBadge, NumericInput, NumericInputOptions, NumericValidateFn,
    NumericValueConstraints, Slider, SliderOptions, TextAssistField, TextAssistFieldOptions,
    TextAssistFieldSurface, TextField, TextFieldBlurBehavior, TextFieldMode, TextFieldOptions,
    TextFieldOutcome, TransformEdit, TransformEditAxisOutcome, TransformEditOptions, Vec3Edit,
    VecEditAxisOutcome, VecEditOptions,
};
use fret_ui_editor::imui as editor_imui;
use fret_ui_editor::theme::EditorThemePresetV1;
use fret_ui_kit::headless::text_assist::{TextAssistItem, TextAssistMatch};
use fret_ui_kit::recipes::imui_drag_preview::{
    DragPreviewGhostOptions, drag_preview_ghost_with_options,
    publish_cross_window_drag_preview_ghost_with_options, render_cross_window_drag_preview_ghosts,
};
use fret_ui_kit::recipes::imui_sortable::{
    SortableInsertionSide, reorder_vec_by_key, sortable_row,
};

mod asset_ref;
mod authoring_parity;
mod collection;
mod proof_helpers;
mod workbench_shell;

use proof_helpers::*;

const VIEWPORT_PX_SIZE: (u32, u32) = (960, 540);
const AUX_LOGICAL_WINDOW_ID: &str = "aux";
const ENV_SINGLE_WINDOW: &str = "FRET_IMUI_EDITOR_PROOF_SINGLE_WINDOW";
const ENV_EDITOR_PRESET: &str = "FRET_IMUI_EDITOR_PRESET";
const ENV_PROOF_LAYOUT: &str = "FRET_IMUI_EDITOR_PROOF_LAYOUT";
const EDITOR_HOST_BASE_COLOR: shadcn::themes::ShadcnBaseColor =
    shadcn::themes::ShadcnBaseColor::Slate;
const EDITOR_HOST_DEFAULT_SCHEME: shadcn::themes::ShadcnColorScheme =
    shadcn::themes::ShadcnColorScheme::Dark;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ImUiEditorProofLayout {
    #[default]
    Full,
    EditorReview,
}

fn diag_enabled() -> bool {
    std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty() && v != "0")
}

fn selected_editor_theme_preset() -> EditorThemePresetV1 {
    // This proof demo is explicitly editor-grade, so prefer the dense imgui-inspired preset by
    // default and keep the conservative baseline available via `FRET_IMUI_EDITOR_PRESET=default`
    // for A/B screenshots and regression triage.
    crate::editor_theme_preset_from_env(ENV_EDITOR_PRESET)
        .unwrap_or(EditorThemePresetV1::ImguiLikeDense)
}

fn selected_proof_layout() -> ImUiEditorProofLayout {
    let Some(raw) = std::env::var_os(ENV_PROOF_LAYOUT) else {
        return ImUiEditorProofLayout::Full;
    };

    match raw.to_string_lossy().trim().to_ascii_lowercase().as_str() {
        "editor_review" => ImUiEditorProofLayout::EditorReview,
        _ => ImUiEditorProofLayout::Full,
    }
}

fn editor_demo_name_assist_items(cx: &mut ElementContext<'_, KernelApp>) -> Arc<[TextAssistItem]> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.state.name_assist_items",
        |_cx| {
            vec![
                TextAssistItem::new("cube", "Cube").aliases(vec![Arc::from("box")]),
                TextAssistItem::new("cylinder", "Cylinder"),
                TextAssistItem::new("capsule", "Capsule"),
                TextAssistItem::new("camera", "Camera").aliases(vec![Arc::from("cam")]),
                TextAssistItem::new("curve-editor", "Curve Editor"),
                TextAssistItem::new("directional-light", "Directional Light")
                    .aliases(vec![Arc::from("dir light")]),
            ]
            .into()
        },
    )
}

fn editor_demo_search_assist_items(
    cx: &mut ElementContext<'_, KernelApp>,
) -> Arc<[TextAssistItem]> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.state.search_assist_items",
        |_cx| {
            vec![
                TextAssistItem::new("assist", "Assist"),
                TextAssistItem::new("material", "Material"),
                TextAssistItem::new("buffered", "Buffered"),
                TextAssistItem::new("gradient", "Gradient"),
                TextAssistItem::new("password", "Password"),
                TextAssistItem::new("validation", "Validation")
                    .aliases(vec![Arc::from("error"), Arc::from("invalid")]),
            ]
            .into()
        },
    )
}

fn record_editor_text_assist_accept(
    host: &mut dyn fret_ui::action::UiActionHost,
    accepted_label_model: &Model<String>,
    active: TextAssistMatch,
) {
    let next_query = active.label.as_ref().to_string();
    let _ = host.models_mut().update(accepted_label_model, |value| {
        value.clear();
        value.push_str(&next_query);
    });
}

fn record_text_field_outcome(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    outcome_model: &Model<String>,
    outcome: TextFieldOutcome,
) {
    let next = edit_session_outcome_label(outcome);
    let _ = host.models_mut().update(outcome_model, |text| {
        text.clear();
        text.push_str(next);
    });
    host.request_redraw(action_cx.window);
}

fn render_editor_name_assist_surface(
    cx: &mut ElementContext<'_, KernelApp>,
    query_model: Model<String>,
    dismissed_query_model: Model<String>,
    active_item_id_model: Model<Option<Arc<str>>>,
    accepted_label_model: Model<String>,
) -> impl IntoUiElement<KernelApp> + use<> {
    let items = editor_demo_name_assist_items(cx);
    TextAssistField::new(
        query_model,
        dismissed_query_model,
        active_item_id_model,
        items,
    )
    .options(TextAssistFieldOptions {
        field: TextFieldOptions {
            id_source: Some(Arc::from("imui-editor-proof.editor.object.name-assist")),
            placeholder: Some(Arc::from("Type to search object history")),
            clear_button: true,
            buffered: false,
            selection_behavior: EditorTextSelectionBehavior::SelectAllOnFocus,
            test_id: Some(Arc::from("imui-editor-proof.editor.object.name-assist")),
            clear_test_id: Some(Arc::from(
                "imui-editor-proof.editor.object.name-assist.clear",
            )),
            ..Default::default()
        },
        surface: TextAssistFieldSurface::AnchoredOverlay,
        list_label: Arc::from("Name history suggestions"),
        list_test_id: Some(Arc::from(
            "imui-editor-proof.editor.object.name-assist.list",
        )),
        empty_test_id: Some(Arc::from(
            "imui-editor-proof.editor.object.name-assist.no-matches",
        )),
        ..Default::default()
    })
    .on_accept(Some(Arc::new(move |host, _action_cx, active| {
        record_editor_text_assist_accept(host, &accepted_label_model, active);
    })))
    .into_element(cx)
}

fn configure_imui_editor_proof_driver(
    driver: fret::UiAppDriver<ViewWindowState<ImUiEditorProofView>>,
) -> fret::UiAppDriver<ViewWindowState<ImUiEditorProofView>> {
    driver
        .drive_embedded_viewport()
        .dock_op(workbench_shell::on_dock_op)
        .window_create_spec(workbench_shell::window_create_spec)
        .window_created(workbench_shell::window_created)
        .before_close_window(workbench_shell::before_close_window)
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
        _app: &mut KernelApp,
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
        .defaults(Defaults {
            shadcn: false,
            ..Defaults::desktop_app()
        })
        .view_with_hooks::<ImUiEditorProofView>(configure_imui_editor_proof_driver)?
        .setup_with(move |app| {
            configure_single_window_caps_if_requested(app);
            install_imui_editor_proof_theme(app);
            fret_icons_lucide::app::install(app);
            workbench_shell::install_dock_panel_registry(app);
        })
        .run()?;
    Ok(())
}

fn install_imui_editor_proof_theme(app: &mut KernelApp) {
    // This proof owns a fixed editor-grade baseline. Do not route it through the generic shadcn
    // environment-sync lifecycle or the host can flip back to the OS light theme mid-run.
    shadcn::themes::apply_shadcn_new_york(app, EDITOR_HOST_BASE_COLOR, EDITOR_HOST_DEFAULT_SCHEME);
    fret_ui_editor::theme::install_editor_theme_preset_v1(app, selected_editor_theme_preset());
}

fn single_window_mode_enabled() -> bool {
    std::env::var_os(ENV_SINGLE_WINDOW).is_some_and(|v| !v.is_empty() && v != "0")
}

fn configure_single_window_caps_if_requested(app: &mut KernelApp) {
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
    fn init(app: &mut KernelApp, window: AppWindowId) -> Self {
        embedded::ensure_models(app, window);
        if !single_window_mode_enabled() {
            workbench_shell::ensure_aux_window_requested(app, window);
        }

        Self {
            embedded: embedded::EmbeddedViewportSurface::new(
                wgpu::TextureFormat::Bgra8UnormSrgb,
                RenderTargetColorSpace::Srgb,
                VIEWPORT_PX_SIZE,
            ),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        render_view(cx)
    }
}

fn render_view<'a, Cx>(cx: &mut Cx) -> ViewElements
where
    Cx: fret::app::ElementContextAccess<'a, KernelApp>,
{
    let cx = cx.elements();
    let window = cx.window;
    let single = single_window_mode_enabled();
    let proof_layout = selected_proof_layout();
    let editor_review_layout = proof_layout == ImUiEditorProofLayout::EditorReview;
    let dock_test_id = workbench_shell::dock_test_id_for_window(cx.app, window);
    let editor_value_model = editor_demo_value_model(cx);
    let editor_drag_value_outcome_model = editor_demo_drag_value_outcome_model(cx);
    let editor_roughness_model = editor_demo_roughness_model(cx);
    let editor_metallic_model = editor_demo_metallic_model(cx);
    let editor_alpha_clip_model = editor_demo_alpha_clip_model(cx);
    let editor_cast_shadows_model = editor_demo_cast_shadows_model(cx);
    let editor_shading_model = editor_demo_shading_model(cx);
    let editor_base_color_model = editor_demo_base_color_model(cx);
    let editor_asset_slot_model = asset_ref::asset_slot_model(cx);
    let editor_asset_action_model = asset_ref::asset_action_model(cx);
    let editor_name_model = editor_demo_name_model(cx);
    let editor_buffered_name_model = editor_demo_buffered_name_model(cx);
    let editor_inline_rename_model = editor_demo_inline_rename_model(cx);
    let editor_inline_rename_outcome_model = editor_demo_inline_rename_outcome_model(cx);
    let editor_name_assist_model = editor_demo_name_assist_model(cx);
    let editor_name_assist_dismissed_query_model =
        editor_demo_name_assist_dismissed_query_model(cx);
    let editor_name_assist_active_item_model = editor_demo_name_assist_active_item_model(cx);
    let editor_name_assist_accepted_model = editor_demo_name_assist_accepted_model(cx);
    let editor_password_model = editor_demo_password_model(cx);
    let editor_password_outcome_model = editor_demo_password_outcome_model(cx);
    let editor_notes_model = editor_demo_notes_model(cx);
    let editor_notes_outcome_model = editor_demo_notes_outcome_model(cx);
    let (editor_pos_x, editor_pos_y, editor_pos_z) = editor_demo_position_models(cx);
    let editor_position_outcome_model = editor_demo_position_outcome_model(cx);
    let (editor_rot_x, editor_rot_y, editor_rot_z) = editor_demo_rotation_models(cx);
    let (editor_scl_x, editor_scl_y, editor_scl_z) = editor_demo_scale_models(cx);
    let editor_transform_outcome_model = editor_demo_transform_outcome_model(cx);
    let editor_iterations_model = editor_demo_iterations_model(cx);
    let editor_exposure_model = editor_demo_exposure_model(cx);
    let editor_search_model = editor_demo_search_model(cx);
    let editor_search_assist_dismissed_query_model =
        editor_demo_search_assist_dismissed_query_model(cx);
    let editor_search_assist_active_item_model = editor_demo_search_assist_active_item_model(cx);
    let editor_gradient_angle_model = editor_demo_gradient_angle_model(cx);
    let editor_gradient_stops_model = editor_demo_gradient_stops_model(cx);
    let editor_gradient_next_id_model = editor_demo_gradient_next_id_model(cx);
    let parity_models = authoring_parity::shared_models(cx);

    #[cfg(debug_assertions)]
    {
        debug_assert_ne!(
            editor_roughness_model.id(),
            editor_metallic_model.id(),
            "Roughness/Metallic models must be distinct; otherwise sliders will sync unintentionally."
        );
    }

    imui(cx, |ui| {
        let root_content = fret_ui_kit::ui::v_flex_build(move |cx, out| {
            imui_build(cx, out, |ui| {
                if !editor_review_layout {
                    proof_imui_section_text(
                        ui,
                        format!(
                            "imui editor-grade proof (M7): docking + multi-window + viewport surfaces (window={window:?})"
                        ),
                    );

                    if single {
                        proof_imui_readout_text(
                            ui,
                            format!(
                                "single-window mode enabled ({ENV_SINGLE_WINDOW}=1): dock tear-off should degrade to in-window floating"
                            ),
                        );
                    }

                    let controls = fret_ui_kit::ui::h_flex_build(move |cx, out| {
                        imui_build(cx, out, |ui| {
                            let reset = ui.button("Reset layout");
                            let _ = ui.tooltip_text_with_options(
                                "imui-editor-proof.controls.reset-layout.tooltip",
                                reset,
                                "Restore the canonical dock graph for this proof window.",
                                kit::TooltipOptions {
                                    open_delay_frames_override: Some(0),
                                    close_delay_frames_override: Some(0),
                                    test_id: Some(Arc::from(
                                        "imui-editor-proof.controls.reset-layout.tooltip",
                                    )),
                                    ..Default::default()
                                },
                            );
                            if reset.clicked() {
                                workbench_shell::reset_dock_graph(ui.cx_mut().app, window);
                                dock_runtime::request_dock_invalidation(ui.cx_mut().app, [window]);
                            }
                            let recenter = ui.button("Center floatings");
                            let _ = ui.tooltip_text_with_options(
                                "imui-editor-proof.controls.center-floatings.tooltip",
                                recenter,
                                "Recenter in-window floating panels without resetting content state.",
                                kit::TooltipOptions {
                                    open_delay_frames_override: Some(0),
                                    close_delay_frames_override: Some(0),
                                    test_id: Some(Arc::from(
                                        "imui-editor-proof.controls.center-floatings.tooltip",
                                    )),
                                    ..Default::default()
                                },
                            );
                            if recenter.clicked() {
                                dock_runtime::recenter_in_window_floatings(ui.cx_mut().app, window);
                            }
                        });
                    })
                    .gap(fret_ui_kit::Space::N2);
                    ui.add_ui(controls);

                    ui.separator();

                    proof_imui_compact_paragraph_text(
                        ui,
                        "authoring parity proof: shared models, left declarative, right imui adapters; compare drag scrub, typed numeric entry, and bounded slider surfaces, then verify each paired row stays in sync under the same preset",
                    );

                    let parity_models_for_surface = parity_models.clone();
                    ui.mount(move |cx| {
                        vec![
                            authoring_parity::render_surface(cx, parity_models_for_surface.clone())
                                .into_element(cx),
                        ]
                    });

                    proof_imui_compact_paragraph_text(
                        ui,
                            "shared state readout: each declarative/imui pair should mutate the same model, while drag, typed numeric, and slider stay intentionally distinct",
                    );

                    let parity_models_for_state = parity_models.clone();
                    ui.mount(move |cx| {
                        vec![authoring_parity::render_shared_state(
                            cx,
                            parity_models_for_state.clone(),
                        )
                        .into_element(cx)]
                    });
                    ui.separator();

                    proof_imui_section_text(
                        ui,
                        "fret-ui-editor (M2): PropertyGroup + PropertyGrid + search assist",
                    );
                }
                ui.mount(|cx| {
                    let fixed_presentation = editor_fixed_decimals_presentation();
                    let validate: NumericValidateFn<f64> = Arc::new(|v| {
                        if (0.0..=1.0).contains(&v) {
                            None
                        } else {
                            Some(Arc::from("Expected 0.0..=1.0"))
                        }
                    });

                    vec![InspectorPanel::new(Some(editor_search_model.clone()))
                        .options(InspectorPanelOptions {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: if editor_review_layout {
                                        Length::Fill
                                    } else {
                                        Length::Auto
                                    },
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            test_id: Some(Arc::from("imui-editor-proof.editor.inspector")),
                            header_test_id: Some(Arc::from(
                                "imui-editor-proof.editor.inspector.header",
                            )),
                            search_test_id: Some(Arc::from("imui-editor-proof.editor.search")),
                            search_clear_test_id: Some(Arc::from(
                                "imui-editor-proof.editor.search.clear",
                            )),
                            search_assist: Some(InspectorPanelSearchAssistOptions {
                                dismissed_query_model: editor_search_assist_dismissed_query_model
                                    .clone(),
                                active_item_id_model: editor_search_assist_active_item_model
                                    .clone(),
                                items: editor_demo_search_assist_items(cx),
                                list_label: Arc::from("Inspector search history"),
                                empty_label: Arc::from("No search history matches"),
                                key_options: Default::default(),
                                list_test_id: Some(Arc::from(
                                    "imui-editor-proof.editor.search.list",
                                )),
                                item_test_id_prefix: Some(Arc::from(
                                    "imui-editor-proof.editor.search.list.item",
                                )),
                                empty_test_id: Some(Arc::from(
                                    "imui-editor-proof.editor.search.no-matches",
                                )),
                                max_list_height: None,
                            }),
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
                                let show_asset_ref = material_show_all
                                    || matches("asset")
                                    || matches("texture")
                                    || matches("map")
                                    || matches("base");
                                let show_shading_model =
                                    material_show_all || matches("shading") || matches("model");
                                let show_alpha_clip =
                                    material_show_all || matches("alpha") || matches("clip");
                                let show_cast_shadows =
                                    material_show_all || matches("shadow") || matches("shadows");

                                let advanced_show_all = matches("advanced");
                                let show_exposure =
                                    advanced_show_all || matches("exposure") || matches("validate");
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
                                    || show_asset_ref
                                    || show_shading_model
                                    || show_alpha_clip
                                    || show_cast_shadows
                                    || show_exposure
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
                                                        PropertyRow::new(),
                                                        |cx| row_cx.label_text(cx, "Name"),
                                                        |cx| {
                                                            TextField::new(
                                                                editor_name_model.clone(),
                                                            )
                                                            .options(TextFieldOptions {
                                                                placeholder: Some(Arc::from(
                                                                    "Untitled",
                                                                )),
                                                                clear_button: true,
                                                                selection_behavior:
                                                                    EditorTextSelectionBehavior::SelectAllOnFocus,
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
                                                        PropertyRow::new(),
                                                        |cx| row_cx.label_text(cx, "Inline rename"),
                                                        |cx| {
                                                            let outcome_model =
                                                                editor_inline_rename_outcome_model
                                                                    .clone();
                                                            TextField::new(
                                                                editor_inline_rename_model.clone(),
                                                            )
                                                            .on_outcome(Some(Arc::new(
                                                                move |host, action_cx, outcome: TextFieldOutcome| {
                                                                    record_text_field_outcome(
                                                                        host,
                                                                        action_cx,
                                                                        &outcome_model,
                                                                        outcome,
                                                                    );
                                                                },
                                                            )))
                                                            .options(TextFieldOptions {
                                                                id_source: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.inline-rename",
                                                                )),
                                                                placeholder: Some(Arc::from(
                                                                    "Rename selection",
                                                                )),
                                                                clear_button: true,
                                                                selection_behavior:
                                                                    EditorTextSelectionBehavior::SelectAllOnFocus,
                                                                blur_behavior:
                                                                    TextFieldBlurBehavior::Cancel,
                                                                test_id: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.inline-rename",
                                                                )),
                                                                clear_test_id: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.inline-rename.clear",
                                                                )),
                                                                ..Default::default()
                                                            })
                                                            .into_element(cx)
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    let inline_rename_readout =
                                                        editor_text_field_readout(
                                                            cx,
                                                            &editor_inline_rename_model,
                                                            &editor_inline_rename_outcome_model,
                                                        );
                                                    let inline_rename_committed =
                                                        inline_rename_readout.committed.clone();
                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new(),
                                                        |cx| {
                                                            row_cx.label_text(cx, "Rename committed")
                                                        },
                                                        move |cx| {
                                                            proof_compact_readout(
                                                                cx,
                                                                inline_rename_committed.clone(),
                                                                Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.inline-rename.committed",
                                                                )),
                                                            )
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    let inline_rename_outcome =
                                                        inline_rename_readout.outcome;
                                                    if !inline_rename_outcome.trim().is_empty() {
                                                        rows.push(row_cx.row_with(
                                                            cx,
                                                            PropertyRow::new(),
                                                            |cx| {
                                                                row_cx.label_text(cx, "Rename outcome")
                                                            },
                                                            move |cx| {
                                                                let outcome =
                                                                    inline_rename_outcome.clone();
                                                                proof_compact_readout(
                                                                    cx,
                                                                    outcome,
                                                                    Some(Arc::from(
                                                                        "imui-editor-proof.editor.object.inline-rename.outcome",
                                                                    )),
                                                                )
                                                            },
                                                            |_cx| None,
                                                        ));
                                                    }

                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new(),
                                                        |cx| row_cx.label_text(cx, "Buffered name"),
                                                        |cx| {
                                                            TextField::new(
                                                                editor_buffered_name_model
                                                                    .clone(),
                                                            )
                                                            .options(TextFieldOptions {
                                                                id_source: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.buffered-name",
                                                                )),
                                                                placeholder: Some(Arc::from(
                                                                    "Buffered session",
                                                                )),
                                                                clear_button: true,
                                                                test_id: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.buffered-name",
                                                                )),
                                                                clear_test_id: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.buffered-name.clear",
                                                                )),
                                                                ..Default::default()
                                                            })
                                                            .into_element(cx)
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new(),
                                                        |cx| row_cx.label_text(cx, "Password"),
                                                        |cx| {
                                                            let outcome_model =
                                                                editor_password_outcome_model
                                                                    .clone();
                                                            TextField::new(
                                                                editor_password_model.clone(),
                                                            )
                                                            .on_outcome(Some(Arc::new(
                                                                move |host, action_cx, outcome: TextFieldOutcome| {
                                                                    record_text_field_outcome(
                                                                        host,
                                                                        action_cx,
                                                                        &outcome_model,
                                                                        outcome,
                                                                    );
                                                                },
                                                            )))
                                                            .options(TextFieldOptions {
                                                                id_source: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.password",
                                                                )),
                                                                placeholder: Some(Arc::from(
                                                                    "Editor password",
                                                                )),
                                                                clear_button: true,
                                                                mode: TextFieldMode::Password,
                                                                test_id: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.password",
                                                                )),
                                                                clear_test_id: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.password.clear",
                                                                )),
                                                                ..Default::default()
                                                            })
                                                            .into_element(cx)
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    let password_readout = editor_text_field_readout(
                                                        cx,
                                                        &editor_password_model,
                                                        &editor_password_outcome_model,
                                                    );
                                                    let password_committed =
                                                        password_readout.committed.clone();
                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new(),
                                                        |cx| row_cx.label_text(cx, "Secret length"),
                                                        move |cx| {
                                                            let readout =
                                                                committed_char_count_label(
                                                                    &password_committed,
                                                                );
                                                            proof_compact_readout(
                                                                cx,
                                                                readout,
                                                                Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.password.committed-length",
                                                                )),
                                                            )
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    let password_outcome =
                                                        password_readout.outcome;
                                                    if !password_outcome.trim().is_empty() {
                                                        rows.push(row_cx.row_with(
                                                            cx,
                                                            PropertyRow::new(),
                                                            |cx| {
                                                                row_cx.label_text(cx, "Password outcome")
                                                            },
                                                            move |cx| {
                                                                let outcome =
                                                                    password_outcome.clone();
                                                                proof_compact_readout(
                                                                    cx,
                                                                    outcome,
                                                                    Some(Arc::from(
                                                                        "imui-editor-proof.editor.object.password.outcome",
                                                                    )),
                                                                )
                                                            },
                                                            |_cx| None,
                                                        ));
                                                    }

                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new(),
                                                        |cx| row_cx.label_text(cx, "Committed"),
                                                        |cx| {
                                                            let committed = editor_string_model_readout(
                                                                cx,
                                                                &editor_buffered_name_model,
                                                            );
                                                            proof_compact_readout(
                                                                cx,
                                                                committed,
                                                                Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.buffered-name.committed",
                                                                )),
                                                            )
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new(),
                                                        |cx| row_cx.label_text(cx, "Name assist"),
                                                        |cx| {
                                                            render_editor_name_assist_surface(
                                                                cx,
                                                                editor_name_assist_model.clone(),
                                                                editor_name_assist_dismissed_query_model
                                                                    .clone(),
                                                                editor_name_assist_active_item_model
                                                                    .clone(),
                                                                editor_name_assist_accepted_model
                                                                    .clone(),
                                                            )
                                                            .into_element(cx)
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    let name_assist_items =
                                                        editor_demo_name_assist_items(cx);
                                                    let name_assist_readout =
                                                        editor_text_assist_readout(
                                                            cx,
                                                            name_assist_items,
                                                            &editor_name_assist_model,
                                                            &editor_name_assist_dismissed_query_model,
                                                            &editor_name_assist_active_item_model,
                                                        );
                                                    let name_assist_state =
                                                        name_assist_readout.state_label.clone();
                                                    let name_assist_active =
                                                        name_assist_readout.active_label.clone();

                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new(),
                                                        |cx| row_cx.label_text(cx, "Assist state"),
                                                        move |cx| {
                                                            let state = name_assist_state.clone();
                                                            proof_compact_readout(
                                                                cx,
                                                                state,
                                                                Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.name-assist.state",
                                                                )),
                                                            )
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new(),
                                                        |cx| row_cx.label_text(cx, "Active assist"),
                                                        move |cx| {
                                                            let active_label =
                                                                name_assist_active.clone();
                                                            proof_compact_readout(
                                                                cx,
                                                                active_label,
                                                                Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.name-assist.active",
                                                                )),
                                                            )
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new(),
                                                        |cx| row_cx.label_text(cx, "Accepted assist"),
                                                        |cx| {
                                                            let accepted = editor_string_model_readout(
                                                                cx,
                                                                &editor_name_assist_accepted_model,
                                                            );
                                                            let readout = if accepted.trim().is_empty() {
                                                                "None".to_string()
                                                            } else {
                                                                accepted
                                                            };
                                                            proof_compact_readout(
                                                                cx,
                                                                readout,
                                                                Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.name-assist.accepted",
                                                                )),
                                                            )
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new(),
                                                        |cx| row_cx.label_text(cx, "Notes"),
                                                        |cx| {
                                                            let outcome_model =
                                                                editor_notes_outcome_model.clone();
                                                            TextField::new(
                                                                editor_notes_model.clone(),
                                                            )
                                                            .on_outcome(Some(Arc::new(
                                                                move |host, action_cx, outcome: TextFieldOutcome| {
                                                                    record_text_field_outcome(
                                                                        host,
                                                                        action_cx,
                                                                        &outcome_model,
                                                                        outcome,
                                                                    );
                                                                },
                                                            )))
                                                            .options(TextFieldOptions {
                                                                id_source: Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.notes",
                                                                )),
                                                                multiline: true,
                                                                min_height: Some(Px(96.0)),
                                                                clear_button: true,
                                                                blur_behavior:
                                                                    TextFieldBlurBehavior::PreserveDraft,
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

                                                    let notes_readout = editor_text_field_readout(
                                                        cx,
                                                        &editor_notes_model,
                                                        &editor_notes_outcome_model,
                                                    );
                                                    let notes_committed =
                                                        notes_readout.committed.clone();
                                                    rows.push(row_cx.row_with(
                                                        cx,
                                                        PropertyRow::new(),
                                                        |cx| {
                                                            row_cx.label_text(cx, "Notes committed")
                                                        },
                                                        move |cx| {
                                                            let readout =
                                                                committed_line_count_label(
                                                                    &notes_committed,
                                                                );
                                                            proof_compact_readout(
                                                                cx,
                                                                readout,
                                                                Some(Arc::from(
                                                                    "imui-editor-proof.editor.object.notes.committed-lines",
                                                                )),
                                                            )
                                                        },
                                                        |_cx| None,
                                                    ));

                                                    let notes_outcome = notes_readout.outcome;
                                                    if !notes_outcome.trim().is_empty() {
                                                        rows.push(row_cx.row_with(
                                                            cx,
                                                            PropertyRow::new(),
                                                            |cx| {
                                                                row_cx.label_text(cx, "Notes outcome")
                                                            },
                                                            move |cx| {
                                                                let outcome =
                                                                    notes_outcome.clone();
                                                                proof_compact_readout(
                                                                    cx,
                                                                    outcome,
                                                                    Some(Arc::from(
                                                                        "imui-editor-proof.editor.object.notes.outcome",
                                                                    )),
                                                                )
                                                            },
                                                            |_cx| None,
                                                        ));
                                                    }

                                                    rows
                                                },
                                            )]
                                        },
                                    ),
                            );

                            let material_validate = validate.clone();
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
                                            let validate = material_validate.clone();
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
                                                            PropertyRow::new()
                                                                .reset(Some(
                                                                    PropertyRowReset::new(
                                                                        on_reset,
                                                                    )
                                                                    .options(
                                                                        fret_ui_editor::composites::PropertyRowResetOptions {
                                                                            test_id: Some(Arc::from("imui-editor-proof.editor.drag-value-reset")),
                                                                            ..Default::default()
                                                                        },
                                                                    ),
                                                                )),
                                                            |cx| row_cx.label_text(cx, "Opacity"),
                                                            |cx| {
                                                                let outcome_model =
                                                                    editor_drag_value_outcome_model
                                                                        .clone();
                                                                DragValue::from_presentation(
                                                                    editor_value_model.clone(),
                                                                    fixed_presentation.clone(),
                                                                )
                                                                .validate(Some(validate.clone()))
                                                                .on_outcome(Some(Arc::new(
                                                                    move |host,
                                                                          action_cx,
                                                                          outcome: DragValueOutcome| {
                                                                        let next =
                                                                            compact_edit_session_outcome_label(
                                                                                outcome,
                                                                            );
                                                                        let _ = host
                                                                            .models_mut()
                                                                            .update(
                                                                                &outcome_model,
                                                                                |value| {
                                                                                    value.clear();
                                                                                    value.push_str(
                                                                                        next,
                                                                                    );
                                                                                },
                                                                            );
                                                                        host.request_redraw(
                                                                            action_cx.window,
                                                                        );
                                                                    },
                                                                )))
                                                                .options(
                                                                    fret_ui_editor::controls::DragValueOptions {
                                                                        constraints:
                                                                            NumericValueConstraints {
                                                                                min: Some(0.0),
                                                                                max: Some(1.0),
                                                                                clamp: true,
                                                                                step: Some(0.025),
                                                                            },
                                                                        test_id: Some(Arc::from(
                                                                            "imui-editor-proof.editor.drag-value-demo",
                                                                        )),
                                                                        ..Default::default()
                                                                    },
                                                                )
                                                                .into_element(cx)
                                                            },
                                                            |cx| {
                                                                let outcome = editor_string_model_readout(
                                                                    cx,
                                                                    &editor_drag_value_outcome_model,
                                                                );
                                                                proof_optional_outcome_readout(
                                                                    cx,
                                                                    outcome,
                                                                    Arc::from(
                                                                        "imui-editor-proof.editor.drag-value-demo.outcome",
                                                                    ),
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
                                                            PropertyRow::new()
                                                                .reset(Some(
                                                                    PropertyRowReset::new(
                                                                        on_reset,
                                                                    )
                                                                    .options(
                                                                        fret_ui_editor::composites::PropertyRowResetOptions {
                                                                            test_id: Some(Arc::from("imui-editor-proof.editor.material.roughness.reset")),
                                                                            ..Default::default()
                                                                        },
                                                                    ),
                                                                )),
                                                            |cx| row_cx.label_text(cx, "Roughness"),
                                                            |cx| {
                                                                Slider::from_presentation(
                                                                    editor_roughness_model.clone(),
                                                                    0.0,
                                                                    1.0,
                                                                    editor_percent_presentation(),
                                                                )
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
                                                            PropertyRow::new()
                                                                .reset(Some(
                                                                    PropertyRowReset::new(
                                                                        on_reset,
                                                                    )
                                                                    .options(
                                                                        fret_ui_editor::composites::PropertyRowResetOptions {
                                                                            test_id: Some(Arc::from("imui-editor-proof.editor.material.metallic.reset")),
                                                                            ..Default::default()
                                                                        },
                                                                    ),
                                                                )),
                                                            |cx| row_cx.label_text(cx, "Metallic"),
                                                            |cx| {
                                                                Slider::from_presentation(
                                                                    editor_metallic_model.clone(),
                                                                    0.0,
                                                                    1.0,
                                                                    editor_percent_presentation(),
                                                                )
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
                                                            |cx| row_cx.label_text(cx, "Base color"),
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

                                                    if show_asset_ref {
                                                        asset_ref::push_material_rows(
                                                            &mut rows,
                                                            cx,
                                                            &row_cx,
                                                            editor_asset_slot_model.clone(),
                                                            editor_asset_action_model.clone(),
                                                        );
                                                    }

                                                    if show_shading_model {
                                                        let items = editor_material_shading_items();

                                                        rows.push(row_cx.row(
                                                            cx,
                                                            |cx| row_cx.label_text(cx, "Shading model"),
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
                                                                    max_list_height: Some(Px(144.0)),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(cx)
                                                            },
                                                        ));
                                                    }

                                                    if show_alpha_clip {
                                                        rows.push(row_cx.row(
                                                            cx,
                                                            |cx| row_cx.label_text(cx, "Alpha clip"),
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
                                                            |cx| row_cx.label_text(cx, "Cast shadows"),
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
                                                        rows.push(proof_empty_state_text(
                                                            cx,
                                                            "No matches",
                                                            "imui-editor-proof.editor.material.no-matches",
                                                        ));
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
                                            let stops = cx.data().selector_model_paint(
                                                &editor_gradient_stops_model,
                                                |stops| stops,
                                            );

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

                            let advanced_validate = validate.clone();
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
                                            let validate = advanced_validate.clone();
                                            let fixed_presentation =
                                                editor_fixed_decimals_presentation();
                                            let position_presentation =
                                                editor_position_presentation();
                                            let transform_presentations =
                                                editor_transform_presentations();
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
                                                            PropertyRow::new()
                                                                .reset(Some(
                                                                    PropertyRowReset::new(
                                                                        on_reset,
                                                                    )
                                                                    .options(
                                                                        fret_ui_editor::composites::PropertyRowResetOptions {
                                                                            test_id: Some(Arc::from("imui-editor-proof.editor.advanced.position.reset")),
                                                                            ..Default::default()
                                                                        },
                                                                    ),
                                                                )),
                                                            |cx| row_cx.label_text(cx, "Position"),
                                                            |cx| {
                                                                let outcome_model =
                                                                    editor_position_outcome_model
                                                                        .clone();
                                                                Vec3Edit::from_presentation(
                                                                    editor_pos_x.clone(),
                                                                    editor_pos_y.clone(),
                                                                    editor_pos_z.clone(),
                                                                    position_presentation.clone(),
                                                                )
                                                                .on_axis_outcome(Some(Arc::new(
                                                                    move |host, action_cx, outcome: VecEditAxisOutcome| {
                                                                        let next =
                                                                            vec_edit_axis_outcome_label(
                                                                                outcome,
                                                                            );
                                                                        let _ = host.models_mut().update(
                                                                            &outcome_model,
                                                                            |value| {
                                                                                value.clear();
                                                                                value.push_str(&next);
                                                                            },
                                                                        );
                                                                        host.request_redraw(
                                                                            action_cx.window,
                                                                        );
                                                                    },
                                                                )))
                                                                .options(VecEditOptions {
                                                                    test_id: Some(Arc::from(
                                                                        "imui-editor-proof.editor.advanced.position",
                                                                    )),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(cx)
                                                            },
                                                            |cx| {
                                                                let outcome = editor_string_model_readout(
                                                                    cx,
                                                                    &editor_position_outcome_model,
                                                                );
                                                                proof_optional_outcome_readout(
                                                                    cx,
                                                                    outcome,
                                                                    Arc::from(
                                                                        "imui-editor-proof.editor.advanced.position.outcome",
                                                                    ),
                                                                )
                                                            },
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
                                                                .reset(Some(
                                                                    PropertyRowReset::new(on_reset)
                                                                        .options(
                                                                            fret_ui_editor::composites::PropertyRowResetOptions {
                                                                                test_id: Some(Arc::from("imui-editor-proof.editor.advanced.transform.reset")),
                                                                                ..Default::default()
                                                                            },
                                                                        ),
                                                                )),
                                                            |cx| row_cx.label_text(cx, "Transform"),
                                                            |cx| {
                                                                let outcome_model =
                                                                    editor_transform_outcome_model
                                                                        .clone();
                                                                TransformEdit::from_presentations(
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
                                                                    transform_presentations.clone(),
                                                                )
                                                                .on_axis_outcome(Some(Arc::new(
                                                                    move |host,
                                                                          action_cx,
                                                                          outcome: TransformEditAxisOutcome| {
                                                                        let next =
                                                                            transform_edit_axis_outcome_label(
                                                                                outcome,
                                                                            );
                                                                        let _ = host.models_mut().update(
                                                                            &outcome_model,
                                                                            |value| {
                                                                                value.clear();
                                                                                value.push_str(&next);
                                                                            },
                                                                        );
                                                                        host.request_redraw(
                                                                            action_cx.window,
                                                                        );
                                                                    },
                                                                )))
                                                                .options(TransformEditOptions {
                                                                    test_id: Some(Arc::from("imui-editor-proof.editor.advanced.transform")),
                                                                    link_test_id: Some(Arc::from("imui-editor-proof.editor.advanced.transform.link-scale")),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(cx)
                                                            },
                                                            |cx| {
                                                                let outcome = editor_string_model_readout(
                                                                    cx,
                                                                    &editor_transform_outcome_model,
                                                                );
                                                                proof_optional_outcome_readout(
                                                                    cx,
                                                                    outcome,
                                                                    Arc::from(
                                                                        "imui-editor-proof.editor.advanced.transform.outcome",
                                                                    ),
                                                                )
                                                            },
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

                                                        rows.push(row_cx.row_with(
                                                            cx,
                                                            PropertyRow::new().reset(Some(
                                                                PropertyRowReset::new(on_reset).options(
                                                                    fret_ui_editor::composites::PropertyRowResetOptions {
                                                                        test_id: Some(Arc::from("imui-editor-proof.editor.advanced.iterations.reset")),
                                                                        ..Default::default()
                                                                    },
                                                                ),
                                                            )),
                                                            |cx| row_cx.label_text(cx, "Iterations"),
                                                            |cx| {
                                                                DragValue::new(
                                                                    editor_iterations_model.clone(),
                                                                    fmt_i32.clone(),
                                                                    parse_i32.clone(),
                                                                )
                                                                .options(
                                                                    fret_ui_editor::controls::DragValueOptions {
                                                                        test_id: Some(Arc::from(
                                                                            "imui-editor-proof.editor.advanced.iterations",
                                                                        )),
                                                                        ..Default::default()
                                                                    },
                                                                )
                                                                .into_element(cx)
                                                            },
                                                            |cx| {
                                                                Some(
                                                                    FieldStatusBadge::new(FieldStatus::Error(
                                                                        Arc::from("stub"),
                                                                    ))
                                                                    .into_element(cx),
                                                                )
                                                            },
                                                        ));
                                                    }

                                                    if show_exposure {
                                                        let model_for_reset =
                                                            editor_exposure_model.clone();
                                                        let on_reset = Arc::new(
                                                            move |host: &mut dyn fret_ui::action::UiActionHost,
                                                                  action_cx: fret_ui::action::ActionCx| {
                                                                let _ = host.models_mut().update(
                                                                    &model_for_reset,
                                                                    |v| *v = 0.75,
                                                                );
                                                                host.request_redraw(action_cx.window);
                                                            },
                                                        );

                                                        rows.push(row_cx.row_with(
                                                            cx,
                                                            PropertyRow::new().reset(Some(
                                                                PropertyRowReset::new(on_reset).options(
                                                                    fret_ui_editor::composites::PropertyRowResetOptions {
                                                                        test_id: Some(Arc::from("imui-editor-proof.editor.advanced.exposure.reset")),
                                                                        ..Default::default()
                                                                    },
                                                                ),
                                                            )),
                                                            |cx| row_cx.label_text(cx, "Exposure"),
                                                            |cx| {
                                                                NumericInput::from_presentation(
                                                                    editor_exposure_model.clone(),
                                                                    fixed_presentation.clone(),
                                                                )
                                                                .validate(Some(validate.clone()))
                                                                .options(NumericInputOptions {
                                                                    test_id: Some(Arc::from(
                                                                        "imui-editor-proof.editor.advanced.exposure",
                                                                    )),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(cx)
                                                            },
                                                            |_cx| None,
                                                        ));
                                                    }

                                                    if rows.is_empty() {
                                                        rows.push(proof_empty_state_text(
                                                            cx,
                                                            "No matches",
                                                            "imui-editor-proof.editor.advanced.no-matches",
                                                        ));
                                                    }

                                                    rows
                                                },
                                            )]
                                        },
                                    ),
                            );

                            if !panel_cx.is_query_empty() && !any_match {
                                out.push(proof_empty_state_text(
                                    cx,
                                    "No matches",
                                    "imui-editor-proof.editor.no-matches",
                                ));
                            }

                            out
                            },
                        )]
                });
                if !editor_review_layout {
                    ui.separator();

                    ui.with_cx_mut(|cx| {
                        workbench_shell::ensure_dock_graph(cx.app, cx.window);
                    });
                    fret_docking::imui::dock_space_declarative_with(
                        ui,
                        DockSpaceElementOptions {
                            test_id: dock_test_id,
                            ..Default::default()
                        },
                    );
                }
            });
        })
        .w_full()
        .min_w_0();

        if editor_review_layout {
            ui.add_ui(root_content.h_full().min_h_0());
        } else {
            ui.add_ui(
                fret_ui_kit::ui::scroll_area(move |cx| [root_content.into_element(cx)])
                    .viewport_test_id("imui-editor-proof.root.viewport")
                    .show_scrollbar_y(true)
                    .show_scrollbar_x(false)
                    .w_full()
                    .h_full()
                    .min_h_0(),
            );
        }
        let _ = render_cross_window_drag_preview_ghosts(ui.cx_mut());
    })
}

fn editor_material_shading_items() -> Arc<[EnumSelectItem]> {
    vec![
        EnumSelectItem::new("lit", "Lit"),
        EnumSelectItem::new("unlit", "Unlit"),
        EnumSelectItem::new("subsurface", "Subsurface"),
        EnumSelectItem::new("clearcoat", "Clearcoat"),
        EnumSelectItem::new("sheen", "Sheen"),
        EnumSelectItem::new("anisotropy", "Anisotropy"),
        EnumSelectItem::new("iridescence", "Iridescence"),
        EnumSelectItem::new("transmission", "Transmission"),
        EnumSelectItem::new("specular-gloss", "Specular gloss"),
        EnumSelectItem::new("matcap", "Matcap"),
        EnumSelectItem::new("toon", "Toon"),
        EnumSelectItem::new("cloth", "Cloth"),
    ]
    .into()
}

fn named_demo_state<H: UiHost, S: Clone + 'static>(
    cx: &mut ElementContext<'_, H>,
    name: &'static str,
    init: impl FnOnce(&mut ElementContext<'_, H>) -> S,
) -> S {
    cx.named(name, |cx| {
        let slot = cx.slot_id();
        let existing = cx.state_for(slot, || None::<S>, |st| st.clone());
        match existing {
            Some(v) => v,
            None => {
                let v = init(cx);
                cx.state_for(
                    slot,
                    || None::<S>,
                    |st| {
                        if st.is_none() {
                            *st = Some(v.clone());
                        }
                        st.clone()
                            .expect("named_demo_state slot must contain a value after init")
                    },
                )
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
            .insert(Some::<Arc<str>>(Arc::from("cloth")))
    })
}

fn editor_demo_iterations_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<i32> {
    named_demo_state(cx, "imui_editor_proof_demo.model.iterations", |cx| {
        cx.app.models_mut().insert(16_i32)
    })
}

fn editor_demo_exposure_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(cx, "imui_editor_proof_demo.model.exposure", |cx| {
        cx.app.models_mut().insert(0.75_f64)
    })
}

fn editor_demo_search_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.search", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

fn editor_demo_search_assist_dismissed_query_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.search_assist_dismissed_query",
        |cx| cx.app.models_mut().insert(String::new()),
    )
}

fn editor_demo_search_assist_active_item_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<Arc<str>>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.search_assist_active_item",
        |cx| cx.app.models_mut().insert(None::<Arc<str>>),
    )
}

fn editor_demo_name_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.name", |cx| {
        cx.app.models_mut().insert("Cube".to_string())
    })
}

fn editor_demo_buffered_name_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.buffered_name", |cx| {
        cx.app.models_mut().insert("Buffered Cube".to_string())
    })
}

fn editor_demo_inline_rename_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.inline_rename", |cx| {
        cx.app.models_mut().insert("Props_Root".to_string())
    })
}

fn editor_demo_name_assist_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.name_assist", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

fn editor_demo_name_assist_dismissed_query_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.name_assist_dismissed_query",
        |cx| cx.app.models_mut().insert(String::new()),
    )
}

fn editor_demo_name_assist_active_item_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<Arc<str>>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.name_assist_active_item",
        |cx| cx.app.models_mut().insert(None::<Arc<str>>),
    )
}

fn editor_demo_name_assist_accepted_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.name_assist_accepted",
        |cx| cx.app.models_mut().insert(String::new()),
    )
}

fn editor_demo_password_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.password", |cx| {
        cx.app.models_mut().insert("secret42".to_string())
    })
}

fn editor_demo_drag_value_outcome_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.drag_value_outcome",
        |cx| cx.app.models_mut().insert(String::new()),
    )
}

fn editor_demo_password_outcome_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.password_outcome", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

fn editor_demo_inline_rename_outcome_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.inline_rename_outcome",
        |cx| cx.app.models_mut().insert(String::new()),
    )
}

fn editor_demo_notes_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.notes", |cx| {
        cx.app
            .models_mut()
            .insert("Multiline TextField (v1)\n- uses TextArea\n- clear affordance\n".to_string())
    })
}

fn editor_demo_notes_outcome_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.notes_outcome", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

fn editor_demo_position_outcome_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.position_outcome", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

fn editor_demo_transform_outcome_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.transform_outcome", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, TextOverflow, TextWrap};
    use fret_ui::element::ElementKind;
    use fret_ui::elements;
    use fret_ui_editor::primitives::EditSessionOutcome;

    fn test_bounds() -> Rect {
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(96.0)))
    }

    fn collect_text_props<'a>(
        element: &'a fret_ui::element::AnyElement,
        out: &mut Vec<&'a fret_ui::element::TextProps>,
    ) {
        if let ElementKind::Text(props) = &element.kind {
            out.push(props);
        }
        for child in &element.children {
            collect_text_props(child, out);
        }
    }

    #[test]
    fn authoring_parity_blend_slider_uses_formatter_percent_without_extra_suffix() {
        let presentation = authoring_parity_blend_presentation();
        let format = presentation.format();
        assert_eq!(format(0.75).as_ref(), "75%");
        assert!(presentation.chrome_suffix().is_none());

        let declarative = authoring_parity_blend_slider_options(
            "authoring-parity.declarative.slider",
            "imui-editor-proof.authoring.declarative.blend",
        );
        assert!(declarative.suffix.is_none());
        assert_eq!(
            declarative.id_source.as_deref(),
            Some("authoring-parity.declarative.slider")
        );
        assert_eq!(
            declarative.test_id.as_deref(),
            Some("imui-editor-proof.authoring.declarative.blend")
        );

        let imui = authoring_parity_blend_slider_options(
            "authoring-parity.imui.slider",
            "imui-editor-proof.authoring.imui.blend",
        );
        assert!(imui.suffix.is_none());
        assert_eq!(
            imui.id_source.as_deref(),
            Some("authoring-parity.imui.slider")
        );
        assert_eq!(
            imui.test_id.as_deref(),
            Some("imui-editor-proof.authoring.imui.blend")
        );
    }

    #[test]
    fn authoring_parity_value_options_preserve_presentation_chrome() {
        let presentation = authoring_parity_value_presentation();
        let drag = authoring_parity_drag_value_options(
            &presentation,
            "authoring-parity.declarative.drag-value",
            "imui-editor-proof.authoring.declarative.value",
        );
        assert_eq!(drag.prefix.as_deref(), Some("$"));
        assert_eq!(drag.suffix.as_deref(), Some("ms"));
        assert_eq!(
            drag.id_source.as_deref(),
            Some("authoring-parity.declarative.drag-value")
        );
        assert_eq!(
            drag.test_id.as_deref(),
            Some("imui-editor-proof.authoring.declarative.value")
        );

        let input = authoring_parity_numeric_input_options(
            &presentation,
            "authoring-parity.declarative.numeric-input",
            "imui-editor-proof.authoring.declarative.numeric",
        );
        assert_eq!(input.prefix.as_deref(), Some("$"));
        assert_eq!(input.suffix.as_deref(), Some("ms"));
        assert_eq!(
            input.id_source.as_deref(),
            Some("authoring-parity.declarative.numeric-input")
        );
        assert_eq!(
            input.test_id.as_deref(),
            Some("imui-editor-proof.authoring.declarative.numeric")
        );
    }

    #[test]
    fn advanced_transform_proof_uses_heterogeneous_numeric_presentations() {
        let position = editor_position_presentation();
        let rotation = editor_rotation_presentation();
        let scale = editor_transform_presentations().scale;

        assert_eq!(position.format()(1.25).as_ref(), "1.250");
        assert_eq!(position.chrome_suffix().map(Arc::as_ref), Some("m"));
        assert_eq!(rotation.format()(90.0).as_ref(), "90°");
        assert!(rotation.chrome_suffix().is_none());
        assert_eq!(scale.format()(1.0).as_ref(), "100%");
        assert!(scale.chrome_suffix().is_none());
    }

    #[test]
    fn committed_line_count_label_tracks_multiline_readout() {
        assert_eq!(
            committed_line_count_label(
                "Multiline TextField (v1)\n- uses TextArea\n- clear affordance\n"
            ),
            "3 lines"
        );
        assert_eq!(committed_line_count_label("Line A\nLine B"), "2 lines");
        assert_eq!(committed_line_count_label("Solo"), "1 line");
        assert_eq!(committed_line_count_label(""), "0 lines");
    }

    #[test]
    fn committed_char_count_label_tracks_password_readout() {
        assert_eq!(committed_char_count_label(""), "0 chars");
        assert_eq!(committed_char_count_label("a"), "1 char");
        assert_eq!(committed_char_count_label("abc"), "3 chars");
    }

    #[test]
    fn edit_session_outcome_labels_separate_state_from_compact_action() {
        assert_eq!(
            edit_session_outcome_label(EditSessionOutcome::Committed),
            "Committed"
        );
        assert_eq!(
            edit_session_outcome_label(EditSessionOutcome::Canceled),
            "Canceled"
        );
        assert_eq!(
            compact_edit_session_outcome_label(EditSessionOutcome::Committed),
            "Commit"
        );
        assert_eq!(
            compact_edit_session_outcome_label(EditSessionOutcome::Canceled),
            "Cancel"
        );
    }

    #[test]
    fn proof_outliner_reorder_moves_item_after_target() {
        let mut items = authoring_parity::outliner_items()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        assert!(reorder_vec_by_key(
            &mut items,
            "camera",
            "cube",
            SortableInsertionSide::After,
            |item| item.id.as_ref(),
        ));
        assert_eq!(
            proof_outliner_order_line(&items),
            "Order: Cube -> Camera -> Key light -> Post FX"
        );
    }

    #[test]
    fn proof_outliner_reorder_moves_item_before_target() {
        let mut items = authoring_parity::outliner_items()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        assert!(reorder_vec_by_key(
            &mut items,
            "post-fx",
            "cube",
            SortableInsertionSide::Before,
            |item| item.id.as_ref(),
        ));
        assert_eq!(
            proof_outliner_order_line(&items),
            "Order: Camera -> Post FX -> Cube -> Key light"
        );
    }

    #[test]
    fn proof_drag_preview_card_uses_single_line_text_roles() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let card = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
            proof_drag_preview_card(
                Arc::from("Material asset"),
                Some(Arc::from("assets/materials/brushed-metal.mat")),
            )
            .into_element(cx)
        });

        let mut text_props = Vec::new();
        collect_text_props(&card, &mut text_props);
        assert_eq!(text_props.len(), 2);
        assert_eq!(text_props[0].text.as_ref(), "Material asset");
        assert_eq!(
            text_props[1].text.as_ref(),
            "assets/materials/brushed-metal.mat"
        );
        for props in text_props {
            assert_eq!(props.wrap, TextWrap::None);
            assert_eq!(props.overflow, TextOverflow::Ellipsis);
        }
    }
}
