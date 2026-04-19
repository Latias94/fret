use std::collections::HashMap;
use std::sync::Arc;

use fret::advanced::interop::embedded_viewport as embedded;
use fret::advanced::view::{UiCxDataExt as _, ViewWindowState};
use fret::{Defaults, FretApp, advanced::prelude::*, component::prelude::*, shadcn};
use fret_app::{CreateWindowKind, CreateWindowRequest, WindowRequest};
use fret_core::text::TextOverflow;
use fret_core::{Color, Corners, Edges, PanelKind, Px, TextAlign};
use fret_docking::{
    DockManager, DockPanel, DockPanelFactory, DockPanelFactoryCx, DockPanelRegistryBuilder,
    DockPanelRegistryService, ViewportPanel, runtime as dock_runtime,
};
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::{
    ActivationPolicy, FrameId, Model, PlatformCapabilities, TickId, WindowHoverDetectionQuality,
    WindowRole, WindowStyleRequest,
};
use fret_ui::element::{LayoutStyle, Length, SizeStyle};
use fret_ui_editor::composites::{
    GradientEditor, GradientEditorOptions, GradientStopBinding, InspectorPanel,
    InspectorPanelOptions, InspectorPanelSearchAssistOptions, PropertyGrid, PropertyGroup,
    PropertyRow, PropertyRowReset,
};
use fret_ui_editor::controls::{
    Checkbox, ColorEdit, ColorEditOptions, DragValue, DragValueOutcome,
    EditorTextSelectionBehavior, EnumSelect, EnumSelectItem, EnumSelectOptions, FieldStatus,
    FieldStatusBadge, NumericInput, NumericInputOptions, NumericPresentation, NumericValidateFn,
    NumericValueConstraints, Slider, SliderOptions, TextAssistField, TextAssistFieldOptions,
    TextAssistFieldSurface, TextField, TextFieldBlurBehavior, TextFieldMode, TextFieldOptions,
    TextFieldOutcome, TransformEdit, TransformEditAxisOutcome, TransformEditOptions,
    TransformEditPresentations, TransformEditSection, Vec3Edit, VecEditAxis, VecEditAxisOutcome,
    VecEditOptions,
};
use fret_ui_editor::imui as editor_imui;
use fret_ui_editor::primitives::{EditSessionOutcome, EditorCompactReadoutStyle, EditorTokenKeys};
use fret_ui_editor::theme::EditorThemePresetV1;
use fret_ui_kit::headless::text_assist::{
    TextAssistItem, TextAssistMatch, TextAssistMatchMode, controller_with_active_item_id,
    input_owned_text_assist_expanded,
};
use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;
use fret_ui_kit::recipes::imui_drag_preview::{
    DragPreviewGhostOptions, drag_preview_ghost_with_options,
    publish_cross_window_drag_preview_ghost_with_options, render_cross_window_drag_preview_ghosts,
};
use fret_ui_kit::recipes::imui_sortable::{
    SortableInsertionSide, reorder_vec_by_key, sortable_row,
};

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

fn authoring_parity_blend_slider_options(
    id_source: &'static str,
    test_id: &'static str,
) -> SliderOptions {
    SliderOptions {
        id_source: Some(Arc::from(id_source)),
        test_id: Some(Arc::from(test_id)),
        // The text formatter already renders `%`, so slider chrome should not append another unit.
        suffix: None,
        ..Default::default()
    }
}

fn editor_fixed_decimals_presentation() -> NumericPresentation<f64> {
    NumericPresentation::fixed_decimals(3)
}

fn editor_position_presentation() -> NumericPresentation<f64> {
    editor_fixed_decimals_presentation().with_chrome_suffix("m")
}

fn editor_rotation_presentation() -> NumericPresentation<f64> {
    NumericPresentation::degrees(0)
}

fn editor_percent_presentation() -> NumericPresentation<f64> {
    NumericPresentation::percent_0_1(0)
}

fn editor_transform_presentations() -> TransformEditPresentations {
    TransformEditPresentations::new(
        editor_position_presentation(),
        editor_rotation_presentation(),
        editor_percent_presentation(),
    )
}

fn authoring_parity_value_presentation() -> NumericPresentation<f64> {
    editor_fixed_decimals_presentation()
        .with_chrome_prefix("$")
        .with_chrome_suffix("ms")
}

fn authoring_parity_blend_presentation() -> NumericPresentation<f64> {
    editor_percent_presentation()
}

fn edit_session_outcome_label(outcome: EditSessionOutcome) -> &'static str {
    match outcome {
        EditSessionOutcome::Committed => "Commit",
        EditSessionOutcome::Canceled => "Cancel",
    }
}

fn compact_edit_session_outcome_label(outcome: EditSessionOutcome) -> &'static str {
    match outcome {
        EditSessionOutcome::Committed => "Commit",
        EditSessionOutcome::Canceled => "Cancel",
    }
}

fn vec_edit_axis_label(axis: VecEditAxis) -> &'static str {
    match axis {
        VecEditAxis::X => "X",
        VecEditAxis::Y => "Y",
        VecEditAxis::Z => "Z",
        VecEditAxis::W => "W",
    }
}

fn vec_edit_axis_outcome_label(outcome: VecEditAxisOutcome) -> String {
    format!(
        "{} {}",
        vec_edit_axis_label(outcome.axis),
        edit_session_outcome_label(outcome.outcome)
    )
}

fn transform_edit_section_label(section: TransformEditSection) -> &'static str {
    match section {
        TransformEditSection::Position => "Position",
        TransformEditSection::Rotation => "Rotation",
        TransformEditSection::Scale => "Scale",
    }
}

fn transform_edit_axis_outcome_label(outcome: TransformEditAxisOutcome) -> String {
    format!(
        "{}.{} {}",
        transform_edit_section_label(outcome.section),
        vec_edit_axis_label(outcome.axis),
        edit_session_outcome_label(outcome.outcome)
    )
}

fn proof_optional_outcome_readout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    outcome: String,
    test_id: Arc<str>,
) -> Option<fret_ui::element::AnyElement> {
    let outcome = outcome.trim().to_string();
    if outcome.is_empty() {
        return None;
    }

    Some(proof_compact_readout(cx, outcome, Some(test_id)))
}

fn proof_compact_readout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    readout: String,
    test_id: Option<Arc<str>>,
) -> fret_ui::element::AnyElement {
    let theme = fret_ui::Theme::global(&*cx.app);
    let row_height = theme
        .metric_by_key(EditorTokenKeys::DENSITY_ROW_HEIGHT)
        .unwrap_or(Px(24.0));
    let readout_style = EditorCompactReadoutStyle::resolve(theme, row_height);
    let readout = Arc::<str>::from(readout);

    let mut el = cx.text_props(readout_style.text_props(
        readout.clone(),
        LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        TextAlign::Start,
        TextOverflow::Ellipsis,
    ));

    if let Some(test_id) = test_id {
        el = el.test_id(test_id);
    }

    el.a11y_label(readout)
}

fn color_hex_readout(color: Option<Color>) -> String {
    color
        .map(|color| format!("#{:06X}", color.to_srgb_hex_rgb()))
        .unwrap_or_else(|| "<none>".to_string())
}

