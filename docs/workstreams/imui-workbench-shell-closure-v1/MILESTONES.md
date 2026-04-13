# ImUi Workbench Shell Closure v1 - Milestones

Status: active execution lane
Last updated: 2026-04-13

## M0 - Baseline and lane split

Exit criteria:

- the repo explicitly states why this is a new P1 shell follow-on instead of another umbrella note,
- the first-open proof order stays frozen,
- and the owner split is explicit before implementation work starts.

Primary evidence:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-13.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`

Current status:

- Closed on 2026-04-13 via `M0_BASELINE_AUDIT_2026-04-13.md`.
- This lane now exists as the implementation-heavy P1 follow-on under the active immediate-mode
  product-closure umbrella.
- The frozen proof order inherited from the umbrella is:
  `workspace_shell_demo` -> `editor_notes_demo` -> `workspace-tabstrip-editor-grade-v1` ->
  `imui_editor_proof_demo` -> docking multi-window parity.
- The promoted launched shell diagnostics floor inherited from the umbrella remains
  `diag-hardening-smoke-workspace`.
- The current smallest P1 gap is now frozen as shell-assembly posture above the existing starter
  set and proof demos, not as another generic `imui` or tabstrip parity question.

## M1 - Default workbench shell closure slice

Exit criteria:

- one shell-composition gap is isolated as the current blocking slice,
- the responsible owner is explicit,
- and that slice lands with one focused source gate plus one launched/diag proof.

Primary evidence:

- `TODO.md`
- `M1_DEFAULT_WORKBENCH_ASSEMBLY_DECISION_2026-04-13.md`
- `EVIDENCE_AND_GATES.md`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`

Current status:

- Closed on 2026-04-13 via `M1_DEFAULT_WORKBENCH_ASSEMBLY_DECISION_2026-04-13.md`.
- The current P1 shell decision is a no-new-helper-yet verdict:
  keep the default workbench answer as explicit example-local assembly over the frozen starter set.
- The current owner for that decision remains app/example composition above `fret-workspace`,
  rather than a newly promoted first-party shell recipe/helper.
- Future extraction now requires stronger repeated full-shell evidence plus one focused source gate
  and one launched gate.

## M2 - Closeout or narrower follow-on

Exit criteria:

- this lane either closes with an explicit owner split and durable shell proof/gate package,
- or it hands off the next clustered problem to a narrower follow-on without reopening the umbrella.

Primary evidence:

- `WORKSTREAM.json`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`

Current status:

- In progress.
