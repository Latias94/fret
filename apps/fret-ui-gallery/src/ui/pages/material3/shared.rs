use super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};

pub(super) const MATERIAL3_INTRO: &str =
    "Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code).";

pub(super) fn render_material3_demo_page(
    cx: &mut UiCx<'_>,
    intro: Option<&'static str>,
    demo: AnyElement,
    source: &'static str,
) -> Vec<AnyElement> {
    let page = doc_layout::render_doc_page(
        cx,
        intro,
        vec![DocSection::new("Demo", demo).code_rust_from_file_region(source, "example")],
    );

    vec![page]
}

pub(in crate::ui) fn material3_scoped_page<'a, I, F>(
    cx: &mut UiCx<'a>,
    material3_expressive: Model<bool>,
    content: F,
) -> Vec<AnyElement>
where
    F: for<'b> FnOnce(&mut UiCx<'b>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    let enabled = cx
        .get_model_copied(&material3_expressive, Invalidation::Layout)
        .unwrap_or(false);

    let mut out: Vec<AnyElement> = Vec::new();
    out.push(material3_variant_toggle_row(cx, material3_expressive));

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
    cx: &mut UiCx<'_>,
    material3_expressive: Model<bool>,
) -> AnyElement {
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
