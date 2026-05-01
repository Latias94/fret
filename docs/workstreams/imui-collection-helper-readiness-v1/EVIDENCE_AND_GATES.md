# ImUi Collection Helper Readiness v1 - Evidence & Gates

Goal: keep helper-readiness separate from helper implementation and prevent product policy from
slipping into generic IMUI.

## Evidence Anchors

- `docs/workstreams/imui-collection-helper-readiness-v1/DESIGN.md`
- `docs/workstreams/imui-collection-helper-readiness-v1/TODO.md`
- `docs/workstreams/imui-collection-helper-readiness-v1/MILESTONES.md`
- `docs/workstreams/imui-collection-helper-readiness-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-helper-readiness-v1/M1_CANDIDATE_SEAM_AUDIT_2026-04-24.md`
- `docs/workstreams/imui-collection-helper-readiness-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `docs/workstreams/imui-collection-helper-readiness-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_command_package_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `tools/gate_imui_workstream_source.py`

## First-Open Repro Surfaces

1. Collection-first asset-browser proof
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`
2. Shell-mounted `Scene collection` proof
   - `cargo run -p fret-demo --bin editor_notes_demo`
3. Source-policy gate
   - `python tools/gate_imui_workstream_source.py`

## Focused Gates

- `python tools/gate_imui_workstream_source.py`
- `cargo nextest run -p fret-examples --test imui_editor_collection_command_package_surface --test editor_notes_editor_rail_surface --no-fail-fast`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-collection-helper-readiness-v1/WORKSTREAM.json > /dev/null`
- `git diff --check`

## Current Non-Goals

- Do not add a shared `collection(...)`, `collection_list(...)`, or `collection_commands(...)`
  helper in this lane.
- Do not widen `fret-imui` or `crates/fret-ui`.
- Do not make the compact `Scene collection` outline inherit asset-browser grid policy.

## M1 Audit Result

`M1_CANDIDATE_SEAM_AUDIT_2026-04-24.md` keeps shared helper widening closed for now:

- generic collection container/list helpers are not helper-ready,
- generic collection command helpers remain app-owned policy,
- selection summary text is not worth extracting yet,
- stable collection test IDs are documentation/recipe guidance rather than a public helper API.

## Closeout Result

`CLOSEOUT_AUDIT_2026-04-24.md` closes the lane on a no-helper-widening verdict. Reopen pressure
must move to a different narrow follow-on with one exact helper shape and proof that both current
collection surfaces need it.
