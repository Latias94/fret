use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::alert as snippets;

pub(super) fn preview_alert(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let basic = snippets::basic::render(cx);
    let destructive = snippets::destructive::render(cx);
    let action = snippets::action::render(cx);
    let custom_colors = snippets::custom_colors::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/alert.rs` and `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`.",
            "Keep alert copy concise and action-oriented; reserve longer guidance for Dialog or Sheet.",
            "Use `Destructive` only for high-risk or blocking failures to preserve visual hierarchy.",
            "Validate RTL + narrow layout so icon/title/description remain readable in editor sidebars.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Alert docs order: Demo, Basic (docs example), Destructive, Action, Custom Colors, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A small set of inline alerts for different message tones.")
                .test_id_prefix("ui-gallery-alert")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("Upstream shadcn docs example (icon + title + description).")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Destructive", destructive)
                .description("Destructive variant for critical errors.")
                .code_rust_from_file_region(snippets::destructive::SOURCE, "example"),
            DocSection::new("Action", action)
                .description("Use `AlertAction` to pin a top-right action inside the alert.")
                .code_rust_from_file_region(snippets::action::SOURCE, "example"),
            DocSection::new("Custom Colors", custom_colors)
                .description("Custom chrome override for special emphasis.")
                .code_rust_from_file_region(snippets::custom_colors::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Alert layout under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("API reference pointers and caveats."),
        ],
    );

    vec![body.test_id("ui-gallery-alert-component")]
}
