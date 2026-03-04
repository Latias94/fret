use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::input_otp as snippets;

pub(super) fn preview_input_otp(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let pattern = snippets::pattern::render(cx);
    let separator = snippets::separator::render(cx);
    let disabled = snippets::disabled::render(cx);
    let controlled = snippets::controlled::render(cx);
    let invalid = snippets::invalid::render(cx);
    let four_digits = snippets::four_digits::render(cx);
    let alphanumeric = snippets::alphanumeric::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "This page follows shadcn Input OTP patterns (grouping, separators, and active slot behavior).",
            "Invalid state is modeled via `InputOtp::aria_invalid(true)` (shadcn docs: `aria-invalid`).",
            "API reference: `ecosystem/fret-ui-shadcn/src/input_otp.rs`.",
            "Pattern parity: `InputOtpPattern::DigitsAndChars` mirrors shadcn `REGEXP_ONLY_DIGITS_AND_CHARS` outcomes.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Input OTP docs order: Demo, Pattern, Separator, Disabled, Controlled, Invalid, Four Digits, Alphanumeric, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .no_shell()
                .test_id_prefix("ui-gallery-input-otp-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Pattern", pattern)
                .description("Alphanumeric OTP filtering using shadcn-like patterns.")
                .code_rust_from_file_region(snippets::pattern::SOURCE, "example"),
            DocSection::new("Separator", separator)
                .description("Multiple group separators between 2-digit chunks.")
                .code_rust_from_file_region(snippets::separator::SOURCE, "example"),
            DocSection::new("Disabled", disabled)
                .description("Disabled OTP blocks focus/typing and uses muted styling.")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Controlled", controlled)
                .description(
                    "OTP input is model-controlled; this section echoes the current value.",
                )
                .code_rust_from_file_region(snippets::controlled::SOURCE, "example"),
            DocSection::new("Invalid", invalid)
                .description("Invalid state uses `aria_invalid(true)` to apply destructive chrome.")
                .code_rust_from_file_region(snippets::invalid::SOURCE, "example"),
            DocSection::new("Four Digits", four_digits)
                .description("A common PIN/4-digit verification pattern.")
                .code_rust_from_file_region(snippets::four_digits::SOURCE, "example"),
            DocSection::new("Alphanumeric", alphanumeric)
                .description("Two groups separated by a minus icon, accepting letters and digits.")
                .code_rust_from_file_region(snippets::alphanumeric::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Same OTP surface under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes),
        ],
    );

    vec![body.test_id("ui-gallery-input-otp-component")]
}
