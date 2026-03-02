use super::super::super::super::doc_layout::DocSection;
use super::super::super::super::*;

pub(in crate::ui) fn preview_material3_gallery(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_list_value: Model<Arc<str>>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::gallery::render(
        cx,
        material3_checkbox,
        material3_switch,
        material3_radio_value,
        material3_tabs_value,
        material3_list_value,
        material3_navigation_bar_value,
        material3_text_field_value,
        material3_text_field_disabled,
        material3_text_field_error,
    );

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::material3::gallery::SOURCE, "example"),
        ],
    );

    vec![page]
}
