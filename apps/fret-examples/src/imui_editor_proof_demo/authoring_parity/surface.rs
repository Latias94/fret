use std::sync::Arc;

use fret::AppComponentCx;
use fret::advanced::KernelApp;
use fret::component::prelude::*;
use fret_ui_editor::controls::EnumSelectItem;

use super::super::collection;
use super::super::diag_enabled;
use super::super::proof_helpers::{authoring_parity_theme_diag_lines, proof_compact_readout};
use super::common::{
    authoring_parity_shading_items as common_authoring_parity_shading_items,
    build_authoring_parity_gradient_editor as common_build_authoring_parity_gradient_editor,
    render_authoring_parity_imui_host as common_render_authoring_parity_imui_host,
};
use super::{AuthoringParityModels, drag_assets};
use super::{declarative, imui};

pub(in super::super) fn render_surface(
    cx: &mut AppComponentCx<'_>,
    models: AuthoringParityModels,
) -> impl IntoUiElement<KernelApp> + use<> {
    let shading_items = common_authoring_parity_shading_items();
    let asset_chips = drag_assets();

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
                        let models = models.clone();
                        move |cx, out| {
                            out.push(
                                render_authoring_parity_declarative_group(
                                    cx,
                                    models,
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
                                models,
                                shading_items,
                                asset_chips.clone(),
                                move |ui| {
                                    collection::render_collection_first_asset_browser_proof(ui);
                                },
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

fn render_authoring_parity_declarative_group(
    cx: &mut AppComponentCx<'_>,
    models: AuthoringParityModels,
    shading_items: Arc<[EnumSelectItem]>,
) -> impl IntoUiElement<KernelApp> + use<> {
    declarative::render_authoring_parity_declarative_group(cx, models, shading_items)
}

fn render_authoring_parity_imui_group<FCollection>(
    cx: &mut AppComponentCx<'_>,
    models: AuthoringParityModels,
    shading_items: Arc<[EnumSelectItem]>,
    asset_chips: Arc<[super::super::ProofDragAsset]>,
    render_collection_browser: FCollection,
) -> impl IntoUiElement<KernelApp> + use<FCollection>
where
    FCollection: for<'cx, 'a> FnOnce(&mut fret::imui::ImUi<'cx, 'a, KernelApp>) + 'static,
{
    imui::render_authoring_parity_imui_group(
        cx,
        models,
        shading_items,
        asset_chips,
        render_collection_browser,
    )
}

fn build_authoring_parity_gradient_editor(
    cx: &mut AppComponentCx<'_>,
    angle_model: fret_runtime::Model<f64>,
    stops_model: fret_runtime::Model<Vec<super::super::GradientDemoStop>>,
    next_id_model: fret_runtime::Model<u64>,
    id_source: &'static str,
    test_id_prefix: &'static str,
) -> fret_ui_editor::composites::GradientEditor {
    common_build_authoring_parity_gradient_editor(
        cx,
        angle_model,
        stops_model,
        next_id_model,
        id_source,
        test_id_prefix,
    )
}

fn render_authoring_parity_imui_host<H, F>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    f: F,
) -> impl IntoUiElement<H> + use<H, F>
where
    H: fret_ui::UiHost,
    F: for<'cx, 'a> FnOnce(&mut fret::imui::ImUi<'cx, 'a, H>) + 'static,
{
    common_render_authoring_parity_imui_host(cx, f)
}

fn authoring_parity_shading_items() -> Arc<[EnumSelectItem]> {
    common_authoring_parity_shading_items()
}