fn authoring_parity_theme_diag_lines(cx: &mut AppComponentCx<'_>) -> [String; 2] {
    let theme = fret_ui::Theme::global(&*cx.app);
    let scheme = match theme.color_scheme {
        Some(fret_core::ColorScheme::Dark) => "Dark",
        Some(fret_core::ColorScheme::Light) => "Light",
        None => "Unknown",
    };

    [
        format!(
            "diag theme: scheme={scheme} bg={} card={} input={} secondary={}",
            color_hex_readout(theme.color_by_key("background")),
            color_hex_readout(theme.color_by_key("card")),
            color_hex_readout(theme.color_by_key("input")),
            color_hex_readout(theme.color_by_key("secondary")),
        ),
        format!(
            "diag editor: panel={} field={} popup={} accent={}",
            color_hex_readout(theme.color_by_key(EditorTokenKeys::PROPERTY_PANEL_BG)),
            color_hex_readout(theme.color_by_key(EditorTokenKeys::TEXT_FIELD_BG)),
            color_hex_readout(theme.color_by_key(EditorTokenKeys::POPUP_BG)),
            color_hex_readout(theme.color_by_key(EditorTokenKeys::CHROME_ACCENT)),
        ),
    ]
}

fn committed_line_count_label(text: &str) -> String {
    let lines = text.lines().count();
    let noun = if lines == 1 { "line" } else { "lines" };
    format!("{lines} {noun}")
}

fn committed_char_count_label(text: &str) -> String {
    let chars = text.chars().count();
    let noun = if chars == 1 { "char" } else { "chars" };
    format!("{chars} {noun}")
}

fn editor_text_assist_state_label(
    query: &str,
    dismissed_query: &str,
    visible_count: usize,
) -> String {
    if query.trim().is_empty() {
        return "Collapsed".to_string();
    }

    if visible_count == 0 {
        return "No matches".to_string();
    }

    if !input_owned_text_assist_expanded(query, dismissed_query, visible_count) {
        return "Collapsed".to_string();
    }

    format!("Expanded ({visible_count} matches)")
}

#[derive(Clone)]
struct EditorTextAssistReadout {
    state_label: String,
    active_label: String,
}

#[derive(Clone)]
struct EditorTextFieldReadout {
    committed: String,
    outcome: String,
}

#[derive(Clone)]
struct AuthoringParitySharedStateReadout {
    name_line: String,
    value_line: String,
    numeric_line: String,
    blend_line: String,
    enabled_line: String,
    shading_line: String,
    gradient_line: String,
}

#[derive(Clone)]
struct ProofDragAsset {
    label: Arc<str>,
    path: Arc<str>,
}

#[derive(Clone, PartialEq, Eq)]
struct ProofOutlinerItem {
    id: Arc<str>,
    label: Arc<str>,
}

#[derive(Clone)]
struct ProofOutlinerDragItem {
    id: Arc<str>,
    label: Arc<str>,
}

fn proof_outliner_order_line(items: &[ProofOutlinerItem]) -> String {
    let labels = items
        .iter()
        .map(|item| item.label.as_ref())
        .collect::<Vec<_>>()
        .join(" -> ");
    format!("Order: {labels}")
}

fn proof_outliner_items_snapshot(
    app: &KernelApp,
    model: &Model<Vec<ProofOutlinerItem>>,
) -> Vec<ProofOutlinerItem> {
    app.models()
        .read(model, |items| items.clone())
        .unwrap_or_default()
}

fn proof_outliner_order_line_for_model(
    app: &KernelApp,
    model: &Model<Vec<ProofOutlinerItem>>,
) -> String {
    app.models()
        .read(model, |items| proof_outliner_order_line(items))
        .unwrap_or_else(|_| "Order: unavailable".to_string())
}

fn proof_drag_preview_card<H: UiHost>(
    title: Arc<str>,
    subtitle: Option<Arc<str>>,
) -> impl IntoUiElement<H> + use<H> {
    fret_ui_kit::ui::container_build(move |cx, out| {
        let theme = fret_ui::Theme::global(&*cx.app);
        let mut props = fret_ui::element::ContainerProps::default();
        props.layout.size.width = Length::Auto;
        props.layout.size.height = Length::Auto;
        props.padding = Edges::symmetric(Px(10.0), Px(8.0)).into();
        props.background = Some(theme.color_token("popover"));
        props.border = Edges::all(Px(1.0));
        props.border_color = Some(theme.color_token("border"));
        props.corner_radii = Corners::all(Px(8.0));

        let text = subtitle
            .as_ref()
            .map(|subtitle| format!("{title}\n{subtitle}"))
            .unwrap_or_else(|| title.as_ref().to_string());
        out.push(cx.container(props, move |cx| vec![cx.text(text)]));
    })
}

fn editor_text_assist_readout(
    cx: &mut AppComponentCx<'_>,
    items: Arc<[TextAssistItem]>,
    query_model: &Model<String>,
    dismissed_query_model: &Model<String>,
    active_item_id_model: &Model<Option<Arc<str>>>,
) -> EditorTextAssistReadout {
    let (query, dismissed_query, active_item_id) = cx.data().selector_model_paint(
        (query_model, dismissed_query_model, active_item_id_model),
        |(query, dismissed_query, active_item_id)| (query, dismissed_query, active_item_id),
    );

    let controller = controller_with_active_item_id(
        items.as_ref(),
        &query,
        active_item_id.as_ref(),
        TextAssistMatchMode::Prefix,
        false,
    );
    let visible_count = if query.trim().is_empty() {
        0
    } else {
        controller.visible().len()
    };
    let expanded = input_owned_text_assist_expanded(&query, &dismissed_query, visible_count);

    EditorTextAssistReadout {
        state_label: editor_text_assist_state_label(&query, &dismissed_query, visible_count),
        active_label: if expanded {
            controller
                .active_match()
                .map(|entry| entry.label.as_ref().to_string())
                .unwrap_or_else(|| "None".to_string())
        } else {
            "None".to_string()
        },
    }
}

fn editor_text_field_readout(
    cx: &mut AppComponentCx<'_>,
    committed_model: &Model<String>,
    outcome_model: &Model<String>,
) -> EditorTextFieldReadout {
    cx.data()
        .selector_model_paint((committed_model, outcome_model), |(committed, outcome)| {
            EditorTextFieldReadout { committed, outcome }
        })
}

