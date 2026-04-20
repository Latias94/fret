# ImUi Response Status Lifecycle v1 - Evidence & Gates

Goal: keep the `ResponseExt` lifecycle-vocabulary lane tied to one demo surface, one focused
boundary gate, and one explicit interaction floor instead of turning into another vague `imui`
backlog.

## Evidence anchors (current)

- `docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/M0_BASELINE_AUDIT_2026-04-13.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/TODO.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/MILESTONES.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-stack-fearless-refactor-v1/UIWRITER_RESPONSE_CONTRACT_CLOSEOUT_2026-03-29.md`
- `ecosystem/fret-authoring/src/lib.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
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
- `apps/fret-examples/src/lib.rs`

## First-open repro surfaces

Use these before reading older historical `imui` facade notes in depth:

1. Current response demo surface
   - `cargo run -p fret-demo --bin imui_response_signals_demo`
2. Shared-vs-facade response boundary smoke
   - `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke`
3. Current interaction floor
   - `cargo nextest run -p fret-imui`

## Current focused gates

### P0 source-policy gate

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p0_response_status_lifecycle_follow_on`

This gate currently proves:

- the lane still stays facade-only,
- the first lifecycle quartet remains explicit,
- the lane still points at the correct first-open demo surface,
- and the umbrella still records that this work moved into a narrow follow-on.

### Demo/source gate

- `cargo nextest run -p fret-examples --lib imui_response_signals_demo_keeps_menu_and_combo_lifecycle_proof`

This gate currently proves:

- the first-open response demo still demonstrates the public menu/combo lifecycle surfaces,
- `menu_item_with_options` remains visible as a click-only lifecycle example,
- `combo_with_options` still exposes popup-open activation / deactivation through `ComboResponse`,
- and `combo_model_with_options` still demonstrates `edited` / `deactivated_after_edit` on
  selection commit.

### Response boundary gate

- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke`

This gate currently proves:

- the shared `Response` boundary still compiles cleanly beside `ResponseExt`,
- richer status remains on the facade side,
- the new lifecycle accessors remain part of the facade response surface,
- and future lifecycle work still has to respect that split.

### Current interaction floor

- `cargo nextest run -p fret-imui button_lifecycle_edges_follow_press_session`
- `cargo nextest run -p fret-imui menu_item_lifecycle_edges_follow_press_session`
- `cargo nextest run -p fret-imui checkbox_lifecycle_reports_edit_and_deactivated_after_edit`
- `cargo nextest run -p fret-imui slider_lifecycle_reports_edit_and_deactivated_after_pointer_commit`
- `cargo nextest run -p fret-imui input_text_lifecycle_tracks_focus_edit_and_blur_edges`
- `cargo nextest run -p fret-imui textarea_lifecycle_tracks_focus_edit_and_blur_edges`
- `cargo nextest run -p fret-imui combo_lifecycle_tracks_open_session_edges`
- `cargo nextest run -p fret-imui combo_model_lifecycle_reports_edit_on_option_pick`

This focused package currently proves the first landed lifecycle slice around:

- press-session activation / deactivation on direct buttons,
- press-session activation / deactivation on click-only menu items,
- `edited` and `deactivated_after_edit` on a value-editing checkbox,
- pointer-commit `edited` / `deactivated_after_edit` on `slider_f32_model_with_options`,
- focus-gain `activated`, edit-time `edited`, and blur-time `deactivated_after_edit` on
  `input_text_model_with_options`,
- focus-gain `activated`, edit-time `edited`, and blur-time `deactivated_after_edit` on
  `textarea_model_with_options`,
- popup-open activation / deactivation on generic combo triggers,
- selection-commit `edited` / `deactivated_after_edit` on `combo_model_with_options`,
- and the immediate interaction wiring that those lifecycle edges depend on.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-response-status-lifecycle-v1/WORKSTREAM.json > /dev/null`

## Closeout read

The old focused-floor gap is now closed:

- button/menu/checkbox/combo proof stayed green,
- slider pointer-commit lifecycle now has a focused runtime gate,
- and both single-line and multiline text-entry lifecycle now have focused runtime gates.

Do not respond to future pressure by widening the shared response contract or by bundling
key-owner semantics back into this lane.
