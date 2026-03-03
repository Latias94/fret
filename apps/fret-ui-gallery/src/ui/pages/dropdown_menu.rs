use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::dropdown_menu as snippets;

pub(super) fn preview_dropdown_menu(
    cx: &mut ElementContext<'_, App>,
    _open: Model<bool>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let parts = snippets::parts::render(cx);
    let basic = snippets::basic::render(cx);
    let submenu = snippets::submenu::render(cx);
    let shortcuts = snippets::shortcuts::render(cx);
    let icons = snippets::icons::render(cx);
    let checkboxes = snippets::checkboxes::render(cx);
    let checkboxes_icons = snippets::checkboxes_icons::render(cx);
    let radio_group = snippets::radio_group::render(cx);
    let radio_icons = snippets::radio_icons::render(cx);
    let destructive = snippets::destructive::render(cx);
    let avatar = snippets::avatar::render(cx);
    let complex = snippets::complex::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows the upstream shadcn Dropdown Menu docs (v4 Base UI).",
            "Examples are snippet-backed: Preview ≡ Code (single source of truth).",
            "Keep `ui-gallery-dropdown-menu-*` test IDs stable; multiple diag scripts depend on them.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Dropdown Menu examples, aligned with upstream shadcn docs (Base UI variant). \
             Includes an extra `Parts` section for the Trigger/Content authoring surface.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Parts", parts)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-parts")
                .code_rust_from_file_region(snippets::parts::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-basic")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Submenu", submenu)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-submenu")
                .code_rust_from_file_region(snippets::submenu::SOURCE, "example"),
            DocSection::new("Shortcuts", shortcuts)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-shortcuts")
                .code_rust_from_file_region(snippets::shortcuts::SOURCE, "example"),
            DocSection::new("Icons", icons)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-icons")
                .code_rust_from_file_region(snippets::icons::SOURCE, "example"),
            DocSection::new("Checkboxes", checkboxes)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-checkboxes")
                .code_rust_from_file_region(snippets::checkboxes::SOURCE, "example"),
            DocSection::new("Checkboxes Icons", checkboxes_icons)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-checkboxes-icons")
                .code_rust_from_file_region(snippets::checkboxes_icons::SOURCE, "example"),
            DocSection::new("Radio Group", radio_group)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-radio-group")
                .code_rust_from_file_region(snippets::radio_group::SOURCE, "example"),
            DocSection::new("Radio Icons", radio_icons)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-radio-icons")
                .code_rust_from_file_region(snippets::radio_icons::SOURCE, "example"),
            DocSection::new("Destructive", destructive)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-destructive")
                .code_rust_from_file_region(snippets::destructive::SOURCE, "example"),
            DocSection::new("Avatar", avatar)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-avatar")
                .code_rust_from_file_region(snippets::avatar::SOURCE, "example"),
            DocSection::new("Complex", complex)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-complex")
                .code_rust_from_file_region(snippets::complex::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-dropdown-menu-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes),
        ],
    );

    vec![body.test_id("ui-gallery-dropdown-menu")]
}