fn editor_string_model_readout(cx: &mut AppComponentCx<'_>, model: &Model<String>) -> String {
    cx.data().selector_model_paint(model, |value| value)
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
        .dock_op(on_dock_op)
        .window_create_spec(window_create_spec)
        .window_created(window_created)
        .before_close_window(before_close_window)
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
            install_dock_panel_registry(app);
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
    let editor_drag_value_outcome_model = editor_demo_drag_value_outcome_model(cx);
    let editor_roughness_model = editor_demo_roughness_model(cx);
    let editor_metallic_model = editor_demo_metallic_model(cx);
    let editor_alpha_clip_model = editor_demo_alpha_clip_model(cx);
    let editor_cast_shadows_model = editor_demo_cast_shadows_model(cx);
    let editor_shading_model = editor_demo_shading_model(cx);
    let editor_base_color_model = editor_demo_base_color_model(cx);
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
    let parity_name_model = authoring_parity_name_model(cx);
    let parity_drag_value_model = authoring_parity_drag_value_model(cx);
    let parity_numeric_input_model = authoring_parity_numeric_input_model(cx);
    let parity_slider_model = authoring_parity_slider_model(cx);
    let parity_enabled_model = authoring_parity_enabled_model(cx);
    let parity_shading_model = authoring_parity_shading_model(cx);
    let parity_gradient_angle_model = authoring_parity_gradient_angle_model(cx);
    let parity_gradient_stops_model = authoring_parity_gradient_stops_model(cx);
    let parity_gradient_next_id_model = authoring_parity_gradient_next_id_model(cx);

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

        let root_content = fret_ui_kit::ui::v_flex_build(move |cx, out| {
            fret_imui::imui_build(cx, out, |ui| {
                if !editor_review_layout {
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

                    let controls = fret_ui_kit::ui::h_flex_build(move |cx, out| {
                        fret_imui::imui_build(cx, out, |ui| {
                            let reset = <_ as fret_ui_kit::imui::UiWriterImUiFacadeExt<KernelApp>>::button(
                                ui,
                                "Reset layout",
                            );
                            let _ = ui.tooltip_text_with_options(
                                "imui-editor-proof.controls.reset-layout.tooltip",
                                reset,
                                "Restore the canonical dock graph for this proof window.",
                                fret_ui_kit::imui::TooltipOptions {
                                    open_delay_frames_override: Some(0),
                                    close_delay_frames_override: Some(0),
                                    test_id: Some(Arc::from(
                                        "imui-editor-proof.controls.reset-layout.tooltip",
                                    )),
                                    ..Default::default()
                                },
                            );
                            if reset.clicked() {
                                reset_dock_graph(ui.cx_mut().app, window);
                                dock_runtime::request_dock_invalidation(ui.cx_mut().app, [window]);
                            }
                            let recenter =
                                <_ as fret_ui_kit::imui::UiWriterImUiFacadeExt<KernelApp>>::button(
                                ui,
                                "Center floatings",
                            );
                            let _ = ui.tooltip_text_with_options(
                                "imui-editor-proof.controls.center-floatings.tooltip",
                                recenter,
                                "Recenter in-window floating panels without resetting content state.",
                                fret_ui_kit::imui::TooltipOptions {
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

                    let parity_intro = fret_ui_kit::ui::text(
                        "authoring parity proof: shared models, left declarative, right imui adapters; compare drag scrub, typed numeric entry, and bounded slider surfaces, then verify each paired row stays in sync under the same preset",
                    )
                    .text_xs();
                    ui.add_ui(parity_intro);

                    let parity_name_model_for_surface = parity_name_model.clone();
                    let parity_drag_value_model_for_surface = parity_drag_value_model.clone();
                    let parity_numeric_input_model_for_surface =
                        parity_numeric_input_model.clone();
                    let parity_slider_model_for_surface = parity_slider_model.clone();
                    let parity_enabled_model_for_surface = parity_enabled_model.clone();
                    let parity_shading_model_for_surface = parity_shading_model.clone();
                    let parity_gradient_angle_model_for_surface =
                        parity_gradient_angle_model.clone();
                    let parity_gradient_stops_model_for_surface =
                        parity_gradient_stops_model.clone();
                    let parity_gradient_next_id_model_for_surface =
                        parity_gradient_next_id_model.clone();
                    ui.mount(move |cx| {
                        vec![render_authoring_parity_surface(
                            cx,
                            parity_name_model_for_surface.clone(),
                            parity_drag_value_model_for_surface.clone(),
                            parity_numeric_input_model_for_surface.clone(),
                            parity_slider_model_for_surface.clone(),
                            parity_enabled_model_for_surface.clone(),
                            parity_shading_model_for_surface.clone(),
                            parity_gradient_angle_model_for_surface.clone(),
                            parity_gradient_stops_model_for_surface.clone(),
                            parity_gradient_next_id_model_for_surface.clone(),
                        )
                        .into_element(cx)]
                    });

                    let parity_state_hint =
                        fret_ui_kit::ui::text(
                            "shared state readout: each declarative/imui pair should mutate the same model, while drag, typed numeric, and slider stay intentionally distinct",
                        )
                        .text_xs();
                    ui.add_ui(parity_state_hint);

                    let parity_name_model_for_state = parity_name_model.clone();
                    let parity_drag_value_model_for_state = parity_drag_value_model.clone();
                    let parity_numeric_input_model_for_state =
                        parity_numeric_input_model.clone();
                    let parity_slider_model_for_state = parity_slider_model.clone();
                    let parity_enabled_model_for_state = parity_enabled_model.clone();
                    let parity_shading_model_for_state = parity_shading_model.clone();
                    let parity_gradient_angle_model_for_state = parity_gradient_angle_model.clone();
                    let parity_gradient_stops_model_for_state = parity_gradient_stops_model.clone();
                    ui.mount(move |cx| {
                        vec![render_authoring_parity_shared_state(
                            cx,
                            parity_name_model_for_state.clone(),
                            parity_drag_value_model_for_state.clone(),
                            parity_numeric_input_model_for_state.clone(),
                            parity_slider_model_for_state.clone(),
                            parity_enabled_model_for_state.clone(),
                            parity_shading_model_for_state.clone(),
                            parity_gradient_angle_model_for_state.clone(),
                            parity_gradient_stops_model_for_state.clone(),
                        )
                        .into_element(cx)]
                    });
                    ui.separator();

                    let editor_label =
                        fret_ui_kit::ui::text("fret-ui-editor (M2): PropertyGroup + PropertyGrid + search assist")
                            .text_xs();
                    ui.add_ui(editor_label);
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
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Inline rename"),
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
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Rename committed"),
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
                                                            PropertyRow::new().options(
                                                                row_cx.row_options.clone(),
                                                            ),
                                                            |cx| cx.text("Rename outcome"),
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
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Buffered name"),
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
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Password"),
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
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Secret length"),
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
                                                            PropertyRow::new().options(
                                                                row_cx.row_options.clone(),
                                                            ),
                                                            |cx| cx.text("Password outcome"),
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
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Committed"),
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
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Name assist"),
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
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Assist state"),
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
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Active assist"),
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
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Accepted assist"),
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
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Notes"),
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
                                                        PropertyRow::new().options(
                                                            row_cx.row_options.clone(),
                                                        ),
                                                        |cx| cx.text("Notes committed"),
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
                                                            PropertyRow::new().options(
                                                                row_cx.row_options.clone(),
                                                            ),
                                                            |cx| cx.text("Notes outcome"),
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
                                                                .options(
                                                                    row_cx.row_options.clone(),
                                                                )
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
                                                            |cx| cx.text("Opacity"),
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
                                                                .options(
                                                                    row_cx.row_options.clone(),
                                                                )
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
                                                            |cx| cx.text("Roughness"),
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
                                                                .options(
                                                                    row_cx.row_options.clone(),
                                                                )
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
                                                            |cx| cx.text("Metallic"),
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
                                                        let items = editor_material_shading_items();

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
                                                                .options(
                                                                    row_cx.row_options.clone(),
                                                                )
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
                                                            |cx| cx.text("Position"),
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
                                                                ),
                                                        );
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

                                                        rows.push(
                                                            PropertyRow::new()
                                                                .options(row_cx.row_options.clone())
                                                                .reset(Some(
                                                                    PropertyRowReset::new(on_reset).options(
                                                                        fret_ui_editor::composites::PropertyRowResetOptions {
                                                                            test_id: Some(Arc::from("imui-editor-proof.editor.advanced.exposure.reset")),
                                                                            ..Default::default()
                                                                        },
                                                                    ),
                                                                ))
                                                                .into_element(
                                                                    cx,
                                                                    |cx| cx.text("Exposure"),
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
                if !editor_review_layout {
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

fn render_authoring_parity_surface(
    cx: &mut AppComponentCx<'_>,
    name_model: Model<String>,
    drag_value_model: Model<f64>,
    numeric_input_model: Model<f64>,
    slider_model: Model<f64>,
    enabled_model: Model<bool>,
    shading_model: Model<Option<Arc<str>>>,
    gradient_angle_model: Model<f64>,
    gradient_stops_model: Model<Vec<GradientDemoStop>>,
    gradient_next_id_model: Model<u64>,
) -> impl IntoUiElement<KernelApp> + use<> {
    let shading_items = authoring_parity_shading_items();

    fret_ui_kit::ui::v_flex_build(move |cx, out| {
        if diag_enabled() {
            let [theme_line, editor_line] = authoring_parity_theme_diag_lines(cx);
            out.push(proof_compact_readout(
                cx,
                theme_line,
                Some(Arc::from("imui-editor-proof.authoring.diag.theme")),
            ));
            out.push(proof_compact_readout(
                cx,
                editor_line,
                Some(Arc::from("imui-editor-proof.authoring.diag.editor")),
            ));
        }

        out.push(
            fret_ui_kit::ui::h_flex_build(move |cx, out| {
                out.push(
                    fret_ui_kit::ui::container_build({
                        let shading_items = shading_items.clone();
                        let name_model = name_model.clone();
                        let drag_value_model = drag_value_model.clone();
                        let numeric_input_model = numeric_input_model.clone();
                        let slider_model = slider_model.clone();
                        let enabled_model = enabled_model.clone();
                        let shading_model = shading_model.clone();
                        let gradient_angle_model = gradient_angle_model.clone();
                        let gradient_stops_model = gradient_stops_model.clone();
                        let gradient_next_id_model = gradient_next_id_model.clone();
                        move |cx, out| {
                            out.push(
                                render_authoring_parity_declarative_group(
                                    cx,
                                    name_model,
                                    drag_value_model,
                                    numeric_input_model,
                                    slider_model,
                                    enabled_model,
                                    shading_model,
                                    gradient_angle_model,
                                    gradient_stops_model,
                                    gradient_next_id_model,
                                    shading_items,
                                )
                                .into_element(cx),
                            );
                        }
                    })
                    .basis_0()
                    .flex_1()
                    .into_element(cx),
                );

                out.push(
                    fret_ui_kit::ui::container_build(move |cx, out| {
                        out.push(
                            render_authoring_parity_imui_group(
                                cx,
                                name_model,
                                drag_value_model,
                                numeric_input_model,
                                slider_model,
                                enabled_model,
                                shading_model,
                                gradient_angle_model,
                                gradient_stops_model,
                                gradient_next_id_model,
                                shading_items,
                            )
                            .into_element(cx),
                        );
                    })
                    .basis_0()
                    .flex_1()
                    .into_element(cx),
                );
            })
            .gap(fret_ui_kit::Space::N3)
            .into_element(cx),
        );
    })
    .gap(fret_ui_kit::Space::N2)
    .into_element(cx)
}

fn render_authoring_parity_shared_state(
    cx: &mut AppComponentCx<'_>,
    name_model: Model<String>,
    drag_value_model: Model<f64>,
    numeric_input_model: Model<f64>,
    slider_model: Model<f64>,
    enabled_model: Model<bool>,
    shading_model: Model<Option<Arc<str>>>,
    gradient_angle_model: Model<f64>,
    gradient_stops_model: Model<Vec<GradientDemoStop>>,
) -> impl IntoUiElement<KernelApp> + use<> {
    let shared = cx.data().selector_model_paint(
        (
            &name_model,
            &drag_value_model,
            &numeric_input_model,
            &slider_model,
            &enabled_model,
            &shading_model,
            &gradient_angle_model,
            &gradient_stops_model,
        ),
        |(name, value, numeric, blend, enabled, shading, gradient_angle, gradient_stops)| {
            AuthoringParitySharedStateReadout {
                name_line: if name.trim().is_empty() {
                    "shared name: <empty>".to_string()
                } else {
                    format!("shared name: {name}")
                },
                value_line: format!("shared value: {value:.3}"),
                numeric_line: format!("shared typed numeric: {numeric:.3}"),
                blend_line: format!("shared blend: {:.0}%", blend * 100.0),
                enabled_line: format!("shared enabled: {enabled}"),
                shading_line: match shading.as_deref() {
                    Some("lit") => "shared mode: lit (Lit)".to_string(),
                    Some("unlit") => "shared mode: unlit (Unlit)".to_string(),
                    Some("matcap") => "shared mode: matcap (Matcap)".to_string(),
                    Some(other) => format!("shared mode: {other}"),
                    None => "shared mode: <none>".to_string(),
                },
                gradient_line: format!(
                    "shared gradient: {} stops @ {:.0}°",
                    gradient_stops.len(),
                    gradient_angle
                ),
            }
        },
    );
    let name_line = shared.name_line;
    let value_line = shared.value_line;
    let numeric_line = shared.numeric_line;
    let blend_line = shared.blend_line;
    let enabled_line = shared.enabled_line;
    let shading_line = shared.shading_line;
    let gradient_line = shared.gradient_line;

    fret_ui_kit::ui::v_flex_build(move |cx, out| {
        let name_line_row = name_line.clone();
        let value_line_row = value_line.clone();
        let numeric_line_row = numeric_line.clone();
        out.push(
            fret_ui_kit::ui::h_flex_build(move |cx, out| {
                out.push(
                    cx.text(name_line_row)
                        .test_id("imui-editor-proof.authoring.shared.name"),
                );
                out.push(
                    cx.text(value_line_row)
                        .test_id("imui-editor-proof.authoring.shared.value"),
                );
                out.push(
                    cx.text(numeric_line_row)
                        .test_id("imui-editor-proof.authoring.shared.numeric"),
                );
            })
            .gap(fret_ui_kit::Space::N3)
            .into_element(cx),
        );
        out.push(
            fret_ui_kit::ui::h_flex_build(move |cx, out| {
                out.push(
                    cx.text(blend_line)
                        .test_id("imui-editor-proof.authoring.shared.blend"),
                );
                out.push(
                    cx.text(enabled_line)
                        .test_id("imui-editor-proof.authoring.shared.enabled"),
                );
                out.push(
                    cx.text(shading_line)
                        .test_id("imui-editor-proof.authoring.shared.mode"),
                );
            })
            .gap(fret_ui_kit::Space::N3)
            .into_element(cx),
        );
        out.push(
            cx.text(gradient_line)
                .test_id("imui-editor-proof.authoring.shared.gradient"),
        );
    })
    .gap(fret_ui_kit::Space::N1)
    .into_element(cx)
}

fn render_authoring_parity_declarative_group(
    cx: &mut AppComponentCx<'_>,
    name_model: Model<String>,
    drag_value_model: Model<f64>,
    numeric_input_model: Model<f64>,
    slider_model: Model<f64>,
    enabled_model: Model<bool>,
    shading_model: Model<Option<Arc<str>>>,
    gradient_angle_model: Model<f64>,
    gradient_stops_model: Model<Vec<GradientDemoStop>>,
    gradient_next_id_model: Model<u64>,
    shading_items: Arc<[EnumSelectItem]>,
) -> impl IntoUiElement<KernelApp> + use<> {
    let value_presentation = authoring_parity_value_presentation();
    let blend_presentation = authoring_parity_blend_presentation();

    PropertyGroup::new("Declarative authoring")
        .options(fret_ui_editor::composites::PropertyGroupOptions {
            test_id: Some(Arc::from("imui-editor-proof.authoring.declarative.group")),
            header_test_id: Some(Arc::from(
                "imui-editor-proof.authoring.declarative.group.header",
            )),
            content_test_id: Some(Arc::from(
                "imui-editor-proof.authoring.declarative.group.content",
            )),
            ..Default::default()
        })
        .into_element(
            cx,
            |_cx| None,
            move |cx| {
                vec![
                    PropertyGrid::new().into_element(cx, move |cx, row_cx| {
                        let mut rows = Vec::new();

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new().options(row_cx.row_options.clone()),
                            |cx| cx.text("Name"),
                            |cx| {
                                TextField::new(name_model.clone())
                                    .options(TextFieldOptions {
                                        clear_button: true,
                                        selection_behavior:
                                            EditorTextSelectionBehavior::SelectAllOnFocus,
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.declarative.name",
                                        )),
                                        clear_test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.declarative.name.clear",
                                        )),
                                        ..Default::default()
                                    })
                                    .into_element(cx)
                            },
                            |_cx| None,
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new().options(row_cx.row_options.clone()),
                            |cx| cx.text("Drag value"),
                            |cx| {
                                DragValue::from_presentation(
                                    drag_value_model.clone(),
                                    value_presentation.clone(),
                                )
                                .options(fret_ui_editor::controls::DragValueOptions {
                                    id_source: Some(Arc::from(
                                        "authoring-parity.declarative.drag-value",
                                    )),
                                    test_id: Some(Arc::from(
                                        "imui-editor-proof.authoring.declarative.value",
                                    )),
                                    ..Default::default()
                                })
                                .into_element(cx)
                            },
                            |_cx| None,
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new().options(row_cx.row_options.clone()),
                            |cx| cx.text("Typed numeric"),
                            |cx| {
                                NumericInput::from_presentation(
                                    numeric_input_model.clone(),
                                    value_presentation.clone(),
                                )
                                .options(NumericInputOptions {
                                    id_source: Some(Arc::from(
                                        "authoring-parity.declarative.numeric-input",
                                    )),
                                    test_id: Some(Arc::from(
                                        "imui-editor-proof.authoring.declarative.numeric",
                                    )),
                                    ..Default::default()
                                })
                                .into_element(cx)
                            },
                            |_cx| None,
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new().options(row_cx.row_options.clone()),
                            |cx| cx.text("Blend slider"),
                            |cx| {
                                Slider::from_presentation(
                                    slider_model.clone(),
                                    0.0,
                                    1.0,
                                    blend_presentation.clone(),
                                )
                                .options(authoring_parity_blend_slider_options(
                                    "authoring-parity.declarative.slider",
                                    "imui-editor-proof.authoring.declarative.blend",
                                ))
                                .into_element(cx)
                            },
                            |cx| {
                                Some(
                                    FieldStatusBadge::new(FieldStatus::Dirty)
                                        .into_element(cx)
                                        .test_id(
                                            "imui-editor-proof.authoring.declarative.blend.status",
                                        ),
                                )
                            },
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new().options(row_cx.row_options.clone()),
                            |cx| cx.text("Enabled"),
                            |cx| {
                                Checkbox::new(enabled_model.clone())
                                    .options(fret_ui_editor::controls::CheckboxOptions {
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.declarative.enabled",
                                        )),
                                        ..Default::default()
                                    })
                                    .into_element(cx)
                            },
                            |_cx| None,
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new().options(row_cx.row_options.clone()),
                            |cx| cx.text("Mode"),
                            |cx| {
                                EnumSelect::new(shading_model.clone(), shading_items.clone())
                                    .options(EnumSelectOptions {
                                        id_source: Some(Arc::from(
                                            "authoring-parity.declarative.mode",
                                        )),
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.declarative.mode",
                                        )),
                                        list_test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.declarative.mode.list",
                                        )),
                                        search_test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.declarative.mode.search",
                                        )),
                                        ..Default::default()
                                    })
                                    .into_element(cx)
                            },
                            |_cx| None,
                        ));

                        rows
                    }),
                    cx.text("Gradient editor")
                        .test_id("imui-editor-proof.authoring.declarative.gradient.label"),
                    build_authoring_parity_gradient_editor(
                        cx,
                        gradient_angle_model.clone(),
                        gradient_stops_model.clone(),
                        gradient_next_id_model.clone(),
                        "authoring-parity.declarative.gradient",
                        "imui-editor-proof.authoring.declarative.gradient",
                    )
                    .into_element(cx),
                ]
            },
        )
}

