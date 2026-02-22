# A11y range/numeric semantics (fearless refactor v1) — TODO

Last updated: 2026-02-22

## Contract + docs

- [ ] Decide invariants: should `numeric_value` be clamped to `[min,max]` or can it be out-of-range?
- [ ] Decide indeterminate progress representation (omit `numeric_value` vs sentinel).
- [ ] Confirm we want `numeric_value_jump` in v1 (PageUp/PageDown increments).
- [ ] (If needed) add/update an ADR describing the numeric/range semantics surface.

## Adjacent semantics candidates (batchable during contract touch)

These are optional, but high leverage if we want to avoid follow-up “contract churn”.

- [ ] Scroll semantics: add `scroll_{x,y}` and `{min,max}` fields and map to AccessKit for `Viewport`/scrollables.
- [ ] Scroll actions: add a portable `scroll_by` action (payload) and wire it through AccessKit where supported.
- [ ] Tree/outline hierarchy: add `level` for `TreeItem` (and future `Heading`) and map to AccessKit `level`.
- [ ] Text flags: add `read_only` (and possibly `required/invalid`) and map to AccessKit flags.
- [ ] Text placeholder: add a portable `placeholder` string and map to AccessKit `placeholder`.
- [ ] Link URL: add `url` string for `SemanticsRole::Link` and map to AccessKit `url`.
- [ ] Image semantics: add `SemanticsRole::Image` + label-as-alt-text behavior.

## Core + runtime plumbing

- [ ] Extend `fret-core` `SemanticsNode` with numeric/range fields (additive).
- [ ] Extend `fret-ui` `SemanticsProps` and `SemanticsDecoration` to carry numeric/range fields.
- [ ] Forward numeric/range fields into the snapshot in `crates/fret-ui/src/tree/ui_tree_semantics.rs`.
- [ ] Add snapshot validation for numeric ranges (best-effort; strict mode later).

## AccessKit adapter

- [ ] Emit AccessKit numeric properties when fields are present.
- [ ] Add adapter unit tests covering slider + progress.

## Ecosystem adoption (shadcn/Radix alignment)

- [ ] Slider: populate numeric value/min/max/step (and jump if chosen) on slider semantics nodes.
- [ ] Progress: populate numeric now/min/max for determinate progress.
- [ ] Update semantics-focused tests in `ecosystem/fret-ui-shadcn/tests/*` to assert numeric fields.

## Diagnostics + scripts

- [ ] Prefer `numeric_value` in `SetSliderValue` script step; fallback to parsing `value` string.
- [ ] Add/adjust one `tools/diag-scripts/*set-slider-value*.json` script to assert numeric semantics explicitly.

## Quality gates

- [ ] `cargo fmt` for touched crates.
- [ ] `cargo nextest run -p fret-a11y-accesskit` (and any affected packages) or `cargo test` fallback.
- [ ] (If cross-crate moves happen) `python3 tools/check_layering.py`.
