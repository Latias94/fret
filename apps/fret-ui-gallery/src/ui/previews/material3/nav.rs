use super::super::super::doc_layout::DocSection;
use super::super::super::*;

pub(in crate::ui) fn preview_material3_tabs(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::tabs::render(cx, value);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::material3::tabs::SOURCE, "example"),
        ],
    );

    vec![page]
}

pub(in crate::ui) fn preview_material3_navigation_bar(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::navigation_bar::render(cx, value);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![
            DocSection::new("Demo", demo)
                .code_rust_from_file_region(snippets::material3::navigation_bar::SOURCE, "example"),
        ],
    );

    vec![page]
}

pub(in crate::ui) fn preview_material3_navigation_rail(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::navigation_rail::render(cx, value);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![
            DocSection::new("Demo", demo).code_rust_from_file_region(
                snippets::material3::navigation_rail::SOURCE,
                "example",
            ),
        ],
    );

    vec![page]
}

pub(in crate::ui) fn preview_material3_navigation_drawer(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::navigation_drawer::render(cx, value);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![
            DocSection::new("Demo", demo).code_rust_from_file_region(
                snippets::material3::navigation_drawer::SOURCE,
                "example",
            ),
        ],
    );

    vec![page]
}

pub(in crate::ui) fn preview_material3_modal_navigation_drawer(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::modal_navigation_drawer::render(cx, open, value);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![DocSection::new("Demo", demo).code_rust_from_file_region(
            snippets::material3::modal_navigation_drawer::SOURCE,
            "example",
        )],
    );

    vec![page]
}