fn render_authoring_parity_imui_group(
    cx: &mut AppComponentCx<'_>,
    name_model: Model<String>,
    drag_value_model: Model<f64>,
    numeric_input_model: Model<f64>,
    slider_model: Model<f64>,
    enabled_model: Model<bool>,
    shading_model: Model<Option<Arc<str>>>,
    gradient_angle_model: Model<f64>,
    gradient_stops_model: Model<Vec<GradientDemoStop>>,
    gradient_next_id_model: Model<u64>,
    shading_items: Arc<[EnumSelectItem]>,
) -> impl IntoUiElement<KernelApp> + use<> {
    let value_presentation = authoring_parity_value_presentation();
    let blend_presentation = authoring_parity_blend_presentation();

    render_authoring_parity_imui_host(cx, move |ui| {
        editor_imui::property_group(
            ui,
            PropertyGroup::new("imui authoring").options(
                fret_ui_editor::composites::PropertyGroupOptions {
                    test_id: Some(Arc::from("imui-editor-proof.authoring.imui.group")),
                    header_test_id: Some(Arc::from(
                        "imui-editor-proof.authoring.imui.group.header",
                    )),
                    content_test_id: Some(Arc::from(
                        "imui-editor-proof.authoring.imui.group.content",
                    )),
                    ..Default::default()
                },
            ),
            |_cx| None,
            move |cx| {
                let mut out = Vec::new();
                fret_imui::imui_build(cx, &mut out, move |ui| {
                    editor_imui::property_grid(ui, PropertyGrid::new(), move |cx, row_cx| {
                        let mut rows = Vec::new();

                        rows.push(row_cx.row(
                            cx,
                            |cx| cx.text("Name"),
                            |cx| {
                                render_authoring_parity_imui_host(cx, move |ui| {
                                    editor_imui::text_field(
                                        ui,
                                        TextField::new(name_model.clone()).options(
                                            TextFieldOptions {
                                                clear_button: true,
                                                selection_behavior:
                                                    EditorTextSelectionBehavior::SelectAllOnFocus,
                                                test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.name",
                                                )),
                                                clear_test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.name.clear",
                                                )),
                                                ..Default::default()
                                            },
                                        ),
                                    );
                                })
                                .into_element(cx)
                            },
                        ));

                        rows.push(row_cx.row(
                            cx,
                            |cx| cx.text("Drag value"),
                            |cx| {
                                let value_presentation = value_presentation.clone();
                                render_authoring_parity_imui_host(cx, move |ui| {
                                    editor_imui::drag_value(
                                        ui,
                                        DragValue::from_presentation(
                                            drag_value_model.clone(),
                                            value_presentation.clone(),
                                        )
                                        .options(
                                            fret_ui_editor::controls::DragValueOptions {
                                                id_source: Some(Arc::from(
                                                    "authoring-parity.imui.drag-value",
                                                )),
                                                test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.value",
                                                )),
                                                ..Default::default()
                                            },
                                        ),
                                    );
                                })
                                .into_element(cx)
                            },
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new().options(row_cx.row_options.clone()),
                            |cx| cx.text("Typed numeric"),
                            |cx| {
                                let value_presentation = value_presentation.clone();
                                render_authoring_parity_imui_host(cx, move |ui| {
                                    editor_imui::numeric_input(
                                        ui,
                                        NumericInput::from_presentation(
                                            numeric_input_model.clone(),
                                            value_presentation.clone(),
                                        )
                                        .options(
                                            NumericInputOptions {
                                                id_source: Some(Arc::from(
                                                    "authoring-parity.imui.numeric-input",
                                                )),
                                                test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.numeric",
                                                )),
                                                ..Default::default()
                                            },
                                        ),
                                    );
                                })
                                .into_element(cx)
                            },
                            |_cx| None,
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new().options(row_cx.row_options.clone()),
                            |cx| cx.text("Blend slider"),
                            |cx| {
                                let blend_presentation = blend_presentation.clone();
                                render_authoring_parity_imui_host(cx, move |ui| {
                                    editor_imui::slider(
                                        ui,
                                        Slider::from_presentation(
                                            slider_model.clone(),
                                            0.0,
                                            1.0,
                                            blend_presentation.clone(),
                                        )
                                        .options(
                                            authoring_parity_blend_slider_options(
                                                "authoring-parity.imui.slider",
                                                "imui-editor-proof.authoring.imui.blend",
                                            ),
                                        ),
                                    );
                                })
                                .into_element(cx)
                            },
                            |cx| {
                                Some(
                                    render_authoring_parity_imui_host(cx, move |ui| {
                                        editor_imui::field_status_badge(
                                            ui,
                                            FieldStatusBadge::new(FieldStatus::Dirty),
                                        );
                                    })
                                    .into_element(cx)
                                    .test_id("imui-editor-proof.authoring.imui.blend.status"),
                                )
                            },
                        ));

                        rows.push(row_cx.row(
                            cx,
                            |cx| cx.text("Enabled"),
                            |cx| {
                                render_authoring_parity_imui_host(cx, move |ui| {
                                    editor_imui::checkbox(
                                        ui,
                                        Checkbox::new(enabled_model.clone()).options(
                                            fret_ui_editor::controls::CheckboxOptions {
                                                test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.enabled",
                                                )),
                                                ..Default::default()
                                            },
                                        ),
                                    );
                                })
                                .into_element(cx)
                            },
                        ));

                        rows.push(row_cx.row(
                            cx,
                            |cx| cx.text("Mode"),
                            |cx| {
                                render_authoring_parity_imui_host(cx, move |ui| {
                                    editor_imui::enum_select(
                                        ui,
                                        EnumSelect::new(
                                            shading_model.clone(),
                                            shading_items.clone(),
                                        )
                                        .options(
                                            EnumSelectOptions {
                                                id_source: Some(Arc::from(
                                                    "authoring-parity.imui.mode",
                                                )),
                                                test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.mode",
                                                )),
                                                list_test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.mode.list",
                                                )),
                                                search_test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.mode.search",
                                                )),
                                                ..Default::default()
                                            },
                                        ),
                                    );
                                })
                                .into_element(cx)
                            },
                        ));

                        rows
                    });

                    ui.text("Gradient editor");
                    let gradient_editor = build_authoring_parity_gradient_editor(
                        ui.cx_mut(),
                        gradient_angle_model.clone(),
                        gradient_stops_model.clone(),
                        gradient_next_id_model.clone(),
                        "authoring-parity.imui.gradient",
                        "imui-editor-proof.authoring.imui.gradient",
                    );
                    editor_imui::gradient_editor(ui, gradient_editor);
                });
                out
            },
        );

        ui.separator();
        ui.text("Generic tree/collapsing helpers");
        let _ = ui.collapsing_header_with_options(
            "imui-editor-proof.authoring.imui.outliner.section",
            "Scene outliner",
            fret_ui_kit::imui::CollapsingHeaderOptions {
                default_open: true,
                test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.outliner.section",
                )),
                header_test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.outliner.section.header",
                )),
                content_test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.outliner.section.content",
                )),
                ..Default::default()
            },
            |ui| {
                let _ = ui.tree_node_with_options(
                    "imui-editor-proof.authoring.imui.outliner.scene",
                    "Scene",
                    fret_ui_kit::imui::TreeNodeOptions {
                        default_open: true,
                        test_id: Some(Arc::from(
                            "imui-editor-proof.authoring.imui.outliner.scene",
                        )),
                        content_test_id: Some(Arc::from(
                            "imui-editor-proof.authoring.imui.outliner.scene.content",
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.tree_node_with_options(
                            "imui-editor-proof.authoring.imui.outliner.scene.camera",
                            "Camera",
                            fret_ui_kit::imui::TreeNodeOptions {
                                leaf: true,
                                level: 2,
                                selected: true,
                                test_id: Some(Arc::from(
                                    "imui-editor-proof.authoring.imui.outliner.scene.camera",
                                )),
                                ..Default::default()
                            },
                            |_ui| {},
                        );
                        let _ = ui.tree_node_with_options(
                            "imui-editor-proof.authoring.imui.outliner.scene.geometry",
                            "Geometry",
                            fret_ui_kit::imui::TreeNodeOptions {
                                default_open: true,
                                level: 2,
                                test_id: Some(Arc::from(
                                    "imui-editor-proof.authoring.imui.outliner.scene.geometry",
                                )),
                                content_test_id: Some(Arc::from(
                                    "imui-editor-proof.authoring.imui.outliner.scene.geometry.content",
                                )),
                                ..Default::default()
                            },
                            |ui| {
                                let _ = ui.tree_node_with_options(
                                    "imui-editor-proof.authoring.imui.outliner.scene.geometry.cube",
                                    "Cube",
                                    fret_ui_kit::imui::TreeNodeOptions {
                                        leaf: true,
                                        level: 3,
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.imui.outliner.scene.geometry.cube",
                                        )),
                                        ..Default::default()
                                    },
                                    |_ui| {},
                                );
                                let _ = ui.tree_node_with_options(
                                    "imui-editor-proof.authoring.imui.outliner.scene.geometry.key-light",
                                    "Key light",
                                    fret_ui_kit::imui::TreeNodeOptions {
                                        leaf: true,
                                        level: 3,
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.imui.outliner.scene.geometry.key-light",
                                        )),
                                        ..Default::default()
                                    },
                                    |_ui| {},
                                );
                            },
                        );
                        let _ = ui.tree_node_with_options(
                            "imui-editor-proof.authoring.imui.outliner.scene.postfx",
                            "Post FX",
                            fret_ui_kit::imui::TreeNodeOptions {
                                leaf: true,
                                level: 2,
                                test_id: Some(Arc::from(
                                    "imui-editor-proof.authoring.imui.outliner.scene.postfx",
                                )),
                                ..Default::default()
                            },
                            |_ui| {},
                        );
                    },
                );
            },
        );

        ui.separator();
        ui.text("Typed drag/drop helpers");
        ui.text("Drag an asset chip onto the material slot. Payload and preview stay app-defined.");

        let asset_slot_model = authoring_parity_asset_slot_model(ui.cx_mut());
        let asset_chips = authoring_parity_drag_assets();

        ui.horizontal(|ui| {
            for (ix, asset) in asset_chips.iter().enumerate() {
                let trigger = ui.button_with_options(
                    asset.label.clone(),
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from(format!(
                            "imui-editor-proof.authoring.imui.drag-drop.asset.{ix}"
                        ))),
                        ..Default::default()
                    },
                );
                let source = ui.drag_source_with_options(
                    trigger,
                    asset.clone(),
                    fret_ui_kit::imui::DragSourceOptions {
                        cross_window: true,
                        ..Default::default()
                    },
                );
                let ghost_id =
                    format!("imui-editor-proof.authoring.imui.drag-drop.asset.{ix}.ghost");
                let _ = publish_cross_window_drag_preview_ghost_with_options(
                    ui,
                    ghost_id.as_str(),
                    source,
                    DragPreviewGhostOptions {
                        test_id: Some(Arc::from(format!(
                            "imui-editor-proof.authoring.imui.drag-drop.asset.{ix}.ghost"
                        ))),
                        ..Default::default()
                    },
                    {
                        let label = asset.label.clone();
                        let path = asset.path.clone();
                        move |_cx| proof_drag_preview_card(label.clone(), Some(path.clone()))
                    },
                );
            }
        });

        let assigned_asset = editor_string_model_readout(ui.cx_mut(), &asset_slot_model);
        let slot_trigger = ui.button_with_options(
            format!("Base color slot: {assigned_asset}"),
            fret_ui_kit::imui::ButtonOptions {
                test_id: Some(Arc::from("imui-editor-proof.authoring.imui.drag-drop.slot")),
                ..Default::default()
            },
        );
        let slot_drop = ui.drop_target::<ProofDragAsset>(slot_trigger);
        if let Some(payload) = slot_drop.delivered_payload() {
            let delivered = payload.path.as_ref().to_string();
            let cx = ui.cx_mut();
            let _ = cx
                .app
                .models_mut()
                .update(&asset_slot_model, |value: &mut String| {
                    value.clear();
                    value.push_str(delivered.as_str());
                });
        }

        let drag_drop_status = if let Some(payload) = slot_drop.delivered_payload() {
            format!("Delivered {}", payload.path)
        } else if let Some(payload) = slot_drop.preview_payload() {
            format!("Preview {}", payload.path)
        } else if slot_drop.active() {
            "Compatible drag active".to_string()
        } else {
            "Idle".to_string()
        };
        ui.text(drag_drop_status);

        ui.separator();
        ui.text("Reorderable outliner proof");
        ui.text(
            "Sortable math stays app-owned. `imui` only provides typed payloads + drop positions.",
        );

        let outliner_items_model = authoring_parity_outliner_items_model(ui.cx_mut());
        let outliner_status_model = authoring_parity_outliner_status_model(ui.cx_mut());
        let outliner_items = proof_outliner_items_snapshot(ui.cx_mut().app, &outliner_items_model);
        let mut pending_reorder: Option<(
            Arc<str>,
            Arc<str>,
            Arc<str>,
            Arc<str>,
            SortableInsertionSide,
        )> = None;
        let mut preview_status: Option<String> = None;

        let _ = ui.tree_node_with_options(
            "imui-editor-proof.authoring.imui.outliner.reorder.scene",
            "Scene",
            fret_ui_kit::imui::TreeNodeOptions {
                default_open: true,
                test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.outliner.reorder.scene",
                )),
                content_test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.outliner.reorder.scene.content",
                )),
                ..Default::default()
            },
            |ui| {
                for item in &outliner_items {
                    let row = ui.tree_node_with_options(
                        item.id.as_ref(),
                        item.label.clone(),
                        fret_ui_kit::imui::TreeNodeOptions {
                            leaf: true,
                            level: 2,
                            test_id: Some(Arc::from(format!(
                                "imui-editor-proof.authoring.imui.outliner.reorder.row.{}",
                                item.id
                            ))),
                            ..Default::default()
                        },
                        |_ui| {},
                    );

                    let payload = ProofOutlinerDragItem {
                        id: item.id.clone(),
                        label: item.label.clone(),
                    };
                    let sortable = sortable_row(ui, row.trigger, payload);
                    let ghost_id = format!(
                        "imui-editor-proof.authoring.imui.outliner.reorder.row.{}.ghost",
                        item.id
                    );
                    let _ = drag_preview_ghost_with_options(
                        ui,
                        ghost_id.as_str(),
                        sortable.source(),
                        DragPreviewGhostOptions {
                            test_id: Some(Arc::from(format!(
                                "imui-editor-proof.authoring.imui.outliner.reorder.row.{}.ghost",
                                item.id
                            ))),
                            ..Default::default()
                        },
                        proof_drag_preview_card(item.label.clone(), None),
                    );

                    if let Some(signal) = sortable.delivered_reorder() {
                        let dragged = signal.payload();
                        if dragged.id != item.id {
                            pending_reorder = Some((
                                dragged.id.clone(),
                                dragged.label.clone(),
                                item.id.clone(),
                                item.label.clone(),
                                signal.side(),
                            ));
                        }
                    } else if let Some(signal) = sortable.preview_reorder() {
                        let dragged = signal.payload();
                        let side = signal.side();
                        if dragged.id != item.id {
                            preview_status = Some(format!(
                                "Preview: move {} {} {}",
                                dragged.label,
                                side.label(),
                                item.label
                            ));
                        }
                    }
                }
            },
        );

        if let Some((active_id, active_label, over_id, over_label, side)) = pending_reorder {
            let moved = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&outliner_items_model, |items| {
                    reorder_vec_by_key(items, active_id.as_ref(), over_id.as_ref(), side, |item| {
                        item.id.as_ref()
                    })
                })
                .unwrap_or(false);
            let next_status = if moved {
                format!("Moved {} {} {}", active_label, side.label(), over_label)
            } else {
                "Drop ignored".to_string()
            };
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&outliner_status_model, |status| {
                    status.clear();
                    status.push_str(&next_status);
                });
        }

        let outliner_order =
            proof_outliner_order_line_for_model(ui.cx_mut().app, &outliner_items_model);
        let persisted_outliner_status =
            editor_string_model_readout(ui.cx_mut(), &outliner_status_model);
        let visible_outliner_status = preview_status.unwrap_or_else(|| persisted_outliner_status);
        ui.text(outliner_order);
        ui.text(format!("Status: {visible_outliner_status}"));
    })
}

