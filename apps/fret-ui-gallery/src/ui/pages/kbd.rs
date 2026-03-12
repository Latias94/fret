use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::kbd as snippets;

pub(super) fn preview_kbd(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let group = snippets::group::render(cx);
    let button = snippets::button::render(cx);
    let tooltip = snippets::tooltip::render(cx);
    let input_group = snippets::input_group::render(cx);
    let rtl = snippets::rtl::render(cx);

    let usage = doc_layout::muted_full_width(
        cx,
        "Use `Kbd` for a single key token and `KbdGroup` for shortcut chords or grouped hints.",
    );

    let api_reference = doc_layout::notes(
        cx,
        [
            "`Kbd::new(\"Ctrl\")` covers the default textual key token, while `Kbd::from_children([...])` covers icon-based keys such as Command or Arrow icons.",
            "`KbdGroup::new([...])` is the public surface for grouped shortcuts and keeps spacing/chrome consistent across adjacent tokens.",
            "Kbd chrome, fixed height, and text centering remain recipe-owned; composition into buttons, tooltips, and input-group addons stays caller-owned.",
            "No extra generic `asChild` / `compose()` surface is needed here: upstream composition already happens around `Kbd`, and Fret matches that layering directly.",
            "Keep `ui-gallery-kbd-*` test ids stable for diag scripts and future web-vs-fret gates.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Kbd docs path first: Demo, Usage, Group, Button, Tooltip, Input Group, RTL, and API Reference.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Two shortcut display patterns: icon-based chord and textual chord.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `Kbd` and `KbdGroup`.")
                .code_rust(
                    r#"use fret_ui_shadcn::{facade as shadcn, prelude::*};

shadcn::KbdGroup::new([
    shadcn::Kbd::new("Ctrl").into_element(cx),
    shadcn::Kbd::new("B").into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Group", group)
                .description("Use `KbdGroup` to keep spacing consistent across tokens.")
                .code_rust_from_file_region(snippets::group::SOURCE, "example"),
            DocSection::new("Button", button)
                .description("kbd tokens can be composed into button labels for discoverability.")
                .code_rust_from_file_region(snippets::button::SOURCE, "example"),
            DocSection::new("Tooltip", tooltip)
                .description("Tooltips often include shortcut hints for expert users.")
                .code_rust_from_file_region(snippets::tooltip::SOURCE, "example"),
            DocSection::new("Input Group", input_group)
                .description("Trailing kbd hints can be rendered inside an input group.")
                .code_rust_from_file_region(snippets::input_group::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("kbd token order should respect right-to-left direction context.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .description("Public surface summary and ownership notes."),
        ],
    )
    .test_id("ui-gallery-kbd-component");

    vec![body]
}
