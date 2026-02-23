# A11y range/numeric semantics (fearless refactor v1) — TODO

Last updated: 2026-02-23

## Contract + docs

- [ ] Decide invariants: should `extra.numeric.value` be clamped to `[min,max]` or can it be out-of-range?
- [ ] Decide indeterminate progress representation (omit `extra.numeric.value` vs sentinel).
- [ ] Confirm we want `extra.numeric.jump` in v1 (PageUp/PageDown increments).
- [ ] (If needed) add/update an ADR describing the numeric/range semantics surface.

## Adjacent semantics candidates (batchable during contract touch)

These are optional, but high leverage if we want to avoid follow-up “contract churn”.

- [x] Scroll semantics: add `extra.scroll.{x,x_min,x_max,y,y_min,y_max}` and map to AccessKit.
- [ ] Scroll actions: add a portable `scroll_by` action (payload) and wire it through AccessKit where supported.
- [ ] Tree/outline hierarchy: populate `extra.level` for `TreeItem` (and future `Heading`) and map to AccessKit `level`.
- [x] Text flags: add `SemanticsFlags::read_only` and map to AccessKit flags.
- [x] Text placeholder: add `extra.placeholder` and map to AccessKit `placeholder`.
- [x] Link URL: add `extra.url` and map to AccessKit `url`.
- [x] Image semantics: add `SemanticsRole::Image` (and treat `label` as alt text by convention).

## Core + runtime plumbing

- [x] Extend `fret-core` `SemanticsNode` with structured extras (`extra.numeric`, `extra.scroll`, etc.).
- [x] Extend `fret-ui` `SemanticsProps` and `SemanticsDecoration` to carry extras + read-only.
- [x] Forward extras into the snapshot in `crates/fret-ui/src/tree/ui_tree_semantics.rs`.
- [ ] Add snapshot validation for numeric ranges (best-effort; strict mode later).

## AccessKit adapter

- [x] Emit AccessKit numeric properties when fields are present.
- [x] Add adapter unit tests covering numeric + extra properties.

## Ecosystem adoption (shadcn/Radix alignment)

- [x] Slider: populate numeric value/min/max/step (and jump if chosen) on slider semantics nodes.
- [x] Progress: populate numeric now/min/max for determinate progress.
- [ ] Update semantics-focused tests in `ecosystem/fret-ui-shadcn/tests/*` to assert numeric fields.

## Diagnostics + scripts

- [x] Prefer `extra.numeric.value` in `SetSliderValue` script step; fallback to parsing `value` string.
- [ ] Add/adjust one `tools/diag-scripts/*set-slider-value*.json` script to assert numeric semantics explicitly.

## Quality gates

- [ ] `cargo fmt` for touched crates.
- [ ] `cargo nextest run -p fret-a11y-accesskit` (and any affected packages) or `cargo test` fallback.
- [ ] (If cross-crate moves happen) `python3 tools/check_layering.py`.
