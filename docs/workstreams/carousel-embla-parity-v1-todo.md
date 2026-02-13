# Carousel Embla parity (v1) — TODO

This file is the *living checklist* for the workstream. It should stay short and actionable.
Use `carousel-embla-parity-v1-milestones.md` for timeline/phase planning.

## TODO (ordered)

### Mechanism contracts (crates/fret-ui)

- [ ] Ensure capture-phase move opt-in is stable and documented in the runtime contract matrix.
- [ ] Ensure capture switching always dispatches `PointerCancel` to the previous capture target (pressed state must clear).
- [ ] Verify no double-dispatch hazards (capture + bubble move) for opted-in pointer regions.

### Carousel behavior (ecosystem/fret-ui-shadcn)

- [ ] Verify drag arming/threshold matches Embla’s `dragThreshold` default (10px) across mouse + touch.
- [ ] Ensure drag can start on an interactive descendant and suppress descendant activation when drag wins.
- [ ] Decide whether focusable descendants should be excluded from arming (Embla ignores INPUT/SELECT/TEXTAREA by default).
- [ ] Decide whether cross-axis heuristics are needed for touch (Embla uses scroll-vs-cross diff to decide preventing scroll).

### Docs parity / UI gallery (apps/fret-ui-gallery)

- [ ] Add a stable “inner interactive element” inside a demo slide to make drag-vs-click semantics visible.
- [ ] Keep the page layout aligned with shadcn docs sections (Demo/Sizes/Spacing/Orientation/Options/API/Events/Plugins).

### Regression gates

- [ ] Add a Rust test: drag from descendant pressable must not activate the pressable.
- [ ] Add/extend a diag script: drag starting on the inner button must not set the “clicked” marker; a click must set it.
- [ ] Add evidence anchors to the parity audit note (upstream refs + in-tree anchors + gate paths).

### Follow-ups (v2+)

- [ ] Decide where the Embla-like headless engine belongs (`fret-ui-kit` vs `fret-ui-headless`).
- [ ] Design a minimal “controlled index + callbacks” surface without locking into Embla’s full event API.

