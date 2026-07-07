use std::sync::Arc;

use fret::AppComponentCx;
use fret::advanced::KernelApp;
use fret::advanced::text;
use fret::app::AppRenderDataExt as _;
use fret::imui::UiWriterImUiFacadeExt;
use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, Length};
use fret_ui::{ElementContext, UiHost};
use fret_ui_editor::controls::{
    DragValueOptions, NumericInputOptions, NumericPresentation, SliderOptions,
    TransformEditAxisOutcome, TransformEditPresentations, TransformEditSection, VecEditAxis,
    VecEditAxisOutcome,
};
use fret_ui_editor::primitives::{EditSessionOutcome, EditorTokenKeys};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::headless::text_assist::{
    TextAssistItem, TextAssistMatchMode, controller_with_active_item_id,
    input_owned_text_assist_expanded,
};

pub(super) fn authoring_parity_blend_slider_options(
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

pub(super) fn authoring_parity_drag_value_options(
    presentation: &NumericPresentation<f64>,
    id_source: &'static str,
    test_id: &'static str,
) -> DragValueOptions {
    DragValueOptions {
        id_source: Some(Arc::from(id_source)),
        test_id: Some(Arc::from(test_id)),
        prefix: presentation.chrome_prefix().cloned(),
        suffix: presentation.chrome_suffix().cloned(),
        ..Default::default()
    }
}

pub(super) fn authoring_parity_numeric_input_options(
    presentation: &NumericPresentation<f64>,
    id_source: &'static str,
    test_id: &'static str,
) -> NumericInputOptions {
    NumericInputOptions {
        id_source: Some(Arc::from(id_source)),
        test_id: Some(Arc::from(test_id)),
        prefix: presentation.chrome_prefix().cloned(),
        suffix: presentation.chrome_suffix().cloned(),
        ..Default::default()
    }
}

pub(super) fn editor_fixed_decimals_presentation() -> NumericPresentation<f64> {
    NumericPresentation::fixed_decimals(3)
}

pub(super) fn editor_position_presentation() -> NumericPresentation<f64> {
    editor_fixed_decimals_presentation().with_chrome_suffix("m")
}

pub(super) fn editor_rotation_presentation() -> NumericPresentation<f64> {
    NumericPresentation::degrees(0)
}

pub(super) fn editor_percent_presentation() -> NumericPresentation<f64> {
    NumericPresentation::percent_0_1(0)
}

pub(super) fn editor_transform_presentations() -> TransformEditPresentations {
    TransformEditPresentations::new(
        editor_position_presentation(),
        editor_rotation_presentation(),
        editor_percent_presentation(),
    )
}

pub(super) fn authoring_parity_value_presentation() -> NumericPresentation<f64> {
    editor_fixed_decimals_presentation()
        .with_chrome_prefix("$")
        .with_chrome_suffix("ms")
}

pub(super) fn authoring_parity_blend_presentation() -> NumericPresentation<f64> {
    editor_percent_presentation()
}

pub(super) fn edit_session_outcome_label(outcome: EditSessionOutcome) -> &'static str {
    match outcome {
        EditSessionOutcome::Committed => "Committed",
        EditSessionOutcome::Canceled => "Canceled",
    }
}

pub(super) fn compact_edit_session_outcome_label(outcome: EditSessionOutcome) -> &'static str {
    match outcome {
        EditSessionOutcome::Committed => "Commit",
        EditSessionOutcome::Canceled => "Cancel",
    }
}

pub(super) fn vec_edit_axis_label(axis: VecEditAxis) -> &'static str {
    match axis {
        VecEditAxis::X => "X",
        VecEditAxis::Y => "Y",
        VecEditAxis::Z => "Z",
        VecEditAxis::W => "W",
    }
}

pub(super) fn vec_edit_axis_outcome_label(outcome: VecEditAxisOutcome) -> String {
    format!(
        "{} {}",
        vec_edit_axis_label(outcome.axis()),
        edit_session_outcome_label(outcome.outcome())
    )
}

pub(super) fn transform_edit_section_label(section: TransformEditSection) -> &'static str {
    match section {
        TransformEditSection::Position => "Position",
        TransformEditSection::Rotation => "Rotation",
        TransformEditSection::Scale => "Scale",
    }
}

pub(super) fn transform_edit_axis_outcome_label(outcome: TransformEditAxisOutcome) -> String {
    format!(
        "{}.{} {}",
        transform_edit_section_label(outcome.section()),
        vec_edit_axis_label(outcome.axis()),
        edit_session_outcome_label(outcome.outcome())
    )
}

pub(super) fn proof_optional_outcome_readout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    outcome: String,
    test_id: Arc<str>,
) -> Option<AnyElement> {
    let outcome = outcome.trim().to_string();
    if outcome.is_empty() {
        return None;
    }

    Some(proof_compact_readout(cx, outcome, Some(test_id)))
}

pub(super) fn proof_compact_readout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    readout: String,
    test_id: Option<Arc<str>>,
) -> fret_ui::element::AnyElement {
    let readout = Arc::<str>::from(readout);
    let mut el = text::control_readout(cx, readout.clone());
    if let Some(test_id) = test_id {
        el = el.test_id(test_id);
    }
    el.a11y_label(readout)
}

