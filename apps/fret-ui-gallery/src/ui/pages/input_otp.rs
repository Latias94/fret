use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::input_otp as snippets;

pub(super) fn preview_input_otp(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let pattern = snippets::pattern::render(cx);
    let separator = snippets::separator::render(cx);
    let disabled = snippets::disabled::render(cx);
    let controlled = snippets::controlled::render(cx);
    let invalid = snippets::invalid::render(cx);
    let four_digits = snippets::four_digits::render(cx);
    let alphanumeric = snippets::alphanumeric::render(cx);
    let form = snippets::form::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/input_otp.rs`.",
        "Default root gap and slot chrome ownership already match upstream source; the meaningful parity work here was form association (`control_id`) and separator semantics.",
        "First-party examples now prefer the compact `InputOTP::new(model)` root builder with `length(...)` and `group_size(...)`; `InputOTPGroup` / `InputOTPSlot` / `InputOTPSeparator` plus `into_element_parts(...)` remain the upstream-shaped bridge when callers explicitly want that shape.",
        "Invalid state is modeled via `InputOtp::aria_invalid(true)` (shadcn docs: `aria-invalid`).",
        "Pattern parity: `InputOtpPattern::DigitsAndChars` mirrors shadcn `REGEXP_ONLY_DIGITS_AND_CHARS` outcomes.",
        "`InputOTPSeparator` now maps to separator semantics, and `FieldLabel::for_control(...)` can focus the hidden text input via `InputOtp::control_id(...)`.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes).description("API surface and parity notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("shadcn demo: grouped OTP slots backed by a single input model.")
        .no_shell()
        .test_id_prefix("ui-gallery-input-otp-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for the compact root builder; parts remain available as the upstream-shaped bridge.")
        .test_id_prefix("ui-gallery-input-otp-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let pattern = DocSection::build(cx, "Pattern", pattern)
        .description("Alphanumeric OTP filtering using shadcn-like patterns.")
        .code_rust_from_file_region(snippets::pattern::SOURCE, "example");
    let separator = DocSection::build(cx, "Separator", separator)
        .description("Multiple group separators between 2-digit chunks.")
        .code_rust_from_file_region(snippets::separator::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled", disabled)
        .description("Disabled OTP blocks focus/typing and uses muted styling.")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let controlled = DocSection::build(cx, "Controlled", controlled)
        .description("OTP input is model-controlled; this section echoes the current value.")
        .code_rust_from_file_region(snippets::controlled::SOURCE, "example");
    let invalid = DocSection::build(cx, "Invalid", invalid)
        .description("Invalid state uses `aria_invalid(true)` to apply destructive chrome.")
        .code_rust_from_file_region(snippets::invalid::SOURCE, "example");
    let four_digits = DocSection::build(cx, "Four Digits", four_digits)
        .description("A common PIN/4-digit verification pattern.")
        .code_rust_from_file_region(snippets::four_digits::SOURCE, "example");
    let alphanumeric = DocSection::build(cx, "Alphanumeric", alphanumeric)
        .description("Two groups separated by a minus icon, accepting letters and digits.")
        .code_rust_from_file_region(snippets::alphanumeric::SOURCE, "example");
    let form = DocSection::build(cx, "Form", form)
        .description(
            "Card + field composition aligned with the upstream verification form example.",
        )
        .code_rust_from_file_region(snippets::form::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Same OTP surface under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Input OTP docs order: Demo, Usage, Pattern, Separator, Disabled, Controlled, Invalid, Four Digits, Alphanumeric, Form, RTL.",
        ),
        vec![
            demo,
            usage,
            pattern,
            separator,
            disabled,
            controlled,
            invalid,
            four_digits,
            alphanumeric,
            form,
            rtl,
            notes,
        ],
    );

    vec![
        body.test_id("ui-gallery-input-otp-component")
            .into_element(cx),
    ]
}
