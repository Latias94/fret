use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::menubar as snippets;

pub(super) fn preview_menubar(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let checkbox = snippets::checkbox::render(cx);
    let radio = snippets::radio::render(cx);
    let submenu = snippets::submenu::render(cx);
    let with_icons = snippets::with_icons::render(cx);
    let rtl = snippets::rtl::render(cx);
    let parts = snippets::parts::render(cx);

    let notes = doc_layout::notes_block([
        "Preview follows the upstream shadcn Menubar docs (v4 Base UI): Demo, Usage, Checkbox, Radio, Submenu, With Icons, RTL.",
        "Compact Fret-first root authoring uses `Menubar::new([MenubarMenu::new(...).entries([...])])`.",
        "`MenubarTrigger::new(...).into_menu().entries_parts(...)` remains the upstream-shaped copyable lane; the `Parts` section is a focused adapter example on that same lane rather than an advanced escape hatch.",
        "Menubar already has a usable parts bridge, so the remaining parity gap here is mostly documentation clarity rather than missing mechanism.",
        "Keep `ui-gallery-menubar-*` test IDs stable; multiple diag scripts depend on them.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes).test_id_prefix("ui-gallery-menubar-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .test_id_prefix("ui-gallery-menubar-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Default upstream-shaped copyable composition reference for Menubar.")
        .test_id_prefix("ui-gallery-menubar-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let checkbox = DocSection::build(cx, "Checkbox", checkbox)
        .test_id_prefix("ui-gallery-menubar-checkbox")
        .code_rust_from_file_region(snippets::checkbox::SOURCE, "example");
    let radio = DocSection::build(cx, "Radio", radio)
        .test_id_prefix("ui-gallery-menubar-radio")
        .code_rust_from_file_region(snippets::radio::SOURCE, "example");
    let submenu = DocSection::build(cx, "Submenu", submenu)
        .test_id_prefix("ui-gallery-menubar-submenu")
        .code_rust_from_file_region(snippets::submenu::SOURCE, "example");
    let with_icons = DocSection::build(cx, "With Icons", with_icons)
        .test_id_prefix("ui-gallery-menubar-with-icons")
        .code_rust_from_file_region(snippets::with_icons::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .test_id_prefix("ui-gallery-menubar-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let parts = DocSection::build(cx, "Parts", parts)
        .description("Focused Trigger/Content adapter example on the same copyable parts lane.")
        .test_id_prefix("ui-gallery-menubar-parts")
        .code_rust_from_file_region(snippets::parts::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Menubar examples, aligned with upstream shadcn docs (Base UI variant), plus a focused parts adapter example on the same copyable lane.",
        ),
        vec![
            demo, usage, checkbox, radio, submenu, with_icons, rtl, parts, notes,
        ],
    );

    let body = body.test_id("ui-gallery-menubar-component");
    vec![body.into_element(cx)]
}