pub(super) fn proof_compact_readout_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    readout: impl Into<Arc<str>>,
    test_id: impl Into<Arc<str>>,
) -> AnyElement {
    let readout = readout.into();
    let mut el = text::control_readout(cx, readout.clone()).test_id(test_id.into());
    el = el.a11y_label(readout);
    el
}

pub(super) fn proof_empty_state_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: &'static str,
    test_id: &'static str,
) -> AnyElement {
    proof_compact_readout_element(cx, Arc::<str>::from(text), Arc::<str>::from(test_id))
}

pub(super) fn proof_section_chrome_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: &'static str,
    test_id: &'static str,
) -> AnyElement {
    text::section_chrome_label(cx, text).test_id(test_id)
}

pub(super) fn proof_imui_section_text(
    ui: &mut (impl UiWriterImUiFacadeExt<KernelApp> + ?Sized),
    text: impl Into<Arc<str>>,
) {
    let text = text.into();
    let element = ui.with_cx_mut(move |cx| text::section_chrome_label(cx, text));
    ui.add(element);
}

pub(super) fn proof_imui_readout_text(
    ui: &mut (impl UiWriterImUiFacadeExt<KernelApp> + ?Sized),
    text: impl Into<Arc<str>>,
) {
    let text = text.into();
    let element = ui.with_cx_mut(move |cx| text::control_readout(cx, text));
    ui.add(element);
}

pub(super) fn proof_imui_compact_paragraph_text(
    ui: &mut (impl UiWriterImUiFacadeExt<KernelApp> + ?Sized),
    text: impl Into<Arc<str>>,
) {
    let text = text.into();
    let element = ui.with_cx_mut(move |cx| text::compact_paragraph(cx, text));
    ui.add(element);
}

fn color_hex_readout(color: Option<Color>) -> String {
    color
        .map(|color| format!("#{:06X}", color.to_srgb_hex_rgb()))
        .unwrap_or_else(|| "<none>".to_string())
}

pub(super) fn authoring_parity_theme_diag_lines(cx: &mut AppComponentCx<'_>) -> [String; 2] {
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

pub(super) fn committed_line_count_label(text: &str) -> String {
    let lines = text.lines().count();
    let noun = if lines == 1 { "line" } else { "lines" };
    format!("{lines} {noun}")
}

pub(super) fn committed_char_count_label(text: &str) -> String {
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
pub(super) struct EditorTextAssistReadout {
    pub(super) state_label: String,
    pub(super) active_label: String,
}

#[derive(Clone)]
pub(super) struct EditorTextFieldReadout {
    pub(super) committed: String,
    pub(super) outcome: String,
}

#[derive(Clone)]
pub(super) struct AuthoringParitySharedStateReadout {
    pub(super) name_line: String,
    pub(super) value_line: String,
    pub(super) numeric_line: String,
    pub(super) blend_line: String,
    pub(super) enabled_line: String,
    pub(super) shading_line: String,
    pub(super) gradient_line: String,
}

#[derive(Clone)]
pub(super) struct ProofDragAsset {
    pub(super) label: Arc<str>,
    pub(super) path: Arc<str>,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct ProofOutlinerItem {
    pub(super) id: Arc<str>,
    pub(super) label: Arc<str>,
}

#[derive(Clone)]
pub(super) struct ProofOutlinerDragItem {
    pub(super) id: Arc<str>,
    pub(super) label: Arc<str>,
}

pub(super) fn proof_outliner_order_line(items: &[ProofOutlinerItem]) -> String {
    let labels = items
        .iter()
        .map(|item| item.label.as_ref())
        .collect::<Vec<_>>()
        .join(" -> ");
    format!("Order: {labels}")
}

pub(super) fn proof_outliner_items_snapshot(
    app: &KernelApp,
    model: &Model<Vec<ProofOutlinerItem>>,
) -> Vec<ProofOutlinerItem> {
    app.models()
        .read(model, |items| items.clone())
        .unwrap_or_default()
}

pub(super) fn proof_outliner_order_line_for_model(
    app: &KernelApp,
    model: &Model<Vec<ProofOutlinerItem>>,
) -> String {
    app.models()
        .read(model, |items| proof_outliner_order_line(items))
        .unwrap_or_else(|_| "Order: unavailable".to_string())
}

pub(super) fn proof_drag_preview_card<H: UiHost>(
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

        out.push(cx.container(props, move |cx| {
            let mut children = Vec::new();
            children.push(text::section_chrome_label(cx, title.clone()));
            if let Some(subtitle) = subtitle.as_ref() {
                children.push(text::control_readout(cx, subtitle.clone()));
            }
            children
        }));
    })
}

pub(super) fn editor_text_assist_readout(
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

pub(super) fn editor_text_field_readout(
    cx: &mut AppComponentCx<'_>,
    committed_model: &Model<String>,
    outcome_model: &Model<String>,
) -> EditorTextFieldReadout {
    cx.keyed(
        (
            "imui-editor-proof.editor-text-field-readout",
            committed_model.id(),
            outcome_model.id(),
        ),
        |cx| {
            cx.data()
                .selector_model_paint((committed_model, outcome_model), |(committed, outcome)| {
                    EditorTextFieldReadout { committed, outcome }
                })
        },
    )
}

pub(super) fn editor_string_model_readout(
    cx: &mut AppComponentCx<'_>,
    model: &Model<String>,
) -> String {
    cx.keyed(
        ("imui-editor-proof.string-model-readout", model.id()),
        |cx| cx.data().selector_model_paint(model, |value| value),
    )
}
