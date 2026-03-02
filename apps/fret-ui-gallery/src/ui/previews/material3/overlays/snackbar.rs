use super::super::super::super::doc_layout::DocSection;
use super::super::super::super::*;

pub(in crate::ui) fn preview_material3_snackbar(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::snackbar::render(cx, last_action);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::material3::snackbar::SOURCE, "example"),
        ],
    );

    vec![page]
}
