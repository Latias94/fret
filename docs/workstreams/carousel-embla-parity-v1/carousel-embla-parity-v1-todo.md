# Carousel Embla parity (v1) — TODO


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- `embla-carousel`: https://github.com/search?q=embla-carousel&type=repositories
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This file is the *living checklist* for the workstream. It should stay short and actionable.
Use `carousel-embla-parity-v1-milestones.md` for timeline/phase planning.

## TODO (ordered)

### Mechanism contracts (crates/fret-ui)

- [x] Ensure capture-phase move opt-in is stable and documented in the runtime contract matrix.
- [x] Ensure capture switching always dispatches `PointerCancel` to the previous capture target (pressed state must clear).
- [x] Verify no double-dispatch hazards (capture + bubble move) for opted-in pointer regions.

### Carousel behavior (ecosystem/fret-ui-shadcn)

- [x] Verify drag arming/threshold matches Embla’s `dragThreshold` default (10px) across mouse + touch.
- [x] Ensure drag can start on an interactive descendant and suppress descendant activation when drag wins.
- [x] Decide whether focusable descendants should be excluded from arming (Embla ignores INPUT/SELECT/TEXTAREA by default).
- [x] Decide whether cross-axis heuristics are needed for touch (Embla uses scroll-vs-cross diff to decide preventing scroll).

### Docs parity / UI gallery (apps/fret-ui-gallery)

- [x] Add a stable “inner interactive element” inside a demo slide to make drag-vs-click semantics visible.
- [x] Keep the page layout aligned with shadcn docs sections (Demo/Sizes/Spacing/Orientation/Options/API/Events/Plugins).

### Regression gates

- [x] Add a Rust test: drag from descendant pressable must not activate the pressable.
- [x] Add/extend a diag script: drag starting on the inner button must not set the “clicked” marker; a click must set it.
- [x] Add evidence anchors to the parity audit note (upstream refs + in-tree anchors + gate paths).

## Evidence anchors (v1)

### Upstream references

- Embla options defaults (`dragThreshold=10`): `repo-ref/embla-carousel/packages/embla-carousel/src/components/Options.ts`
- Embla drag handler click suppression: `repo-ref/embla-carousel/packages/embla-carousel/src/components/DragHandler.ts`
- shadcn/ui carousel composition: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/carousel.tsx`

### In-tree anchors + gates

- Mechanism contracts: `docs/runtime-contract-matrix.md` (capture-phase moves, cancel-on-capture-switch)
- Carousel implementation: `ecosystem/fret-ui-shadcn/src/carousel.rs` (uses `fret_ui_headless::carousel::DEFAULT_DRAG_THRESHOLD_PX`)
- UI gallery demo surface: `apps/fret-ui-gallery/src/ui/pages/carousel.rs` (`ui-gallery-carousel-demo-inner-button`)
- Rust test gate: `ecosystem/fret-ui-shadcn/tests/carousel_pointer_passthrough.rs`
- Diag script gate: `tools/diag-scripts/ui-gallery/carousel/ui-gallery-carousel-demo-swipe-and-buttons.json`

### Follow-ups (v2+)

- [ ] Decide where the Embla-like headless engine belongs (`fret-ui-kit` vs `fret-ui-headless`).
- [ ] Design a minimal “controlled index + callbacks” surface without locking into Embla’s full event API.
