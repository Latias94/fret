use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::input_otp as snippets;

pub(super) fn preview_input_otp(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let compact_builder = snippets::compact_builder::render(cx);
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

    let about = doc_layout::notes_block([
        "shadcn Input OTP is an accessible one-time-password surface with copy-paste support; Fret matches that outcome with one hidden text input and projected slot chrome.",
        "The docs-shaped composition lane is `InputOTPGroup` / `InputOTPSlot` / `InputOTPSeparator`; the page now stays on that lane for the docs-path examples, while `Compact Builder` remains shorter when groups are regular and caller-owned layout is simple.",
        "Slot chrome, fake caret, and focus choreography stay recipe/runtime-owned, while page width caps such as `max-w-xs` remain caller-owned.",
    ]);
    let api_reference = doc_layout::notes_block([
        "`InputOTP::new(model)` owns the shared OTP value model; `length(...)` mirrors upstream `maxLength`.",
        "`pattern(...)` and `InputOtpPattern::{Digits, DigitsAndChars}` cover the documented filtering lane.",
        "`InputOTPSlot::aria_invalid(true)` mirrors the upstream slot-level invalid lane, while root `control_id(...)`, `labelled_by_element(...)`, `a11y_label(...)`, and `required(...)` cover form association and accessibility.",
        "`InputOTP::required(true)` keeps required semantics on the hidden root OTP control; any visible required marker remains caller-owned label composition around the projected slots.",
        "`InputOTPGroup` / `InputOTPSlot` / `InputOTPSeparator` plus `into_element_parts(...)` already cover the docs-shaped composition bridge, so a separate generic children API is not needed here.",
    ]);
    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/input_otp.rs`.",
        "Default root gap and slot chrome ownership already match upstream source; the meaningful parity work here was slot-level invalid, form association (`control_id`), and making the docs-facing teaching surface stay on the parts lane end-to-end.",
        "`Demo` through `RTL` now stay on the upstream parts-shaped docs lane, while `Compact Builder` keeps `InputOTP::new(model)` plus `group_size(...)` visible as the explicit Fret shorthand follow-up.",
        "`InputOtpPart` now accepts `InputOTPGroup` / `InputOTPSlot` / `InputOTPSeparator` via `From`, `InputOTPSlot::aria_invalid(true)` mirrors the invalid docs lane, and `InputOTPGroup::{slot_size_px, slot_text_px, slot_line_height_px}` covers the verification-form slot metrics without introducing another root composition surface.",
        "Pattern parity: `InputOtpPattern::DigitsAndChars` mirrors shadcn `REGEXP_ONLY_DIGITS_AND_CHARS` outcomes.",
        "`InputOTPSeparator` now maps to separator semantics, and `FieldLabel::for_control(...)` can focus the hidden text input via `InputOtp::control_id(...)`.",
    ]);
    let about = DocSection::build(cx, "About", about)
        .no_shell()
        .test_id_prefix("ui-gallery-input-otp-about")
        .description("Background, ownership notes, and why the page teaches two authoring lanes.");
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-input-otp-api-reference")
        .description("Public surface summary and ownership notes.");
    let notes = DocSection::build(cx, "Notes", notes).description("API surface and parity notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("shadcn demo: grouped OTP slots using the upstream-shaped parts lane over one shared OTP model.")
        .no_shell()
        .test_id_prefix("ui-gallery-input-otp-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Upstream shadcn docs shape using `InputOTPGroup`, `InputOTPSlot`, and `InputOTPSeparator` on the existing parts bridge.")
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
        .description("Invalid state mirrors upstream slot-level `InputOTPSlot::aria_invalid(true)` and keeps error copy caller-owned via `FieldError` composition.")
        .code_rust_from_file_region(snippets::invalid::SOURCE, "example");
    let four_digits = DocSection::build(cx, "Four Digits", four_digits)
        .description("A common PIN/4-digit verification pattern.")
        .code_rust_from_file_region(snippets::four_digits::SOURCE, "example");
    let alphanumeric = DocSection::build(cx, "Alphanumeric", alphanumeric)
        .description("Two groups separated by a minus icon, accepting letters and digits.")
        .code_rust_from_file_region(snippets::alphanumeric::SOURCE, "example");
    let form = DocSection::build(cx, "Form", form)
        .description(
            "Card + field composition aligned with the upstream verification form example, including group-level slot sizing and root `InputOTP::required(true)` on the docs-shaped parts lane.",
        )
        .code_rust_from_file_region(snippets::form::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Same OTP surface under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let compact_builder = DocSection::build(cx, "Compact Builder", compact_builder)
        .test_id_prefix("ui-gallery-input-otp-compact-builder")
        .description("Compact Fret shorthand for common app call sites with regular OTP groups.")
        .code_rust_from_file_region(snippets::compact_builder::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Input OTP docs path first: Demo, About, Usage, Pattern, Separator, Disabled, Controlled, Invalid, Four Digits, Alphanumeric, Form, RTL, API Reference. `Compact Builder` stays as the explicit Fret shorthand follow-up after those docs-shaped examples.",
        ),
        vec![
            demo,
            about,
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
            api_reference,
            compact_builder,
            notes,
        ],
    );

    let body = body.test_id("ui-gallery-input-otp-component");
    vec![body.into_element(cx)]
}
