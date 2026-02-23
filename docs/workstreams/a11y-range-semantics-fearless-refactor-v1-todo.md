# A11y range/numeric semantics (fearless refactor v1) — TODO

Last updated: 2026-02-23

## Contract + docs

- [x] Decide invariants: `extra.numeric.value` should be within `[min,max]` when all are present (validate as an error; do not clamp in `validate()`).
- [x] Decide indeterminate progress representation: omit `extra.numeric.value` (keep role as `ProgressBar`).
- [x] Confirm `extra.numeric.jump` in v1: keep as optional “page” increment (PageUp/PageDown); producers may omit.
- [x] Add an ADR describing the numeric/range semantics surface (ADR 0288).

## Adjacent semantics candidates (batchable during contract touch)

These are optional, but high leverage if we want to avoid follow-up “contract churn”.

- [x] Scroll semantics: add `extra.scroll.{x,x_min,x_max,y,y_min,y_max}` and map to AccessKit.
- [x] Scroll actions: add a portable `scroll_by` action (payload) and wire it through AccessKit where supported.
- [x] Orientation semantics: add `extra.orientation` and map to AccessKit `orientation` when present.
- [x] Tree/outline hierarchy: populate `extra.level` for `TreeItem` (and future `Heading`) and map to AccessKit `level`.
- [x] Text flags: add `SemanticsFlags::read_only` and map to AccessKit flags.
- [x] Text placeholder: add `extra.placeholder` and map to AccessKit `placeholder`.
- [x] Link URL: add `extra.url` and map to AccessKit `url`.
- [x] Image semantics: add `SemanticsRole::Image` (and treat `label` as alt text by convention).
- [x] Tri-state checked semantics: add `SemanticsFlags.checked_state` (`false/true/mixed`) and map to AccessKit toggled state; adopt in shadcn checkbox indeterminate state and gate via semantics snapshots.

## Core + runtime plumbing

- [x] Extend `fret-core` `SemanticsNode` with structured extras (`extra.numeric`, `extra.scroll`, etc.).
- [x] Extend `fret-ui` `SemanticsProps` and `SemanticsDecoration` to carry extras + read-only.
- [x] Forward extras into the snapshot in `crates/fret-ui/src/tree/ui_tree_semantics.rs`.
- [x] Add snapshot validation for numeric/scroll invariants via `SemanticsNode::validate()` (finite values, bounds order, out-of-bounds values, positive step/jump, `level` is 1-based).
- [x] Wire default `fret-bootstrap` UI driver hooks for common a11y actions (text selection, replace selected text, numeric set value, slider stepping).
- [x] Implement best-effort slider `SetValue` numeric handling via key sequences (Home/End/PageUp/PageDown/ArrowUp/ArrowDown).
- [x] Scroll containers use `SemanticsRole::Viewport` (instead of `Generic`) for clearer platform mappings.

## AccessKit adapter

- [x] Emit AccessKit numeric properties when fields are present.
- [x] Add adapter unit tests covering numeric + extra properties.
- [x] Expose portable slider stepper actions (`increment`/`decrement`) and map to AccessKit `Increment`/`Decrement`.

## Ecosystem adoption (shadcn/Radix alignment)

- [x] Slider: populate numeric value/min/max/step (and jump if chosen) on slider semantics nodes.
- [x] Progress: populate numeric now/min/max for determinate progress.
- [x] Update semantics-focused tests in `ecosystem/fret-ui-shadcn/tests/*` to assert numeric fields.

## Diagnostics + scripts

- [x] Prefer `extra.numeric.value` in `SetSliderValue` script step; fallback to parsing `value` string.
- [x] Add/adjust one `tools/diag-scripts/*set-slider-value*.json` script to assert numeric semantics explicitly.
- [x] Add a progress demo script gate asserting `extra.numeric` on `ProgressBar`.
- [x] Add a scroll demo script gate asserting `extra.scroll` is emitted for scroll containers.

## Quality gates

- [x] `cargo fmt -p fret-core`
- [x] `cargo nextest run -p fret-core`
- [ ] (If cross-crate moves happen) `python3 tools/check_layering.py`.
