use super::*;
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::IntoUiElement;

use crate::ui::doc_layout::{self, DocSection};

pub(super) const MATERIAL3_INTRO: &str =
    "Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code).";

pub(super) fn render_material3_demo_page<D>(
    cx: &mut AppComponentCx<'_>,
    intro: Option<&'static str>,
    demo: D,
    source: &'static str,
) -> Vec<AnyElement>
where
    D: IntoUiElement<fret_app::App>,
{
    let demo_section =
        DocSection::build(cx, "Demo", demo).code_rust_from_file_region(source, "example");
    let page = doc_layout::render_doc_page(cx, intro, vec![demo_section]);

    vec![page.into_element(cx)]
}

pub(in crate::ui) fn material3_scoped_page<'a, I, F>(
    cx: &mut AppComponentCx<'a>,
    material3_expressive: Model<bool>,
    content: F,
) -> Vec<AnyElement>
where
    F: for<'b> FnOnce(&mut AppComponentCx<'b>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    let enabled = cx
        .get_model_copied(&material3_expressive, Invalidation::Layout)
        .unwrap_or(false);

    let mut out: Vec<AnyElement> = Vec::new();
    out.push(material3_variant_toggle_row(cx, material3_expressive).into_element(cx));

    let body = if enabled {
        crate::ui::material3::context::with_material_design_variant(
            cx,
            crate::ui::material3::MaterialDesignVariant::Expressive,
            content,
        )
    } else {
        content(cx)
    };

    out.extend(body);
    out
}

pub(in crate::ui) fn material3_variant_toggle_row(
    cx: &mut AppComponentCx<'_>,
    material3_expressive: Model<bool>,
) -> impl UiChild + use<> {
    let enabled = cx
        .get_model_copied(&material3_expressive, Invalidation::Layout)
        .unwrap_or(false);

    ui::h_row(move |cx| {
        vec![
            shadcn::Switch::new(material3_expressive.clone())
                .a11y_label("Enable Material 3 Expressive variant")
                .test_id("ui-gallery-material3-design-variant-toggle")
                .into_element(cx),
            ui::label(if enabled {
                "Variant: Expressive"
            } else {
                "Variant: Standard"
            })
            .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
}
