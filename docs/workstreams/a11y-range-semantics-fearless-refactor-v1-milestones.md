# A11y range/numeric semantics (fearless refactor v1) — Milestones

Last updated: 2026-02-23

## Current progress (2026-02-23)

- M1: Complete (core contract + AccessKit mapping + unit tests).
- M1.5: Complete (scroll semantics + `scroll_by`, level, read-only, placeholder, url).
- M1.75: Complete (Scrollbar semantics: `SemanticsRole::ScrollBar` + scroll metadata + AccessKit role mapping + shadcn `ScrollArea` snapshot gate).
- M2: Complete (shadcn slider + progress populate numeric semantics; snapshots gated).
- M2.25: Complete (slider Increment/Decrement stepper actions exposed; runner + default UI driver hooks wired).
- M2.5: Complete (best-effort slider `SetValue(NumericValue)` handling via key sequences; runtime-gated exposure + tests).
- M3: Complete (SetSliderValue prefers structured numeric semantics; string parsing as fallback).
- M4: Complete (ADR 0288 landed; ADR 0181 notes updated; alignment matrix updated with evidence anchors).
- M4.5: Complete (tri-state checked semantics: `checked_state` contract + AccessKit mapping + shadcn checkbox indeterminate gates).

## M0 — Agreement on contract (design)

Exit criteria:

- Contract shape selected (`SemanticsNode.extra.numeric` with `value/min/max/step/jump`) and invariants documented.
- Contract invariants enforced by `SemanticsNode::validate()` (finite, bounds order, in-range where applicable).
- Layer ownership confirmed (`fret-core` contract; adapters; ecosystem policy).
- At least one smallest repro target chosen (e.g. `ui-gallery` slider, or shadcn slider tests).

## M1 — Core contract + AccessKit mapping landed

Exit criteria:

- `fret-core` `SemanticsNode.extra.numeric` carries numeric/range fields.
- AccessKit adapter emits numeric properties (plus step/jump if present).
- Unit tests cover numeric + extra mapping behavior.

## M1.5 — Optional: adjacent semantics surfaces batched

Exit criteria (pick any subset, but keep each one gated):

- Scroll semantics: scroll positions/ranges emitted for scrollable elements (at least `Scroll`).
- Hierarchy level: `extra.level` emitted and mapped (TreeItem, and later Heading).
- Text flags/properties: `read_only` and `placeholder` emitted and mapped for text inputs.
- Link URL and/or image role supported where present in ecosystem components.

## M2 — shadcn slider + progress populate numeric semantics

Exit criteria:

- Slider semantics nodes (thumb) include numeric range data.
- Progress semantics nodes include numeric data for determinate states.
- Existing shadcn semantics snapshot tests updated and passing.

## M3 — Diagnostics hardened (automation)

Exit criteria:

- `set_slider_value` prefers structured numeric semantics; string parsing remains as fallback.
- At least one `tools/diag-scripts/*set-slider-value*.json` script passes end-to-end.

## M4 — Docs + alignment cleanup

Exit criteria:

- ADR 0181 implementation alignment updated to remove “range control semantics pending” (if fully delivered).
- Evidence anchors point to code + tests + scripts.
