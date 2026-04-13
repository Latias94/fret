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
- `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
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

### Response boundary gate

- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke`

This gate currently proves:

- the shared `Response` boundary still compiles cleanly beside `ResponseExt`,
- richer status remains on the facade side,
- and future lifecycle work still has to respect that split.

### Current interaction floor

- `cargo nextest run -p fret-imui`

This package currently provides the existing immediate interaction floor around:

- click variants,
- context-menu request,
- drag lifecycle,
- long-press / holding,
- and other current `ResponseExt` semantics that the new lifecycle vocabulary must not regress.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-response-status-lifecycle-v1/WORKSTREAM.json > /dev/null`

## Missing gates before closure

Before claiming this lane is closed, add:

- focused interaction tests for `activated`, `deactivated`, `edited`, and
  `deactivated_after_edit`,
- one demo/source gate that freezes the new signals on the first-open response demo,
- and any extra focused tests needed by the first landed implementation slice.

Do not respond to that gap by widening the shared response contract or by bundling key-owner
semantics into this lane.
