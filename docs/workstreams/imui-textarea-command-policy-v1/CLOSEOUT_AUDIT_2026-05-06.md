# ImUi Textarea Command Policy v1 Closeout Audit - 2026-05-06

Status: closed closeout record.

## What Shipped

- Added `TextAreaSubmitKey` with Ctrl+Enter default submit and Enter opt-in submit policy.
- Added app-owned `submit_command` and `cancel_command` fields to `TextAreaOptions`.
- Added `submit_cancel_command_repeat` to keep repeated keydown suppressed unless explicitly
  enabled.
- Installed focused capture-phase key routing for textarea submit/cancel commands in
  `fret-ui-kit::imui`.
- Kept `crates/fret-ui::TextAreaProps` unchanged so this remains a policy-layer slice.
- Added focused `fret-imui` tests plus a public `fret-ui-kit` options smoke.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/tests/imui_textarea_smoke.rs`
- `ecosystem/fret-imui/src/tests/models_text_area.rs`
- `docs/audits/imui-imgui-gap-audit-2026-04-22.md`

## Gates Run

```bash
cargo fmt --package fret-ui-kit --package fret-imui
cargo nextest run -p fret-imui textarea_submit_and_cancel_commands_dispatch_from_focused_multiline_field textarea_enter_submit_policy_can_opt_into_enter_and_repeat textarea_enter_submit_policy_consumes_repeat_when_repeat_is_disabled --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui textarea_command_policy_options_compile --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
```

## Known Unrelated Failure

`cargo nextest run -p fret-imui --no-fail-fast` currently fails
`tests::composition::table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells` with
`status x header vs row0 drifted: left=335, right=17`. This lane does not modify table helpers or
the composition test module.

## Residual Gaps

- Ctrl+Enter newline insertion is not implemented; Ctrl+Enter is the default submit command policy.
- Multiline selection/range APIs remain a separate editor-text surface question.
- Editor-owned undo stacks and rich multiline history remain separate follow-ons.
- Platform IME and accessibility announcement depth remain broader text-surface work.
