# Editor TabStrip Unification Fearless Refactor v1 (Owner Layer Map)

This document assigns an explicit owner layer for each editor-tabstrip behavior so refactors can
converge `WorkspaceTabStrip` and `Docking TabBar` without pushing policy into the wrong crate.

Primary goal:

- shared, deterministic math and interaction vocabulary live in `ecosystem/fret-ui-headless`
- editor/workspace semantics live in `ecosystem/fret-workspace`
- docking graph integration lives in `ecosystem/fret-docking`
- low-level event/layout/scroll primitives stay in `crates/fret-ui`

## Boundary rules

### `crates/fret-ui`

Owns runtime primitives only:

- pointer / wheel / key event routing
- scroll handles and scroll containers
- semantics nodes / focusable primitives / roving focus host support
- paint, layout, hit-testing, and invalidation contracts

Does **not** own:

- tab overflow policy
- pinned / preview semantics
- tab close rules
- drag reorder policies
- editor command wiring

### `ecosystem/fret-ui-headless`

Owns adapter-agnostic mechanism helpers:

- explicit tab-strip surface vocabulary
- overflow membership and overflow-menu index selection
- canonical insert-index / midpoint drop-target math
- click arbitration vocabulary (`activate` vs `close` vs `toggle overflow`)
- scroll-to-visible math
- pointer-slop / close-hit-test helpers when the rule is generic

This layer should be pure or near-pure and unit-testable.

### `ecosystem/fret-workspace`

Owns editor/workspace policy:

- pinned tabs and pinned-boundary behavior
- preview-tab lifecycle and commit/replace rules
- MRU and focus-restore rules
- `focus_tab_strip` / `focus_content` / toggle focus commands
- keyboard expectations specific to editor shells
- context menu commands and workspace-specific command dispatch

### `ecosystem/fret-docking`

Owns docking adapter policy and graph integration:

- dock graph ops and drag-to-split routing
- docking-local overflow menu presentation and geometry glue
- cross-window / floating-dock drag arbitration
- mapping shared tabstrip intents to dock operations

### `ecosystem/fret-ui-kit`

Optional toolbox layer only:

- re-exports shared headless helpers when that reduces import churn
- shared policy helpers only if more than one adapter truly needs them

This layer should not become a second home for workspace-only semantics.

## Behavior map

| Behavior / invariant | Owner layer | Why this owner is correct | Current evidence |
|---|---|---|---|
| Surface vocabulary: `TabsViewport`, `HeaderSpace`, `OverflowControl`, `ScrollControls`, `PinnedBoundary`, `Outside` | `ecosystem/fret-ui-headless` | Adapters need a shared, deterministic language for hit-testing and drag/drop review | `ecosystem/fret-ui-headless/src/tab_strip_surface.rs`, `ecosystem/fret-docking/src/dock/tab_bar_kernel.rs`, `ecosystem/fret-workspace/src/tab_strip/kernel.rs` |
| Overflow membership from viewport + margin | `ecosystem/fret-ui-headless` | Pure geometry; should not diverge between workspace and docking | `ecosystem/fret-ui-headless/src/tab_strip_overflow.rs` |
| Overflow-menu row intent (`close` does not activate) | `ecosystem/fret-ui-headless` | Small shared interaction rule with adapter-owned dispatch | `ecosystem/fret-ui-headless/src/tab_strip_controller.rs` |
| Scroll active tab into view | `ecosystem/fret-ui-headless` for math, adapters for trigger policy | Math is generic; “when to reveal” depends on product surface | `ecosystem/fret-ui-headless/src/tab_strip_scroll.rs`, `ecosystem/fret-workspace/src/tab_strip/utils.rs` |
| Canonical insert index and midpoint drop target | `ecosystem/fret-ui-headless` + thin adapter kernels | Algorithm should be shared; adapters still map indices to domain ops | `ecosystem/fret-ui-headless/src/tab_strip_drop_target.rs`, `ecosystem/fret-docking/src/dock/tab_bar_kernel.rs`, `ecosystem/fret-workspace/src/tab_strip/kernel.rs` |
| Close-hit testing and pointer-slop arbitration | `ecosystem/fret-ui-headless` | Same editor-tabstrip hazard exists in workspace and docking | `ecosystem/fret-ui-headless/src/tab_strip_hit_test.rs`, `ecosystem/fret-workspace/src/tab_strip/interaction.rs` |
| Overflow-menu contents policy (`overflowed-only` vs `overflowed + active`) | Adapter policy (`workspace` / `docking`) | Product choice differs today and should remain configurable | `ecosystem/fret-workspace/src/tab_strip/mod.rs`, `ecosystem/fret-docking/src/dock/tab_overflow.rs` |
| Overflow-menu rendering and chrome | Adapter policy | Workspace and docking have different shells/chrome and may keep different menu surfaces | `ecosystem/fret-workspace/src/tab_strip/overflow.rs`, `ecosystem/fret-docking/src/dock/space.rs` |
| Pinned tabs and pinned-boundary semantics | `ecosystem/fret-workspace` | Editor semantics, not generic tab mechanism | `ecosystem/fret-workspace/src/tabs.rs`, `ecosystem/fret-workspace/src/tab_strip/mod.rs` |
| Preview-tab semantics | `ecosystem/fret-workspace` | Strongly editor-specific lifecycle; docking does not currently model it | `ecosystem/fret-workspace/src/tabs.rs` |
| MRU fallback and focus restore after close | `ecosystem/fret-workspace` | Editor focus policy depends on pane/content ownership | `ecosystem/fret-workspace/src/tab_strip/mod.rs`, `ecosystem/fret-workspace/tests/tab_strip_focus_restore_after_close_command.rs` |
| `focus_tab_strip`, `focus_content`, `Ctrl+F6` toggle | `ecosystem/fret-workspace` | Shell command policy; not a generic docking or mechanism concern | `ecosystem/fret-workspace/tests/pane_focus_tab_strip_command_focuses_active_tab.rs`, `ecosystem/fret-workspace/tests/workspace_command_scope_toggle_tab_strip_focus_toggles_between_content_and_tab_strip.rs` |
| Dock graph reorder / split-on-drop / cross-window routing | `ecosystem/fret-docking` | Domain-specific integration with docking graph and floating windows | `ecosystem/fret-docking/src/dock/space.rs` |
| Drag-to-split preview in workspace shell | `ecosystem/fret-workspace` + `ecosystem/fret-docking` adapter seam | Workspace owns shell UX; docking owns graph mutation and preview target routing | `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-drag-to-split-right.json`, `tools/diag-scripts/docking/arbitration/` |
| Stable `test_id` anchors and diagnostics publishing | Adapter policy | IDs and diagnostics payloads are product-surface specific | `ecosystem/fret-workspace/src/tab_strip/mod.rs`, `ecosystem/fret-bootstrap/src/ui_diagnostics/workspace_diagnostics.rs` |