fn build_authoring_parity_gradient_editor(
    cx: &mut AppComponentCx<'_>,
    angle_model: Model<f64>,
    stops_model: Model<Vec<GradientDemoStop>>,
    next_id_model: Model<u64>,
    id_source: &'static str,
    test_id_prefix: &'static str,
) -> GradientEditor {
    let stops = cx.data().selector_model_paint(&stops_model, |stops| stops);

    let on_remove: fret_ui_editor::composites::OnGradientStopAction = Arc::new({
        let stops_model = stops_model.clone();
        move |host, action_cx, stop_id| {
            let _ = host
                .models_mut()
                .update(&stops_model, |stops| stops.retain(|s| s.id != stop_id));
            host.request_redraw(action_cx.window);
        }
    });

    let on_add: fret_ui_editor::composites::OnGradientAction = Arc::new({
        let stops_model = stops_model.clone();
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

    GradientEditor::new(bindings)
        .angle_degrees(Some(angle_model))
        .on_add_stop(Some(on_add))
        .options(GradientEditorOptions {
            id_source: Some(Arc::from(id_source)),
            test_id: Some(Arc::from(test_id_prefix)),
            preview_test_id: Some(Arc::<str>::from(format!("{test_id_prefix}.preview"))),
            stops_test_id: Some(Arc::<str>::from(format!("{test_id_prefix}.stops"))),
            add_stop_test_id: Some(Arc::<str>::from(format!("{test_id_prefix}.add-stop"))),
            ..Default::default()
        })
}

fn render_authoring_parity_imui_host<H, F>(
    cx: &mut ElementContext<'_, H>,
    f: F,
) -> impl IntoUiElement<H> + use<H, F>
where
    H: UiHost,
    F: for<'cx, 'a> FnOnce(&mut fret_imui::ImUi<'cx, 'a, H>) + 'static,
{
    // Authoring-parity IMUI content can emit multiple siblings, so the proof-local host must own
    // vertical flow explicitly instead of forwarding them through a non-layout container box.
    fret_ui_kit::ui::v_flex_build(move |cx, out| {
        fret_imui::imui_build(cx, out, f);
    })
    .w_full()
    .into_element(cx)
}

fn authoring_parity_shading_items() -> Arc<[EnumSelectItem]> {
    vec![
        EnumSelectItem::new("lit", "Lit"),
        EnumSelectItem::new("unlit", "Unlit"),
        EnumSelectItem::new("matcap", "Matcap"),
    ]
    .into()
}

fn authoring_parity_drag_assets() -> Arc<[ProofDragAsset]> {
    vec![
        ProofDragAsset {
            label: Arc::from("Stone Albedo"),
            path: Arc::from("textures/stone/albedo.ktx2"),
        },
        ProofDragAsset {
            label: Arc::from("Stone Normal"),
            path: Arc::from("textures/stone/normal.ktx2"),
        },
        ProofDragAsset {
            label: Arc::from("Stone ORM"),
            path: Arc::from("textures/stone/orm.ktx2"),
        },
    ]
    .into()
}

fn authoring_parity_outliner_items() -> Arc<[ProofOutlinerItem]> {
    vec![
        ProofOutlinerItem {
            id: Arc::from("camera"),
            label: Arc::from("Camera"),
        },
        ProofOutlinerItem {
            id: Arc::from("cube"),
            label: Arc::from("Cube"),
        },
        ProofOutlinerItem {
            id: Arc::from("key-light"),
            label: Arc::from("Key light"),
        },
        ProofOutlinerItem {
            id: Arc::from("post-fx"),
            label: Arc::from("Post FX"),
        },
    ]
    .into()
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

fn authoring_parity_name_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.name",
        |cx| cx.app.models_mut().insert("Shared Cube".to_string()),
    )
}

fn authoring_parity_drag_value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.drag_value",
        |cx| cx.app.models_mut().insert(1.250_f64),
    )
}

