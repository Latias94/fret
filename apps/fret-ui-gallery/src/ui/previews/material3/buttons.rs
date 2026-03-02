use super::super::super::doc_layout::DocSection;
use super::super::super::*;

pub(in crate::ui) fn preview_material3_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::material3::button::render(cx);

    let page = doc_layout::render_doc_page(
        cx,
        Some(
            "Material 3 surfaces are still migrating to snippet-backed pages. This page is the first scaffolded example.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::material3::button::SOURCE, "example"),
        ],
    );

    vec![page]
}
