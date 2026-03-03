use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::input_otp as snippets;

pub(super) fn preview_input_otp(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "This page follows shadcn Input OTP patterns (grouping, separators, and active slot behavior).",
            "Invalid state is modeled via `InputOtp::aria_invalid(true)` (shadcn docs: `aria-invalid`).",
            "API reference: `ecosystem/fret-ui-shadcn/src/input_otp.rs`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Input OTP demo: Simple, Digits Only, With Separator, With Spacing.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .no_shell()
                .test_id_prefix("ui-gallery-input-otp-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Notes", notes),
        ],
    );

    vec![body.test_id("ui-gallery-input-otp-component")]
}