fn authoring_parity_numeric_input_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.numeric_input",
        |cx| cx.app.models_mut().insert(0.875_f64),
    )
}

fn authoring_parity_slider_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.slider",
        |cx| cx.app.models_mut().insert(0.35_f64),
    )
}

fn authoring_parity_enabled_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.enabled",
        |cx| cx.app.models_mut().insert(true),
    )
}

fn authoring_parity_shading_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<Arc<str>>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.shading",
        |cx| {
            cx.app
                .models_mut()
                .insert(Some::<Arc<str>>(Arc::from("lit")))
        },
    )
}

fn authoring_parity_gradient_angle_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.gradient_angle",
        |cx| cx.app.models_mut().insert(90.0_f64),
    )
}

fn authoring_parity_gradient_stops_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Vec<GradientDemoStop>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.gradient_stops",
        |cx| {
            let stop_0_pos = cx.app.models_mut().insert(0.0_f64);
            let stop_0_color = cx.app.models_mut().insert(Color {
                a: 1.0,
                ..Color::from_srgb_hex_rgb(0x14_b8_a6)
            });
            let stop_1_pos = cx.app.models_mut().insert(1.0_f64);
            let stop_1_color = cx.app.models_mut().insert(Color {
                a: 1.0,
                ..Color::from_srgb_hex_rgb(0xf9_73_16)
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
        },
    )
}

