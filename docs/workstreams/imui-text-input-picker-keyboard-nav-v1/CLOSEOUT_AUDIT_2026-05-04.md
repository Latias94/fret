# ImUi Text Input Picker Keyboard Navigation v1 Closeout Audit - 2026-05-04

Status: Closed

## Verdict

Closed. IMUI text input completion/history picker recipes now support keyboard active-candidate
navigation and commit without adding completion/history policy to `crates/fret-ui`.

## What Shipped

- `InputTextPickerOptions::keyboard_navigation`
- `InputTextPickerOptions::keyboard_repeat`
- Picker-owned active source index and one-shot keyboard picked state.
- ArrowDown/ArrowUp active movement with wrap.
- Enter/NumpadEnter commit of the active candidate.
- No-candidate pass-through so Enter submit remains available.

## Layering Decision

The input widget remains the editing mechanism. The picker recipe wraps the input and popup surface
at the IMUI policy layer, installing keyboard handling on the popup content owner where overlay key
arbitration can observe it. Candidate values, history storage, and ranking stay app-owned.

## Evidence

- Options: `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- Implementation: `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs`,
  `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- Tests: `ecosystem/fret-imui/src/tests/models_text.rs`

## Follow-On Policy

Do not reopen this lane for editor-owned ranking, persistent history, dismissed-query behavior,
multiline conflicts, or deeper accessibility audits. Those need narrower follow-ons with their own
repro and gates.
