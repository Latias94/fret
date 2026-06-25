use std::sync::Arc;

use fret::advanced::KernelApp;
use fret::component::prelude::*;
use fret_core::Color;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle};
use fret_ui_editor::composites::{
    InspectorPanel, InspectorPanelCx, InspectorPanelOptions, InspectorPanelSearchAssistOptions,
};

use super::asset_ref;
use super::editor_advanced::{EditorAdvancedModels, render_editor_advanced_surface};
use super::editor_gradient::{EditorGradientModels, render_editor_gradient_surface};
use super::editor_material::{EditorMaterialModels, render_editor_material_surface};
use super::editor_object::{EditorObjectModels, render_editor_object_surface};
use super::editor_state::{
    GradientDemoStop, editor_demo_alpha_clip_model, editor_demo_base_color_model,
    editor_demo_buffered_name_model, editor_demo_cast_shadows_model,
    editor_demo_drag_value_outcome_model, editor_demo_exposure_model,
    editor_demo_gradient_angle_model, editor_demo_gradient_next_id_model,
    editor_demo_gradient_stops_model, editor_demo_inline_rename_model,
    editor_demo_inline_rename_outcome_model, editor_demo_iterations_model,
    editor_demo_metallic_model, editor_demo_name_assist_accepted_model,
    editor_demo_name_assist_active_item_model, editor_demo_name_assist_dismissed_query_model,
    editor_demo_name_assist_model, editor_demo_name_model, editor_demo_notes_model,
    editor_demo_notes_outcome_model, editor_demo_password_model,
    editor_demo_password_outcome_model, editor_demo_position_models,
    editor_demo_position_outcome_model, editor_demo_rotation_models, editor_demo_roughness_model,
    editor_demo_scale_models, editor_demo_search_assist_active_item_model,
    editor_demo_search_assist_dismissed_query_model, editor_demo_search_model,
    editor_demo_shading_model, editor_demo_transform_outcome_model, editor_demo_value_model,
};
use super::editor_text_assist::editor_demo_search_assist_items;
use super::proof_helpers::proof_empty_state_text;

#[derive(Clone)]
pub(super) struct EditorInspectorModels {
    opacity: Model<f64>,
    opacity_outcome: Model<String>,
    roughness: Model<f64>,
    metallic: Model<f64>,
    alpha_clip: Model<bool>,
    cast_shadows: Model<Option<bool>>,
    shading: Model<Option<Arc<str>>>,
    base_color: Model<Color>,
    asset_slot: Model<String>,
    asset_action: Model<String>,
    name: Model<String>,
    buffered_name: Model<String>,
    inline_rename: Model<String>,
    inline_rename_outcome: Model<String>,
    name_assist: Model<String>,
    name_assist_dismissed_query: Model<String>,
    name_assist_active_item: Model<Option<Arc<str>>>,
    name_assist_accepted: Model<String>,
    password: Model<String>,
    password_outcome: Model<String>,
    notes: Model<String>,
    notes_outcome: Model<String>,
    pos_x: Model<f64>,
    pos_y: Model<f64>,
    pos_z: Model<f64>,
    position_outcome: Model<String>,
    rot_x: Model<f64>,
    rot_y: Model<f64>,
    rot_z: Model<f64>,
    scl_x: Model<f64>,
    scl_y: Model<f64>,
    scl_z: Model<f64>,
    transform_outcome: Model<String>,
    iterations: Model<i32>,
    exposure: Model<f64>,
    search: Model<String>,
    search_assist_dismissed_query: Model<String>,
    search_assist_active_item: Model<Option<Arc<str>>>,
    gradient_angle: Model<f64>,
    gradient_stops: Model<Vec<GradientDemoStop>>,
    gradient_next_id: Model<u64>,
}

pub(super) fn editor_inspector_models(
    cx: &mut ElementContext<'_, KernelApp>,
) -> EditorInspectorModels {
    let roughness = editor_demo_roughness_model(cx);
    let metallic = editor_demo_metallic_model(cx);

    #[cfg(debug_assertions)]
    {
        debug_assert_ne!(
            roughness.id(),
            metallic.id(),
            "Roughness/Metallic models must be distinct; otherwise sliders will sync unintentionally."
        );
    }

    let (pos_x, pos_y, pos_z) = editor_demo_position_models(cx);
    let (rot_x, rot_y, rot_z) = editor_demo_rotation_models(cx);
    let (scl_x, scl_y, scl_z) = editor_demo_scale_models(cx);

    EditorInspectorModels {
        opacity: editor_demo_value_model(cx),
        opacity_outcome: editor_demo_drag_value_outcome_model(cx),
        roughness,
        metallic,
        alpha_clip: editor_demo_alpha_clip_model(cx),
        cast_shadows: editor_demo_cast_shadows_model(cx),
        shading: editor_demo_shading_model(cx),
        base_color: editor_demo_base_color_model(cx),
        asset_slot: asset_ref::asset_slot_model(cx),
        asset_action: asset_ref::asset_action_model(cx),
        name: editor_demo_name_model(cx),
        buffered_name: editor_demo_buffered_name_model(cx),
        inline_rename: editor_demo_inline_rename_model(cx),
        inline_rename_outcome: editor_demo_inline_rename_outcome_model(cx),
        name_assist: editor_demo_name_assist_model(cx),
        name_assist_dismissed_query: editor_demo_name_assist_dismissed_query_model(cx),
        name_assist_active_item: editor_demo_name_assist_active_item_model(cx),
        name_assist_accepted: editor_demo_name_assist_accepted_model(cx),
        password: editor_demo_password_model(cx),
        password_outcome: editor_demo_password_outcome_model(cx),
        notes: editor_demo_notes_model(cx),
        notes_outcome: editor_demo_notes_outcome_model(cx),
        pos_x,
        pos_y,
        pos_z,
        position_outcome: editor_demo_position_outcome_model(cx),
        rot_x,
        rot_y,
        rot_z,
        scl_x,
        scl_y,
        scl_z,
        transform_outcome: editor_demo_transform_outcome_model(cx),
        iterations: editor_demo_iterations_model(cx),
        exposure: editor_demo_exposure_model(cx),
        search: editor_demo_search_model(cx),
        search_assist_dismissed_query: editor_demo_search_assist_dismissed_query_model(cx),
        search_assist_active_item: editor_demo_search_assist_active_item_model(cx),
        gradient_angle: editor_demo_gradient_angle_model(cx),
        gradient_stops: editor_demo_gradient_stops_model(cx),
        gradient_next_id: editor_demo_gradient_next_id_model(cx),
    }
}

