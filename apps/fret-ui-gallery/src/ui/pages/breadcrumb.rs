use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::breadcrumb as snippets;

pub(super) fn preview_breadcrumb(
    cx: &mut ElementContext<'_, App>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let basic = snippets::basic::render(cx);
    let custom_separator = snippets::custom_separator::render(cx);
    let dropdown = snippets::dropdown::render(cx);
    let collapsed = snippets::collapsed::render(cx);
    let link_component = snippets::link_component::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Prefer short, task-oriented labels and keep only the current page as non-clickable text.",
            "Use separators and collapse strategy (`BreadcrumbItem::ellipsis`) to keep paths readable in narrow sidebars.",
            "Dropdown and router-link samples use typed pressables/links (ADR 0115 avoids general Slot/`asChild` prop merging).",
            "Validate RTL with long labels to ensure truncation and separator spacing remain stable.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Breadcrumb docs order for quick lookup and side-by-side behavior checks.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .title_test_id("ui-gallery-breadcrumb-section-title-demo")
                .description("Basic breadcrumb recipe with ellipsis and current page.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .title_test_id("ui-gallery-breadcrumb-section-title-basic")
                .description("A minimal breadcrumb list with three items.")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Custom Separator", custom_separator)
                .title_test_id("ui-gallery-breadcrumb-section-title-custom-separator")
                .description("Use a custom separator icon for parity with docs.")
                .code_rust_from_file_region(snippets::custom_separator::SOURCE, "example"),
            DocSection::new("Dropdown", dropdown)
                .title_test_id("ui-gallery-breadcrumb-section-title-dropdown")
                .description("Collapsed middle segment can expand via a dropdown menu.")
                .code_rust_from_file_region(snippets::dropdown::SOURCE, "example"),
            DocSection::new("Collapsed", collapsed)
                .title_test_id("ui-gallery-breadcrumb-section-title-collapsed")
                .description("Use `BreadcrumbItem::ellipsis` to keep paths readable in narrow layouts.")
                .code_rust_from_file_region(snippets::collapsed::SOURCE, "example"),
            DocSection::new("Link Component", link_component)
                .title_test_id("ui-gallery-breadcrumb-section-title-link-component")
                .description("Example of a truncated router-link style item.")
                .code_rust_from_file_region(snippets::link_component::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .title_test_id("ui-gallery-breadcrumb-section-title-rtl")
                .description("Breadcrumb layout should follow right-to-left direction context.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .title_test_id("ui-gallery-breadcrumb-section-title-notes")
                .description("Implementation notes and regression guidelines."),
        ],
    )
    .test_id("ui-gallery-breadcrumb-component");

    vec![body]
}
