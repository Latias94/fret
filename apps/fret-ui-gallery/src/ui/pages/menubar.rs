use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::menubar as snippets;

pub(super) fn preview_menubar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let checkbox = snippets::checkbox::render(cx);
    let radio = snippets::radio::render(cx);
    let submenu = snippets::submenu::render(cx);
    let with_icons = snippets::with_icons::render(cx);
    let rtl = snippets::rtl::render(cx);
    let parts = snippets::parts::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows the upstream shadcn Menubar docs (v4 Base UI): Demo, Usage, Checkbox, Radio, Submenu, With Icons, RTL.",
            "This page also includes a `Parts` section for the Trigger/Content authoring adapter surface.",
            "Menubar already has a usable parts bridge, so the remaining parity gap here is mostly documentation clarity rather than missing mechanism.",
            "Keep `ui-gallery-menubar-*` test IDs stable; multiple diag scripts depend on them.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Menubar examples, aligned with upstream shadcn docs (Base UI variant), plus a parts adapter demo.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-menubar-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable shadcn-style composition reference for Menubar.")
                .test_id_prefix("ui-gallery-menubar-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Checkbox", checkbox)
                .test_id_prefix("ui-gallery-menubar-checkbox")
                .code_rust_from_file_region(snippets::checkbox::SOURCE, "example"),
            DocSection::new("Radio", radio)
                .test_id_prefix("ui-gallery-menubar-radio")
                .code_rust_from_file_region(snippets::radio::SOURCE, "example"),
            DocSection::new("Submenu", submenu)
                .test_id_prefix("ui-gallery-menubar-submenu")
                .code_rust_from_file_region(snippets::submenu::SOURCE, "example"),
            DocSection::new("With Icons", with_icons)
                .test_id_prefix("ui-gallery-menubar-with-icons")
                .code_rust_from_file_region(snippets::with_icons::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-menubar-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Parts", parts)
                .test_id_prefix("ui-gallery-menubar-parts")
                .code_rust_from_file_region(snippets::parts::SOURCE, "example"),
            DocSection::new("Notes", notes),
        ],
    );

    vec![body.test_id("ui-gallery-menubar-component")]
}
