---
title: MVP Snapshot — UI Foundation Closure Sprint (2025-12)
---

# MVP Snapshot — UI Foundation Closure Sprint (2025-12)

This snapshot proposes the next time-boxed “foundation-first” phase: close the UI substrate
contracts that are expensive to change later, before scaling component surface area.

Source of truth remains:

- Roadmap: `docs/roadmap.md`
- Golden index: `docs/golden-architecture.md`
- UI closure map: `docs/ui-closure-map.md`
- Runtime contract gates: `docs/adr/0066-fret-ui-runtime-contract-surface.md`

---

## Goal

Make `crates/fret-ui` (mechanism-only) and its immediate neighbors “closed enough to scale” so we
can add components without late-stage rewrites:

- docking + multi-window + viewports
- multi-root overlays (non-modal + modal)
- render transforms + clip stacks + hit testing
- A11y/semantics (AT-ready baseline)

---

## P0 Closure Items (Definition of Done)

Each item is “done” when we have: (a) accepted contract or explicit decision gate, (b) implementation,
(c) regression coverage (tests and/or stable demo + checklist).

### 1) RenderTransform-aware hit testing + anchored overlays (Done)

- Contract: `docs/adr/0082-render-transform-hit-testing.md`
- Validation: `cargo nextest run -p fret-ui` and shadcn overlay anchoring tests (`cargo nextest run -p fret-ui-shadcn`)

### 2) Docking interaction arbitration (Done, keep hardening)

- Contract: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Checklist: `docs/docking-arbitration-checklist.md`
- Demo harness: `cargo run -p fret-demo --bin docking_arbitration_demo`
- Regression tests: `cargo nextest run -p fret-docking`

Hardening targets (follow-ups):

- cross-window edge cases (tear-off + drag cancel + modal barrier)
- platform-specific pointer capture quirks

### 3) Transform + clip + hit-testing parity (Not closed)

Why this is P0: incorrect parity causes “click-through bugs” and forces late rewrites across
components, overlays, and viewport tooling.

- Contract anchors:
  - `docs/adr/0078-scene-transform-and-clip-composition.md`
  - `docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`
  - `docs/adr/0082-render-transform-hit-testing.md`
- Required validation:
  - baseline `fret-ui` parity tests (rounded overflow clip under `render_transform`, overlay transforms, nested transforms),
  - incremental hardening for deeper stacks across multi-root overlays,
  - renderer conformance linkage (deep transform/clip stacks).

### 4) A11y / semantics closure for composite widgets (Not closed)

Why this is P0: semantics reachability must match input reachability under modal barriers and
multi-root overlays, otherwise accessibility becomes impossible to retrofit.

- Contract anchors:
  - `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
  - `docs/adr/0073-active-descendant-and-composite-widget-semantics.md`
  - Acceptance checklist: `docs/a11y-acceptance-checklist.md`
- Required validation:
  - at least one end-to-end harness covering active-descendant + modal reachability rules
  - stable minimum semantics for text fields (value/selection/composition)

### 5) Multi-window degradation policy (Not closed)

Why this is P0: docking graph supports multiple logical windows; we must define how this degrades
on platforms without multi-window (single-window “floating overlays” vs “tabs”, etc.).

- Contract anchors:
  - `docs/adr/0017-multi-window-display-and-dpi.md`
  - `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Output:
  - a single explicit policy document and a small demo/driver-level implementation anchor.

---

## Not In Scope (Deliberate)

- New public runtime surface area in `crates/fret-ui` without an accepted ADR and tests.
- Large new UI kits; use `fret-components-*` to validate policies.
- Arrow rendering / advanced overlay visuals until transform/clip parity is closed.