## Duplication we should remove

These are the highest-value convergence targets because they already behave like shared kernels:

1. Overflow membership / visible-range selection rules.
2. Overflow-menu row click arbitration.
3. Header-space / end-drop surface naming and insert-index closure.
4. Close-hit / pointer-slop arbitration for “close vs activate vs drag”.

These should converge before larger visual or shell-level refactors.

## Duplication we should keep

These behaviors are intentionally adapter-owned and should not be forced into a shared component:

- workspace pinned / preview rules
- workspace focus-transfer commands and MRU restore
- docking graph mutations and floating-window drag routing
- menu contents and command sets specific to each product surface

## Recommended landing order

### Phase 1 — Overflow convergence

Unify the data pipeline first:

- overflow membership
- overflow-menu row intent mapping
- select-from-overflow reveal rules

Why first:

- already partially shared
- highly user-visible
- easy to gate in both unit tests and diag scripts

### Phase 2 — Surface / drop convergence

Unify:

- `HeaderSpace` and explicit end-drop surface semantics
- canonical insert-index closure under overflow
- overflow-control exclusion from drop surfaces

Why second:

- it reduces the most dangerous drag/drop divergence between workspace and docking

### Phase 3 — Close/drag arbitration convergence

Unify generic close-hit and pointer-slop rules.

Keep adapter-owned command dispatch and drag payload logic local.

### Phase 4 — Editor-only policy stabilization

Stabilize workspace-only editor semantics:

- pinned row option
- preview lifecycle
- focus restore / focus toggle contracts

Do not block docking convergence on preview/pinned semantics.

## Review checklist for future changes

Before landing a tabstrip change, answer these questions:

1. Is this behavior generic enough to be shared by workspace and docking?
2. If yes, can it be expressed as pure data/geometry/intent math in `fret-ui-headless`?
3. If no, is it clearly editor policy (`fret-workspace`) or docking policy (`fret-docking`)?
4. What unit test or diag script proves the behavior after the refactor?

If the answer to (1) is “no”, do not move it into a shared mechanism crate just to reduce file count.

## Immediate next steps

- Treat this file as the owner map referenced by M0 in `MILESTONES.md`.
- Use it to drive the next concrete code pass: overflow convergence in `fret-docking`.
- Keep `WorkspaceTabStrip` as the editor-behavior baseline and pull only shared kernels downward.
