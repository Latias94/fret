# ADR 0077: Resizable Panel Groups and Docking Split Sizing

Status: Accepted

## Context

Fret needs an editor-grade “resize handle” contract that can be reused across:

- docking split nodes (the primary consumer),
- shadcn-style `ResizablePanelGroup` (composition surface),
- future: table column resize, sidebar/inspector splits, etc.

If each of these re-implements pointer capture, cursor affordances, and min-size clamping, behavior will drift and regress.

Reference implementations:

- Dear ImGui docking stores a split tree (`ImGuiDockNode`), and processes splitter interaction as part of a recursive tree update:
  - split nodes store axis + `SizeRef` and update children sizes during drag,
  - nested splits on the same axis require "lock size once" / touching-node handling to avoid oscillation.
  - Code: `repo-ref/imgui/imgui_internal.h` (`ImGuiDockNode`), `repo-ref/imgui/imgui.cpp` (`DockNodeTreeUpdateSplitter`).

Related ADRs:

- Dock graph/persistence/ops: `docs/adr/0013-docking-ops-and-persistence.md`
- Docking interaction arbitration: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Runtime contract surface boundary: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Declarative-only + component-owned policy: `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- Model observation / invalidation: `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`

## Decision

### 1) Runtime owns resize interaction mechanics

`fret-ui` provides a runtime primitive that owns:

- pointer hit-testing for resize handles,
- pointer capture during drag,
- cursor icon requests while hovering/dragging,
- clamping to per-panel minimum sizes,
- deterministic “push” behavior when expanding a panel (consume size from the following panels).

The primitive supports a small (or zero) layout `gap` while keeping a larger pointer `hit_thickness` for usability.

This primitive is *mechanism-only* and intentionally does not encode docking semantics (tabs, drag-and-drop, persistence).

### 2) Docking renders split nodes via the resizable primitive

The docking UI host renders any “split” node from the app-owned dock graph as a `ResizablePanelGroup` subtree:

- docking stays the source of truth for the dock graph structure (ADR 0013),
- resizing updates the split node’s sizing model (fractions/weights) via model mutation,
- the dock host observes that model for `Invalidation::Layout` (ADR 0051) to keep hit-testing and bounds correct.

### 3) Persistence remains fraction-based (for now)

The canonical persisted dock layout continues to store split `fractions` (ADR 0013).

Rationale:

- fractions are stable across window sizes and DPI changes,
- they avoid needing to encode viewport-dependent pixel sizes in the persistence format.

Non-goal (deferred):

- storing pixel `SizeRef`/“preferred px” hints in the persisted schema.

If later UX parity requires pixel-sticky sizing (ImGui `SizeRef` feel), add a new schema version that optionally stores
`preferred_px` per split edge, with a clear migration story.

### 4) Component layer provides shadcn taxonomy, not mechanics

`fret-ui-shadcn` exposes shadcn-aligned names:

- `ResizablePanelGroup`, `ResizablePanel`, `ResizableHandle`

Those are declarative composition surfaces that forward into the runtime primitive; they do not re-implement drag logic.

## Consequences

- Docking and shadcn “resizable panels” share the exact same resize interaction behavior.
- The resize contract can be reused by future resize affordances (table columns, inspector splits).
- The runtime primitive is compatible with multi-root overlays and capture rules (ADR 0011, ADR 0072).

Tradeoffs / limitations (current):

- Per-handle element nodes are not modeled as first-class children; handles are internal affordances (painted/hit-tested by the runtime primitive).
- Nested split ergonomics may require additional stabilization (ImGui-style "touching nodes lock" when same-axis nested).

## Implementation Notes (Current Prototype)

- Runtime primitive: `crates/fret-ui/src/resizable_panel_group/mod.rs`
- Declarative surface: `ResizablePanelGroupProps` + `ElementContext::resizable_panel_group(...)`
- shadcn facade: `ecosystem/fret-ui-shadcn/src/resizable.rs`
- Docking integration: split layout/hit-testing/painting delegates to the same panel-group mechanics via `fret-ui/unstable-retained-bridge`.
- Docking drag commit: drag updates mutate the app-owned graph for immediate feedback; drag end emits a single atomic `DockOp::SetSplitFractionsMany` transaction.

## Follow-ups

1. Docking host uses `ResizablePanelGroup` for split rendering (replace ad-hoc split widgets). (done)
2. Same-axis nested split stabilization:
   - Previously implemented as a docking-layer lock pass for binary nested splits.
   - With the docking N-ary canonicalization workstream, same-axis nested splits are flattened into a single N-ary split on import and after ops, so the legacy stabilization is no longer required and was removed.
3. Decide whether to promote pixel `preferred_px` hints into dock persistence schema (new layout version) based on feedback.
