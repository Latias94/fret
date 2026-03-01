use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::menubar as snippets;

pub(super) fn preview_menubar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let checkbox = snippets::checkbox::render(cx);
    let radio = snippets::radio::render(cx);
    let submenu = snippets::submenu::render(cx);
    let with_icons = snippets::with_icons::render(cx);
    let rtl = snippets::rtl::render(cx);
    let parts = snippets::parts::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows the upstream shadcn Menubar docs (v4 Base UI): Demo, Checkbox, Radio, Submenu, With Icons, RTL.",
            "This page also includes a `Parts` section for the Trigger/Content authoring adapter surface.",
            "Examples are snippet-backed: Preview ≡ Code (single source of truth).",
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
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-menubar-demo")
                .code_rust_from_file_region(include_str!("../snippets/menubar/demo.rs"), "example"),
            DocSection::new("Checkbox", checkbox)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-menubar-checkbox")
                .code_rust_from_file_region(
                    include_str!("../snippets/menubar/checkbox.rs"),
                    "example",
                ),
            DocSection::new("Radio", radio)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-menubar-radio")
                .code_rust_from_file_region(include_str!("../snippets/menubar/radio.rs"), "example"),
            DocSection::new("Submenu", submenu)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-menubar-submenu")
                .code_rust_from_file_region(
                    include_str!("../snippets/menubar/submenu.rs"),
                    "example",
                ),
            DocSection::new("With Icons", with_icons)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-menubar-with-icons")
                .code_rust_from_file_region(
                    include_str!("../snippets/menubar/with_icons.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-menubar-rtl")
                .code_rust_from_file_region(include_str!("../snippets/menubar/rtl.rs"), "example"),
            DocSection::new("Parts", parts)
                .max_w(Px(520.0))
                .test_id_prefix("ui-gallery-menubar-parts")
                .code_rust_from_file_region(include_str!("../snippets/menubar/parts.rs"), "example"),
            DocSection::new("Notes", notes).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-menubar-component")]
}

