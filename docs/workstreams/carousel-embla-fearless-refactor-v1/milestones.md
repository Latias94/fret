# Milestones — Carousel Embla fearless refactor (v1)

Milestones are structured to keep changes reviewable and reversible. Each milestone must ship a
“3-pack”: Repro (smallest surface), Gate (tests/scripts), Evidence (anchors + upstream refs).

## M0 — Gate set (fearless foundation)

**Goal:** Create enough regression protection that we can refactor without fear.

**Deliverables**

- Headless unit tests for:
  - variable slide size snap derivation,
  - `align`,
  - `containScroll=trimSnaps`.
- Geometry parity coverage expanded for:
  - vertical constrained viewport,
  - at least one variable-size slide case.
- Diag scripts:
  - drag from interactive descendant cancels activation,
  - (optional) touch cross-axis scroll lock.

**Exit criteria**

- All gates green locally (`cargo nextest run -p fret-ui-shadcn web_vs_fret_layout_carousel` plus the new headless tests).
- At least one diag script produces a packed bundle and is deterministic with fixed frame delta.

## M1 — Engine scaffold (no default switch)

**Goal:** Introduce an engine-backed code path without changing default behavior.

**Deliverables**

- New headless types (names illustrative):
  - `CarouselSnapModel`
  - `CarouselEngineState`
  - `CarouselEngine` (update API)
- `ecosystem/fret-ui-shadcn::Carousel` has:
  - existing v0 path,
  - new v1 engine-backed path behind an internal toggle.
- UI gallery Carousel page uses v1 path (explicitly), leaving default untouched.

**Exit criteria**

- UI gallery renders identically for existing demo sections.
- Existing web-vs-fret tests still pass.

## M2 — Geometry-derived snaps (P0 core)

**Goal:** Replace uniform extent snapping with geometry-derived snap list.

**Deliverables**

- Snap list derived from measured slide sizes + gaps.
- Selection/index computed from snaps.
- Buttons/keys operate on snaps (not “index * extent”).

**Exit criteria**

- Variable slide size gates pass.
- Orientation vertical gates still pass.

## M3 — Align + containScroll(trimSnaps) parity (P0 completion)

**Goal:** Match Embla’s default edge behavior and alignment semantics.

**Deliverables**

- `align=start|center|end` integrated into snap computation.
- `containScroll=trimSnaps` implemented and gated.

**Exit criteria**

- Headless unit tests cover align + containScroll edge cases.
- At least one new geometry parity case is added for clamped edges.

## M4 — slidesToScroll (P1)

**Goal:** Support grouping behavior for snapping.

**Deliverables**

- `slidesToScroll` implemented in snap grouping.
- Gates updated (unit + geometry).

**Exit criteria**

- snaps and `slidesBySnap` match expected for representative cases.

## M5 — Flip default + delete v0

**Goal:** Make v1 the default and remove the old path.

**Deliverables**

- Default `Carousel` uses engine-backed implementation.
- v0 code path removed.

**Exit criteria**

- All gates remain green; no new public API knobs were introduced in mechanism layers.

