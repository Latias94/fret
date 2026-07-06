use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret::advanced::KernelApp;
use fret::advanced::driver::{UiAppDriver, ViewElements};
use fret::advanced::interop::embedded_viewport::{
    self as embedded, EmbeddedViewportUiAppDriverExt as _,
};
use fret::advanced::view::ViewWindowState;
use fret::app::{ElementContextAccess, View};
use fret::imui::{
    UiWriter as _, UiWriterImUiFacadeExt as _, UiWriterUiKitExt as _, imui, imui_build,
    kit::{self, ImUiMultiSelectState},
};
use fret::{AppUi, Defaults, FretApp, Ui, shadcn};
use fret_core::{
    AppWindowId, Color, KeyCode, Modifiers, PanelKind, Point, PointerId, Px, Rect, Size,
};
use fret_docking::{DockSpaceElementOptions, runtime as dock_runtime};
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::{
    FrameId, Model, PlatformCapabilities, TickId, TimerToken, WindowHoverDetectionQuality,
};
use fret_ui::GlobalElementId;
use fret_ui::action::{UiActionHostExt as _, UiFocusActionHost};
use fret_ui::scroll::ScrollHandle;
use fret_ui_editor::imui as editor_imui;
use fret_ui_editor::theme::EditorThemePresetV1;
use fret_ui_kit::IntoUiElement as _;
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
mod editor_advanced;
mod editor_gradient;
mod editor_inspector;
mod editor_material;
mod editor_model_owner;
mod editor_object;
mod editor_state;
mod editor_text_assist;
mod proof_helpers;
mod workbench_shell;

#[cfg(test)]
use editor_advanced::*;
#[cfg(test)]
use editor_gradient::*;
use editor_inspector::*;
#[cfg(test)]
use editor_material::*;
#[cfg(test)]
use editor_object::*;
use editor_state::*;
#[cfg(test)]
use editor_text_assist::*;
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

fn configure_imui_editor_proof_driver(
    driver: UiAppDriver<ViewWindowState<ImUiEditorProofView>>,
) -> UiAppDriver<ViewWindowState<ImUiEditorProofView>> {
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
    Cx: ElementContextAccess<'a, KernelApp>,
{
    let cx = cx.elements();
    let window = cx.window;
    let single = single_window_mode_enabled();
    let proof_layout = selected_proof_layout();
    let editor_review_layout = proof_layout == ImUiEditorProofLayout::EditorReview;
    let dock_test_id = workbench_shell::dock_test_id_for_window(cx.app, window);
    let editor_models = editor_inspector_models(cx);
    let parity_models = authoring_parity::shared_models(cx);

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
                let editor_models_for_inspector = editor_models.clone();
                ui.mount(move |cx| {
                    vec![render_editor_inspector_surface(
                        cx,
                        editor_models_for_inspector.clone(),
                        editor_review_layout,
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