fn authoring_parity_gradient_next_id_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<u64> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.gradient_next_id",
        |cx| cx.app.models_mut().insert(3_u64),
    )
}

fn authoring_parity_asset_slot_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.asset_slot",
        |cx| {
            cx.app
                .models_mut()
                .insert("textures/default/basecolor.ktx2".to_string())
        },
    )
}

fn authoring_parity_outliner_items_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Vec<ProofOutlinerItem>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.outliner_items",
        |cx| {
            cx.app
                .models_mut()
                .insert(authoring_parity_outliner_items().iter().cloned().collect())
        },
    )
}

fn authoring_parity_outliner_status_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.outliner_status",
        |cx| cx.app.models_mut().insert("Idle".to_string()),
    )
}

fn install_dock_panel_registry(app: &mut KernelApp) {
    let mut registry = DockPanelRegistryBuilder::new();
    registry.register(ImUiEditorProofControlsPanelFactory);
    app.with_global_mut(
        DockPanelRegistryService::<KernelApp>::default,
        |svc, _app| {
            svc.set(registry.build_arc());
        },
    );
}

struct ImUiEditorProofControlsPanelFactory;

impl DockPanelFactory<KernelApp> for ImUiEditorProofControlsPanelFactory {
    fn panel_kind(&self) -> PanelKind {
        PanelKind::new("demo.controls")
    }

