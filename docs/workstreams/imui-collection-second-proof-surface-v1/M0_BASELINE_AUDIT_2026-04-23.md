# ImUi Collection Second Proof Surface v1 - M0 Baseline Audit

Date: 2026-04-23
Status: baseline frozen

## What we re-read

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

## Assumptions-first restatement

1. `imui_editor_proof_demo` plus the closed command-package lane still count as only one current
   collection-first proof surface.
2. `editor_notes_demo.rs` is the smallest materially different shell-mounted second proof
   candidate because it already composes left/right rails through `WorkspaceFrame` slots.
3. `workspace_shell_demo.rs` remains supporting evidence for shell-mounted proof pressure, but it
   should not erase the need for a smaller second proof.
4. No dedicated asset-grid/file-browser demo should be introduced yet.
5. The frozen proof-budget rule still blocks shared helper growth until a second real proof
   surface lands.

## Owner split check

- `apps/fret-examples/src/editor_notes_demo.rs`
  - should own the primary second proof-surface candidate.
- `apps/fret-examples/src/workspace_shell_demo.rs`
  - should remain supporting shell-mounted proof evidence.
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
  - should keep the smaller shell-mounted candidate explicit.
- `apps/fret-examples/tests/workspace_shell_pane_proof_surface.rs`
  - should keep the broader shell-mounted proof explicit.
- `fret-imui`, `fret-ui-kit::imui`, and `crates/fret-ui`
  - should remain unchanged.

## Narrow target

The correct M1 slice is:

1. freeze the current second proof-surface roster in docs plus source-policy,
2. name `editor_notes_demo.rs` as the primary candidate and `workspace_shell_demo.rs` as supporting
   evidence,
3. keep “no dedicated asset-grid/file-browser demo yet” explicit,
4. and leave actual collection-surface implementation to the next bounded slice inside this lane.
