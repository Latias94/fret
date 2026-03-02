# Workspace TabStrip (Fearless Refactor v1) — Design Notes

## Goal

Make Workspace TabStrip behavior deterministic, editor-grade, and regression-gated without
expanding `crates/fret-ui` into a policy-heavy component layer.

## Non-negotiable boundaries

- `crates/fret-ui` remains a **mechanism/contract** layer.
- Workspace TabStrip is a **policy-driven component** and lives in the workspace ecosystem
  (`ecosystem/fret-workspace` + `ecosystem/fret` glue), not in `fret-ui`.
- Shared **mechanism helpers** (pure geometry / surface classification) belong in
  `ecosystem/fret-ui-headless`.
- Shared **interaction arbitration policy** belongs in `ecosystem/fret-ui-kit`.

## Contract surface (v1)

### Surface classification

Workspace must classify pointer positions against a stable vocabulary:

- `Outside`
- `OverflowControl`
- `ScrollControls`
- `PinnedBoundary`
- `TabsViewport`
- `HeaderSpace` (explicit end-drop surface)

Evidence:

- `ecosystem/fret-ui-headless/src/tab_strip_surface.rs`

### Hit targets

Workspace must produce a coarse hit target that is stable across rendering refactors:

- tab row: `{ index, part: Content|Close }`
- overflow button
- overflow menu row: `{ index, part: Content|Close }`
- header space / none

Evidence:

- `ecosystem/fret-ui-kit/src/headless/tab_strip_controller.rs`

### Click intent arbitration

Workspace must follow editor-grade click intent rules:

- Close never activates.
- Overflow menu content activates and ensures visible.
- Tab content activates (ensure-visible is optional; workspace may keep it false when already in view).

Evidence:

- `ecosystem/fret-ui-kit/src/headless/tab_strip_controller.rs` (`intent_for_click`)

### Insert index semantics

`insert_index` is always expressed in **canonical list order**, not "visible index" under overflow.

Implications:

- Overflow dropdown is a view over the canonical list.
- Drag previews must map from "where the pointer is" to a canonical insert index.

## Kernel structure recommendation

Keep a workspace-specific kernel (v1) rather than forcing immediate convergence with docking:

- Docking and workspace tab strips have similar vocabulary but differ in ops/payload and may differ
  in pinned/preview semantics.
- Share math helpers (surface classification + overflow membership), keep kernels separate until both
  stabilize.

## Evidence anchors (current)

- Workspace shell usage:
  - `ecosystem/fret/src/workspace_shell.rs`
- Workspace shell demo (for future diag scripts):
  - `apps/fret-examples/src/workspace_shell_demo.rs`
