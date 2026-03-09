# shadcn Input OTP audit (Fret)

This audit compares Fret's `ecosystem/fret-ui-shadcn/src/input_otp.rs` and the UI Gallery page
against shadcn/ui v4 `input-otp` composition and docs in `repo-ref/ui`.

## Upstream references (source of truth)

- Base docs: `repo-ref/ui/apps/v4/content/docs/components/base/input-otp.mdx`
- Radix docs: `repo-ref/ui/apps/v4/content/docs/components/radix/input-otp.mdx`
- Registry implementation (new-york-v4): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input-otp.tsx`
- Base form example: `repo-ref/ui/apps/v4/examples/base/input-otp-form.tsx`

## Fret implementation

- Component: `ecosystem/fret-ui-shadcn/src/input_otp.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/input_otp.rs`
- Gallery snippets: `apps/fret-ui-gallery/src/ui/snippets/input_otp/`

## Findings

- Pass: Default-style ownership already matched upstream source.
  - Root inline layout (`flex items-center gap-2`) remains recipe-owned in `InputOtp`.
  - Slot size/chrome (`h-9 w-9`, border merge, `shadow-xs`, active ring) remains recipe-owned.
  - Caller overrides still stay caller-owned through `refine_layout(...)`, `slot_size_px(...)`,
    `slot_text_px(...)`, and related builder surfaces.
- Pass: Upstream composition model is already available in Fret through `InputOTP`,
  `InputOTPGroup`, `InputOTPSlot`, `InputOTPSeparator`, and `InputOtp::into_element_parts(...)`.
- Fixed: `InputOTPSeparator` now emits `SemanticsRole::Separator`, matching upstream
  `role="separator"` instead of remaining a purely visual icon.
- Fixed: `InputOtp` now supports form association parity via `control_id(...)`,
  `labelled_by_element(...)`, `a11y_label(...)`, and `aria_required(...)`.
  - `FieldLabel::for_control(...)` / `FieldDescription::for_control(...)` can now attach
    `labelled-by` / `described-by` semantics to the hidden text input.
  - Label click forwarding uses the same control-registry contract as `Input` / `Textarea`.
- Fixed: The UI Gallery page now includes the upstream `Form` example, so the example order matches
  shadcn docs more closely: Demo, Usage, Pattern, Separator, Disabled, Controlled, Invalid,
  Four Digits, Alphanumeric, Form, RTL.

## Validation

- Focused component gates:
  - `cargo nextest run -p fret-ui-shadcn --lib input_otp_aria_invalid_sets_semantics_invalid input_otp_control_id_uses_registry_labelled_by_and_described_by_elements input_otp_parts_infer_length_and_respect_explicit_separators --status-level fail`
- Gallery compile gate:
  - `cargo check -p fret-ui-gallery --message-format short`

## Conclusion

- Result: `InputOtp` did not need a default-style ownership rewrite.
- Result: The real parity gaps were form association and separator semantics, not slot chrome.
- Result: Gallery docs parity improves materially once the upstream verification-form example is
  present, because it exercises the now-supported `control_id(...)` path instead of only isolated
  OTP demos.
