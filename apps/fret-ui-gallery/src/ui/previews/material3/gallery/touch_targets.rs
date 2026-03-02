use super::super::super::super::*;
use super::super::super::super::doc_layout::DocSection;

pub(in crate::ui) fn preview_material3_touch_targets(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::touch_targets::render(
        cx,
        material3_checkbox,
        material3_switch,
        material3_radio_value,
        material3_tabs_value,
    );

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![DocSection::new("Demo", demo).code_rust_from_file_region(
            snippets::material3::touch_targets::SOURCE,
            "example",
        )],
    );

    vec![page]
}
