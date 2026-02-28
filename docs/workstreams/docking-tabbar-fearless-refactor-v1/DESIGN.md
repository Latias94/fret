# Docking TabBar Fearless Refactor v1 (Design)

## Context

Docking is a core “editor-grade UI” capability for Fret. The TabBar inside a dock stack is one of the
highest-frequency interaction surfaces (select, close, reorder, drag-to-split, cross-pane move).

We want to make this surface:

- Correct (drop resolution + insert index)
- Observable (stable diagnostics predicates + scripts)
- Modular (mechanism vs policy boundaries; reusable geometry helpers)
- Comparable (feature parity notes vs references)

This workstream is intentionally “fearless refactor”-friendly: land small, gated steps, and keep
interfaces stable even if the implementation is rewritten multiple times.

## Non-goals

- No “component library policy” in `crates/fret-ui`.
- No attempt to copy upstream visuals; only interaction/semantics and contract stability.
- No hard dependency on a specific authoring paradigm (retained vs declarative) for the TabBar.

## Layering (where code should live)

- `crates/fret-core`: docking model + stable ops (no UI policy, no platform).
- `ecosystem/fret-docking`: interaction arbitration + UI integration for docking.
- `ecosystem/fret-workspace`: editor/workspace-specific policies (pinned/preview/etc).
- `ecosystem/fret-dnd`: reusable DnD geometry helpers (pure math / hit-test vocabulary).
- `crates/fret-diag-protocol` + `ecosystem/fret-bootstrap`: typed diagnostics predicates and harness glue.

## Proposed internal split (docking tab bar)

Split the docking tab bar into three conceptual modules (even if they stay in one crate initially):

1. **Kernel (pure)**
   - Inputs: tab rects, header rects, pointer position, drag payload metadata.
   - Outputs: `ResolvedZone`, `insert_index`, optional `split_placement`, plus “why” debug info.
   - Must be deterministic and unit-testable.

2. **View (render)**
   - Renders tabs, overflow controls, and explicit drop surfaces.
   - Emits stable `test_id` anchors where feasible (declarative) or exports diagnostics events (immediate).

3. **Controller (policy)**
   - Maps `ResolvedZone`/`insert_index` into docking operations (move, merge, split).
   - Owns editor-flavored policy (e.g. pinned, preview tab semantics) in the correct higher layer.

## Drop surfaces

Adopt an explicit “end drop surface” concept (dockview / gpui-component style):

- Always available when the tab bar is droppable.
- Visually: can be transparent; contract-wise: has a stable hit-test region.
- Contract: when hit, resolves to `insert_index == tab_count`.

For self-drawn widgets that cannot easily carry fine-grained `test_id`, we keep diagnostics predicates
as the primary gate, but still aim to expose an explicit geometry surface (even if only internally).

## Overflow (pipeline, not a special-case)

Treat overflow as a pipeline stage that produces:

- A visible range (or “visible tabs” list)
- An overflow list (dropdown / menu)
- A mapping layer to keep `insert_index` stable in terms of the *full* tab list

Key invariant:

- `insert_index` is expressed in the canonical tab list order, not “visible index”.

## Reference slicing (what to learn from whom)

- Zed: editor semantics (pinned/preview/focus-neutral interactions, split behaviors).
- dockview: overflow pipeline + header-space drop surfaces + DnD ergonomics.
- gpui-component: minimal wiring shape for `TabBar` + “empty space to drop at end”.

