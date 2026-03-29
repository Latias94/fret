# imui sortable recipe v1 - design

Status: proposed active workstream

Last updated: 2026-03-29

Related:

- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/DRAG_DROP_BOUNDARY_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/headless-dnd-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/headless-dnd-fearless-refactor-v1/TODO.md`

## Purpose

This workstream is the immediate follow-on to
`docs/workstreams/imui-editor-grade-surface-closure-v1/`.

The helper-closure lane already shipped the missing generic seam:

- typed `drag_source(...)` / `drop_target::<T>(...)`,
- preview and delivery pointer positions on compatible targets,
- and first-party proof that app-owned reorder math can be built without widening the runtime
  contract.

What remains is a different question:

> how should Fret ship a reusable immediate sortable/reorder recipe without pushing sortable policy
> back into `fret-ui-kit::imui`?

This lane exists to answer that question cleanly.

## Current assessment

The repo now has enough evidence to say two things confidently.

First:

- the current `imui` drag/drop seam is good enough as the mechanism/helper boundary,
- and it already proves cross-surface typed payload transfer plus app-owned reorder math.

Second:

- repeated editor-grade reorder authoring is still too product-local,
- because every vertical list / outliner / inspector-row reorder flow still has to recompute the
  same insertion-side logic and hover packaging at the call site.

So the remaining gap is not another `imui` primitive.
The remaining gap is a reusable **recipe**.

## Why this follow-on should exist

Without a focused sortable lane, the repo is likely to drift into one of two bad outcomes:

- sortable policy slowly leaks back into `fret-ui-kit::imui`, or
- every product/demo reinvents reorder packaging slightly differently and the authoring story stays
  noisy.

This workstream should close that gap by moving up one layer:

- keep `imui` as the typed payload seam,
- put reusable reorder packaging in `ecosystem/fret-ui-kit::recipes`,
- and extract only pure/data-only helpers into `ecosystem/fret-dnd` when more than one recipe or
  product actually shares them.

## Goals

### G1 - Keep the owner split explicit

This workstream must keep the current boundary intact:

- `fret-ui-kit::imui` owns the typed drag/drop authoring seam,
- `fret-ui-kit::recipes` owns reusable sortable/reorder packaging,
- `fret-dnd` only owns shared pure/data-only helpers if they are justified,
- and app/product crates continue to own their domain state mutation.

### G2 - Start with the smallest reusable sortable target

The first target is not "all reorder problems."

The first target should be one clear reusable flow:

- vertical row reorder for list/outliner-style immediate authoring,
- with before/after insertion signals,
- and no requirement for multi-container choreography on day one.

### G3 - Preserve the fearless refactor posture

This lane does not keep stopgaps alive for compatibility.

If the first recipe shape is wrong:

- delete it,
- collapse it,
- or move it to the correct owner,

instead of layering aliases on top.

### G4 - Prove the recipe on a real editor-grade surface

The recipe must simplify the existing first-party reorder proof in
`apps/fret-examples/src/imui_editor_proof_demo.rs`.

If it does not make that proof surface materially clearer, then the recipe is not yet the right
shape.

## Non-goals

- Adding sortable/reorder policy directly to `fret-ui-kit::imui`.
- Widening the runtime drag contract.
- Recreating Dear ImGui's block-style drag/drop grammar.
- Shipping source ghost / preview chrome as a blocking part of the first slice.
- Taking on multi-container transfer, auto-scroll, or docking/workspace choreography in the first
  recipe pass.
- Preserving compatibility aliases for earlier local helper experiments.

## Non-negotiable boundaries

| Layer | Owns | Must not own |
| --- | --- | --- |
| `ecosystem/fret-ui-kit::imui` | typed immediate drag/drop seam, generic drag geometry signals | sortable insertion policy, reorder recipe defaults, product-specific row choreography |
| `ecosystem/fret-ui-kit::recipes` | reusable sortable/reorder packaging for immediate list/outliner authoring | runtime drag-session ownership, docking/workspace shell policy |
| `ecosystem/fret-dnd` | shared pure/data-only helpers if multiple consumers justify extraction | `UiWriter`-facing immediate helper APIs, product-local visuals |
| `apps/fret-examples` and product crates | item rendering, domain state mutation, product-specific semantics | pretending product-local policy is a generic recipe contract |

## Decision snapshot

### 1) The existing `imui` seam is the foundation, not the target of the next growth pass

The current typed drag/drop seam is already good enough for this follow-on.

That means the next step is:

- do not widen `fret-ui-kit::imui`,
- build on top of the shipped response-driven API,
- and keep the recipe layer honest about which data comes from `DropTargetResponse`.

### 2) The first recipe should be vertical-list reorder, not a generalized drag framework

The first stable recipe should target the common editor case:

- a vertical list/outliner with stable ids,
- app-owned item order,
- one active item at a time,
- and explicit before/after insertion math.

Anything beyond that needs fresh evidence before it becomes part of the first contract.

### 3) Extraction into `fret-dnd` must be evidence-driven

The recipe may discover a tiny pure helper that deserves shared ownership, for example:

- insertion-side classification from pointer position plus row rect,
- or a tiny reorder preview data model.

But that extraction should happen only if:

- at least two consumers need the same helper,
- the helper stays pure/data-only,
- and the abstraction is still valuable outside the immediate recipe itself.

### 4) Proof and gates stay anchored on the current editor-grade evidence surfaces

The first slice must keep the evidence tight:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- focused recipe/unit tests in the owning crate

The asset-chip to material-slot proof should remain on the raw drag/drop seam as a boundary check.

## Target architecture

### `ecosystem/fret-ui-kit::recipes`

Expected first owner for the public reusable surface.

The exact public names are intentionally not frozen yet, but the first slice should likely expose:

- a row-level integration helper for immediate list/outliner items,
- packaged hover/preview state for before/after insertion,
- and a clear place where apps perform the actual reorder mutation.

The recipe should feel like a thin reusable composition layer over:

- `drag_source(...)`,
- `drop_target::<T>(...)`,
- `preview_position`,
- `delivered_position`.

### `ecosystem/fret-dnd`

Optional owner for any extracted pure/data-only helper.

This crate should only gain follow-on helpers if the recipe implementation proves that the same
logic is both:

- shared across more than one consumer,
- and meaningfully independent from the immediate authoring surface.

### Proof surfaces

The first slice should prove itself on:

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- the existing reorder interaction gate in `ecosystem/fret-imui/src/tests/interaction.rs`

If the recipe cannot simplify those surfaces, it is not ready.

## Success criteria

This workstream is successful when:

- the existing reorderable outliner proof migrates from app-local insertion packaging to a reusable
  recipe,
- the resulting surface is clearly owned by `ecosystem/fret-ui-kit::recipes`,
- `fret-ui-kit::imui` remains unchanged except for bug fixes or minimal support signals,
- and the remaining deferred items are short, explicit, and correctly owned.
