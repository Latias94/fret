use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::toggle as snippets;

pub(super) fn preview_toggle(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let outline = snippets::outline::render(cx);
    let with_text = snippets::with_text::render(cx);
    let size = snippets::size::render(cx);
    let disabled = snippets::disabled::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/toggle.rs` and `ecosystem/fret-ui-shadcn/src/toggle_group.rs`.",
            "Use Outline when toggle sits in dense toolbars and needs stronger boundaries.",
            "Prefer icon + short text labels so state remains understandable in compact layouts.",
            "Keep `a11y_label` explicit for icon-heavy toggles to improve accessibility tree quality.",
            "For quick keyboard validation, tab through toggles and verify pressed visual parity.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Toggle docs order: Demo, Outline, With Text, Size, Disabled, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A small outline toggle with an icon + label.")
                .max_w(Px(480.0))
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Outline", outline)
                .description("Outline variant for dense toolbars.")
                .max_w(Px(480.0))
                .code_rust_from_file_region(snippets::outline::SOURCE, "example"),
            DocSection::new("With Text", with_text)
                .description("Default variant with icon + text.")
                .max_w(Px(480.0))
                .code_rust_from_file_region(snippets::with_text::SOURCE, "example"),
            DocSection::new("Size", size)
                .description("Size presets: Sm / Default / Lg.")
                .max_w(Px(480.0))
                .code_rust_from_file_region(snippets::size::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Disabled toggles remain readable and non-interactive.")
                .max_w(Px(480.0))
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Toggle content order and alignment under RTL.")
                .max_w(Px(480.0))
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .description("API reference pointers and accessibility notes."),
        ],
    );

    vec![body.test_id("ui-gallery-toggle")]
}