    fn build_panel(
        &self,
        panel: &fret_core::PanelKey,
        cx: &mut DockPanelFactoryCx<'_, KernelApp>,
    ) -> Option<fret_core::NodeId> {
        let root_name = match panel.instance.as_deref() {
            Some(instance) => format!("imui_editor_proof.panel.{}:{}", panel.kind.0, instance),
            None => format!("imui_editor_proof.panel.{}", panel.kind.0),
        };
        let panel_key = panel.clone();
        Some(cx.render_cached_panel_root(
            &root_name,
            move |cx| {
                let target = embedded::models(&*cx.app, cx.window)
                    .map(|m| cx.data().selector_model_paint(&m.target, |target| target))
                    .unwrap_or_default();

                vec![
                    fret_ui_kit::ui::container_build( move |cx, out| {
                        out.extend(
                            fret_imui::imui(cx, move |ui| {
                                use fret_ui_kit::imui::UiWriterImUiFacadeExt as _;

                                // Dock panels can move across roots and windows, so the immediate
                                // content keeps an explicit stable identity instead of relying on
                                // callsite position alone.
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

fn ensure_dock_graph(app: &mut KernelApp, window: AppWindowId) {
    ensure_dock_graph_inner(app, window, false);
}

fn reset_dock_graph(app: &mut KernelApp, window: AppWindowId) {
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.graph.remove_window_root(window);
        dock.graph.floating_windows_mut(window).clear();
    });
    ensure_dock_graph_inner(app, window, true);
}

fn embedded_target_for_window(app: &KernelApp, window: AppWindowId) -> fret_core::RenderTargetId {
    embedded::models(app, window)
        .and_then(|m| app.models().read(&m.target, |v| *v).ok())
        .unwrap_or_default()
}

fn ensure_dock_graph_inner(app: &mut KernelApp, window: AppWindowId, force: bool) {
    app.with_global_mut(DockManager::default, |dock, app| {
        let logical_window_id = app
            .global::<WindowBootstrapService>()
            .and_then(|svc| svc.logical_by_window.get(&window).cloned())
            .unwrap_or_else(|| format!("{window:?}"));

        let viewport_panel =
            fret_core::PanelKey::with_instance("demo.viewport", logical_window_id.clone());
        let controls_panel = fret_core::PanelKey::with_instance("demo.controls", logical_window_id);

        let target = embedded_target_for_window(app, window);

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

fn ensure_aux_window_requested(app: &mut KernelApp, window: AppWindowId) {
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

fn on_dock_op(app: &mut KernelApp, op: fret_core::DockOp) {
    let _ = dock_runtime::handle_dock_op(app, op);
}

fn window_create_spec(
    _app: &mut KernelApp,
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

fn window_created(
    app: &mut KernelApp,
    request: &fret_app::CreateWindowRequest,
    new_window: AppWindowId,
) {
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

fn before_close_window(app: &mut KernelApp, closing_window: AppWindowId) -> bool {
    let target_window = app
        .global::<WindowBootstrapService>()
        .and_then(|svc| svc.main_window)
        .unwrap_or(closing_window);
    let _ = dock_runtime::handle_dock_before_close_window(app, closing_window, target_window);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn proof_outliner_reorder_moves_item_after_target() {
        let mut items = authoring_parity_outliner_items()
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
        let mut items = authoring_parity_outliner_items()
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
}
