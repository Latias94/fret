# M1 Proof Roster Freeze - 2026-04-21

Status: active decision note

Purpose: freeze the current collection-first and pane-first proof roster for
`imui-collection-pane-proof-v1` before the lane starts inventing narrower dedicated demos by
default.

## Decision

The current M1 proof roster is:

1. Keep `apps/fret-examples/src/imui_editor_proof_demo.rs` as the current collection-first proof
   surface.
2. Keep `apps/fret-examples/src/workspace_shell_demo.rs` as the current pane-first proof surface.
3. Keep `apps/fret-examples/src/editor_notes_demo.rs` as the supporting minimal pane rail proof.

## Why this is the right current roster

### 1) `imui_editor_proof_demo` already carries the best current collection-first evidence

The current editor proof already combines several collection-heavy editor outcomes in one place:

- asset chips with typed drag/drop payloads and preview ghosts,
- a drop slot that demonstrates app-owned drag/drop delivery,
- sortable outliner rows with explicit reorder math,
- and editor-grade inspector/property-grid content in the same proof surface.

That is enough breadth to keep using it as the current collection-first proof while the lane
decides whether a narrower dedicated asset-grid/file-browser proof is actually required.

Primary evidence:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`

### 2) `workspace_shell_demo` is still the broadest current pane-first proof

The current workspace shell proof is still the strongest shell-mounted pane host because it already
exercises:

- the frozen workspace starter-set shell,
- a real inspector rail,
- tabstrip/shell composition,
- and the diagnostics floor that the repo already promoted for shell-mounted editor behavior.

That makes it the correct pane-first proof surface for now.

Primary evidence:

- `apps/fret-examples/src/workspace_shell_demo.rs`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
- `tools/diag-scripts/suites/diag-hardening-smoke-workspace/suite.json`

### 3) `editor_notes_demo` should stay as the supporting minimal pane rail proof

`editor_notes_demo` remains useful because it strips the pane story down to:

- one selected asset,
- one inspector rail,
- and one shell-mounted reusable editor surface.

That makes it the right supporting proof for a smaller rail/in-panel read, but not the primary
pane-first proof.

Primary evidence:

- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`

## Explicit non-decisions

Do not introduce a dedicated asset-grid/file-browser proof demo yet.

Do not introduce a narrower child-region-only proof demo yet.

The lane should first use the current roster to answer whether:

- M2 really needs a new collection-first proof surface,
- M3 really needs a narrower pane composition proof,
- or the real next need is helper widening backed by these existing surfaces.

## Deferred out of this lane

Keep these deferred and explicit:

- key ownership,
- promoted shell helpers,
- runner/backend multi-window parity,
- and broader menu/tab policy.

## Execution consequence

From this note forward:

1. use `imui_editor_proof_demo` as the current collection-first proof surface,
2. use `workspace_shell_demo` as the current pane-first proof surface,
3. use `editor_notes_demo` as the supporting minimal pane rail proof,
4. keep the next lane-local source-policy gate tied to this exact roster,
5. and start a narrower proof demo only if M2 or M3 evidence shows the current roster is
   insufficient.
