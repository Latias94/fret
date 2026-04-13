# ImUi Workbench Shell Closure v1 - Evidence & Gates

Goal: keep P1 workbench-shell closure tied to real proof surfaces and existing shell gates instead
of turning into a vague product backlog.

## Evidence anchors (current)

- `docs/workstreams/imui-workbench-shell-closure-v1/DESIGN.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/M0_BASELINE_AUDIT_2026-04-13.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/M1_DEFAULT_WORKBENCH_ASSEMBLY_DECISION_2026-04-13.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/TODO.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/MILESTONES.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/DESIGN.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`
- `docs/workstreams/workspace-tabstrip-editor-grade-v1/DESIGN.md`
- `docs/workstreams/workspace-shell-tabstrip-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/editor-tabstrip-unification-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-ui-gallery/src/driver/render_flow.rs`
- `apps/fret-ui-gallery/src/driver/chrome.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `apps/fret-examples/src/lib.rs`
- `tools/diag-scripts/suites/diag-hardening-smoke-workspace/suite.json`

## First-open repro surfaces

Use these before reading older shell-specific workstreams in depth:

1. Primary workbench shell proof
   - `cargo run -p fret-demo --bin workspace_shell_demo`
2. Minimal shell-mounted rail proof
   - `cargo run -p fret-demo --bin editor_notes_demo`
3. Supporting docking/editor immediate proof
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`

## Current focused gates

### P1 source-policy gates

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_workbench_shell_proof_matrix immediate_mode_workstream_freezes_the_p1_shell_diag_smoke_minimum immediate_mode_workstream_freezes_the_p1_default_workbench_assembly_decision`

This package currently proves:

- the umbrella lane still names the correct P1 proof roster,
- `workspace_shell_demo` remains the default coherent workbench-shell proof,
- `editor_notes_demo` remains the minimal shell-mounted rail proof,
- the promoted P1 shell smoke floor stays explicit,
- and the current P1 shell-assembly verdict remains explicit:
  no new promoted first-party workbench helper is warranted yet.

### Shell source-surface gates

- `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --test editor_notes_editor_rail_surface --no-fail-fast`

This package currently proves:

- `workspace_shell_demo` still mounts editor rails through `WorkspaceFrame` shell slots,
- `editor_notes_demo` still acts as an app-local shell-mounted rail proof,
- and shell composition remains reviewable at the source surface level.

### Launched shell smoke gate

- `cargo run -p fretboard-dev -- diag suite diag-hardening-smoke-workspace --launch -- cargo run -p fret-demo --bin workspace_shell_demo --release`

This package currently proves the promoted P1 launched shell floor:

- tab close / reorder / split preview,
- dirty-close prompt and discard close,
- Escape-based content-focus restore,
- and file-tree keep-alive / liveness.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-workbench-shell-closure-v1/WORKSTREAM.json > /dev/null`

## Missing gates before closure

The current gate package is intentionally inherited from the umbrella and from existing shell
surfaces.

Before claiming this lane is closed, the next useful addition should be:

- one gate tied to the first actual implementation-heavy shell slice landed from this folder,
- not another generic tabstrip or runner parity gate.

The M0 audit also freezes one immediate interpretation rule for future work:

- if a gap is really about shell assembly/default-path posture, keep it here,
- if it reduces to tabstrip behavior, continue the existing tabstrip lanes,
- if it reduces to multi-window hand-feel, continue docking parity.
