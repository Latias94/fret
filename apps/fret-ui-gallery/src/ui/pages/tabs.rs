use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::tabs as snippets;

pub(super) fn preview_tabs(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let list = snippets::list::render(cx);
    let disabled = snippets::disabled::render(cx);
    let icons = snippets::icons::render(cx);
    let line = snippets::line::render(cx);
    let vertical = snippets::vertical::render(cx);
    let vertical_line = snippets::vertical_line::render(cx);
    let extras = snippets::extras::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows `tabs-demo.tsx` (new-york-v4) order: Demo, list-only, disabled, icons, line, vertical, vertical line.",
            "Password fields use `Input::password()` to mirror `type=\\\"password\\\"` in shadcn/ui examples.",
            "API reference: `ecosystem/fret-ui-shadcn/src/tabs.rs`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("A set of layered sections of content that are displayed one at a time."),
        vec![
            DocSection::new("Demo", demo)
                .description("Account/password card example with inputs and footer actions.")
                .test_id_prefix("ui-gallery-tabs-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("List", list)
                .description("Tabs list without any mounted content.")
                .test_id_prefix("ui-gallery-tabs-list")
                .code_rust_from_file_region(snippets::list::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Disable individual triggers.")
                .test_id_prefix("ui-gallery-tabs-disabled")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Icons", icons)
                .description("Compose icons into triggers.")
                .test_id_prefix("ui-gallery-tabs-icons")
                .code_rust_from_file_region(snippets::icons::SOURCE, "example"),
            DocSection::new("Line", line)
                .description("Line-style list with transparent background.")
                .test_id_prefix("ui-gallery-tabs-line")
                .code_rust_from_file_region(snippets::line::SOURCE, "example"),
            DocSection::new("Vertical", vertical)
                .description("Vertical orientation (Radix parity).")
                .test_id_prefix("ui-gallery-tabs-vertical")
                .code_rust_from_file_region(snippets::vertical::SOURCE, "example"),
            DocSection::new("Vertical (Line)", vertical_line)
                .description("Vertical + line style.")
                .test_id_prefix("ui-gallery-tabs-vertical-line")
                .code_rust_from_file_region(snippets::vertical_line::SOURCE, "example"),
            DocSection::new("Extras", extras)
                .description("Fret-specific regression gates (flex-1 triggers + RTL).")
                .test_id_prefix("ui-gallery-tabs-extras")
                .code_rust_from_file_region(snippets::extras::SOURCE, "example"),
            DocSection::new("Notes", notes).description("Parity notes and references."),
        ],
    );

    vec![body.test_id("ui-gallery-tabs")]
}
