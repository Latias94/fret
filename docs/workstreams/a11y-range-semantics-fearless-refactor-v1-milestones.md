# A11y range/numeric semantics (fearless refactor v1) — Milestones

Last updated: 2026-02-22

## M0 — Agreement on contract (design)

Exit criteria:

- Numeric/range fields selected (`now/min/max/step/jump`) and invariants documented.
- Layer ownership confirmed (`fret-core` contract; adapters; ecosystem policy).
- At least one smallest repro target chosen (e.g. `ui-gallery` slider, or shadcn slider tests).

## M1 — Core contract + AccessKit mapping landed

Exit criteria:

- `fret-core` `SemanticsNode` carries numeric/range fields.
- AccessKit adapter emits `numeric_value` / `min_numeric_value` / `max_numeric_value` (plus step/jump if chosen).
- Unit tests cover at least slider and progress mapping behavior.

## M1.5 — Optional: adjacent semantics surfaces batched

Exit criteria (pick any subset, but keep each one gated):

- Scroll semantics: scroll positions/ranges emitted for scrollable roles (at least `Viewport`).
- Hierarchy level: `TreeItem` level emitted and mapped.
- Text flags/properties: `read_only` and `placeholder` mapped for text inputs.
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
