# ImUi Collection Second Proof Surface v1 - Evidence & Gates

Goal: keep the closed second proof-surface follow-on tied to existing shell-mounted demos, one
explicit source-policy floor, and one bounded evidence set before anyone argues for shared helper
or runtime growth.

## Evidence anchors (current)

- `docs/workstreams/imui-collection-second-proof-surface-v1/DESIGN.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/M0_BASELINE_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/M2_SHELL_MOUNTED_COLLECTION_SURFACE_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/TODO.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/MILESTONES.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-command-package-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `apps/fret-examples/tests/workspace_shell_pane_proof_surface.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `tools/gate_imui_workstream_source.py`

## First-open repro surfaces

1. Primary landed shell-mounted second proof surface
   - `cargo run -p fret-demo --bin editor_notes_demo`
2. Supporting shell-mounted proof surface
   - `cargo run -p fret-demo --bin workspace_shell_demo`
3. Current closeout source-policy and surface floor
   - `python tools/gate_imui_workstream_source.py`
   - `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test workspace_shell_pane_proof_surface --test workspace_shell_editor_rail_surface --no-fail-fast`

## Current focused gates

### Lane-local closeout source-policy floor

- `python tools/gate_imui_workstream_source.py`

This floor currently proves:

- the repo keeps the second proof-surface lane explicit after command-package closeout,
- `editor_notes_demo.rs` stays the primary shell-mounted candidate,
- `editor_notes_demo.rs` now carries a real `Scene collection` surface with stable collection
  summary/list test ids,
- `workspace_shell_demo.rs` stays supporting shell-mounted evidence,
- no dedicated asset-grid/file-browser demo is implied,
- and this lane closes on a no-helper-widening verdict because the two proof surfaces do not yet
  demand the same shared helper.

### Supporting shell-mounted surface floor

- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test workspace_shell_pane_proof_surface --test workspace_shell_editor_rail_surface --no-fail-fast`

This floor currently proves:

- the existing shell-mounted proof surfaces remain explicit and reviewable,
- `editor_notes_demo.rs` carries the landed second collection proof surface,
- the second proof follow-on stays anchored to real demos rather than a synthetic showcase,
- and the repo keeps both the smaller candidate and the broader supporting shell proof visible.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-collection-second-proof-surface-v1/WORKSTREAM.json > /dev/null`
