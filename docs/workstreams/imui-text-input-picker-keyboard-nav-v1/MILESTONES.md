# ImUi Text Input Picker Keyboard Navigation v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Boundary Decision

Closed. Keyboard navigation remains in `fret-ui-kit::imui` because it is picker policy over existing
input, popup, and selectable mechanisms.

Exit evidence:

- `docs/workstreams/imui-text-input-picker-recipe-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`

## M1 - Active Candidate Navigation

Closed. The picker owns only active-source-index and one-shot picked state. Candidate values remain
the caller's slice.

Exit evidence:

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`

## M2 - App-Facing Proof

Closed. `fret-imui` tests prove completion commit, history wrap, and no-candidate Enter
pass-through.

Exit evidence:

- `ecosystem/fret-imui/src/tests/models_text.rs`
- `cargo nextest run -p fret-imui input_text_completion_picker_keyboard_navigation input_text_history_picker_keyboard_navigation input_text_picker_keyboard_navigation --no-fail-fast`