pub(super) fn render_editor_inspector_surface(
    cx: &mut ElementContext<'_, KernelApp>,
    models: EditorInspectorModels,
    editor_review_layout: bool,
) -> AnyElement {
    InspectorPanel::new(Some(models.search.clone()))
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
            header_test_id: Some(Arc::from("imui-editor-proof.editor.inspector.header")),
            search_test_id: Some(Arc::from("imui-editor-proof.editor.search")),
            search_clear_test_id: Some(Arc::from("imui-editor-proof.editor.search.clear")),
            search_assist: Some(InspectorPanelSearchAssistOptions {
                dismissed_query_model: models.search_assist_dismissed_query.clone(),
                active_item_id_model: models.search_assist_active_item.clone(),
                items: editor_demo_search_assist_items(cx),
                list_label: Arc::from("Inspector search history"),
                empty_label: Arc::from("No search history matches"),
                key_options: Default::default(),
                list_test_id: Some(Arc::from("imui-editor-proof.editor.search.list")),
                item_test_id_prefix: Some(Arc::from("imui-editor-proof.editor.search.list.item")),
                empty_test_id: Some(Arc::from("imui-editor-proof.editor.search.no-matches")),
                max_list_height: None,
            }),
            content_test_id: Some(Arc::from("imui-editor-proof.editor.inspector.content")),
            ..Default::default()
        })
        .into_element(
            cx,
            |_cx, _panel_cx| Vec::new(),
            move |cx, panel_cx| render_editor_inspector_content(cx, panel_cx, models.clone()),
        )
}

fn render_editor_inspector_content(
    cx: &mut ElementContext<'_, KernelApp>,
    panel_cx: &InspectorPanelCx,
    models: EditorInspectorModels,
) -> Vec<AnyElement> {
    let mut out = Vec::new();

    let object_surface = render_editor_object_surface(
        cx,
        panel_cx,
        EditorObjectModels {
            name: models.name.clone(),
            buffered_name: models.buffered_name.clone(),
            inline_rename: models.inline_rename.clone(),
            inline_rename_outcome: models.inline_rename_outcome.clone(),
            name_assist: models.name_assist.clone(),
            name_assist_dismissed_query: models.name_assist_dismissed_query.clone(),
            name_assist_active_item: models.name_assist_active_item.clone(),
            name_assist_accepted: models.name_assist_accepted.clone(),
            password: models.password.clone(),
            password_outcome: models.password_outcome.clone(),
            notes: models.notes.clone(),
            notes_outcome: models.notes_outcome.clone(),
        },
    );
    let object_any_match = object_surface.any_match;
    if let Some(element) = object_surface.element {
        out.push(element);
    }

    let material_surface = render_editor_material_surface(
        cx,
        panel_cx,
        EditorMaterialModels {
            opacity: models.opacity.clone(),
            opacity_outcome: models.opacity_outcome.clone(),
            roughness: models.roughness.clone(),
            metallic: models.metallic.clone(),
            base_color: models.base_color.clone(),
            asset_slot: models.asset_slot.clone(),
            asset_action: models.asset_action.clone(),
            shading: models.shading.clone(),
            alpha_clip: models.alpha_clip.clone(),
            cast_shadows: models.cast_shadows.clone(),
        },
    );
    let material_any_match = material_surface.any_match;
    if let Some(element) = material_surface.element {
        out.push(element);
    }

    let gradient_surface = render_editor_gradient_surface(
        cx,
        panel_cx,
        EditorGradientModels {
            angle_degrees: models.gradient_angle.clone(),
            stops: models.gradient_stops.clone(),
            next_id: models.gradient_next_id.clone(),
        },
    );
    let gradient_any_match = gradient_surface.any_match;
    if let Some(element) = gradient_surface.element {
        out.push(element);
    }

    let advanced_surface = render_editor_advanced_surface(
        cx,
        panel_cx,
        EditorAdvancedModels {
            pos_x: models.pos_x.clone(),
            pos_y: models.pos_y.clone(),
            pos_z: models.pos_z.clone(),
            position_outcome: models.position_outcome.clone(),
            rot_x: models.rot_x.clone(),
            rot_y: models.rot_y.clone(),
            rot_z: models.rot_z.clone(),
            scl_x: models.scl_x.clone(),
            scl_y: models.scl_y.clone(),
            scl_z: models.scl_z.clone(),
            transform_outcome: models.transform_outcome.clone(),
            iterations: models.iterations.clone(),
            exposure: models.exposure.clone(),
        },
    );
    let advanced_any_match = advanced_surface.any_match;
    if let Some(element) = advanced_surface.element {
        out.push(element);
    }

    let any_match =
        object_any_match || material_any_match || gradient_any_match || advanced_any_match;

    if !panel_cx.is_query_empty() && !any_match {
        out.push(proof_empty_state_text(
            cx,
            "No matches",
            "imui-editor-proof.editor.no-matches",
        ));
    }

    out
}
