use super::super::super::super::*;
use super::super::super::super::doc_layout::DocSection;

pub(in crate::ui) fn preview_material3_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::menu::render(cx, open, last_action);

    let page = doc_layout::render_doc_page(
        cx,
        Some("Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code)."),
        vec![DocSection::new("Demo", demo)
            .code_rust_from_file_region(snippets::material3::menu::SOURCE, "example")],
    );

    vec![page]
}
