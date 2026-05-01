# ImUi Response Status Lifecycle v1 - Final Status

Status: closed closeout note
Last updated: 2026-04-20

Related:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-13.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_model_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`

## Decision

This lane closes as the closeout record for the first `ResponseExt` lifecycle vocabulary:

- `activated`, `deactivated`, `edited`, and `deactivated_after_edit` now exist as facade-only
  status on the `fret-ui-kit::imui` response surface.
- The shared `fret-authoring::Response` contract remains unchanged.
- The shipped first slice covers direct pressables, menu items, boolean controls, slider, input
  text, textarea, generic combo triggers, and `combo_model_with_options`.
- The richer trigger-response work for menu/submenu/tab helpers remains owned by the separate
  closed follow-on lanes rather than being folded back into this vocabulary lane.

The landed surface stays inside `ecosystem/fret-ui-kit::imui` and does not widen
`fret-authoring::Response`, `crates/fret-ui`, or the separate key-owner model.

## Proof left behind

- Source / demo proof:
  - `python tools/gate_imui_workstream_source.py`
  - `python tools/gate_imui_facade_teaching_source.py`
- Boundary proof:
  - `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke`
- Focused interaction floor:
  - `cargo nextest run -p fret-imui button_lifecycle_edges_follow_press_session`
  - `cargo nextest run -p fret-imui menu_item_lifecycle_edges_follow_press_session`
  - `cargo nextest run -p fret-imui checkbox_lifecycle_reports_edit_and_deactivated_after_edit`
  - `cargo nextest run -p fret-imui slider_lifecycle_reports_edit_and_deactivated_after_pointer_commit`
  - `cargo nextest run -p fret-imui input_text_lifecycle_tracks_focus_edit_and_blur_edges`
  - `cargo nextest run -p fret-imui textarea_lifecycle_tracks_focus_edit_and_blur_edges`
  - `cargo nextest run -p fret-imui combo_lifecycle_tracks_open_session_edges`
  - `cargo nextest run -p fret-imui combo_model_lifecycle_reports_edit_on_option_pick`

The last closeout decision is that text-entry proof is now explicit for both single-line and
multiline text controls, so this lane no longer needs to stay active just to cover one remaining
value-editing family.

## Residual gap routing

Do not reopen this lane for generic IMUI parity drift.

If future pressure appears, route it by owner:

- key ownership or broader proof-depth questions should move to a new narrow follow-on,
- richer menu-bar/submenu/tab policy remains on separate helper-owned trigger-response lanes,
- and any future helper-family parity work should open a new lane around the specific owner family
  instead of widening this vocabulary closeout.
