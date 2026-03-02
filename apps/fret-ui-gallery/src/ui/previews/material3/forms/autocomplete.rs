use super::super::super::super::doc_layout::DocSection;
use super::super::super::super::*;

pub(in crate::ui) fn preview_material3_autocomplete(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
    disabled: Model<bool>,
    error: Model<bool>,
    dialog_open: Model<bool>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::autocomplete::render(cx, value, disabled, error, dialog_open);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::material3::autocomplete::SOURCE, "example"),
        ],
    );

    vec![page]
}
