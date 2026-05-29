# Material3 Slider Semantics Draw Region Motion Packet v2

Date: 2026-05-29
Task: M3PV2-068

## Truth

- Single-thumb Slider exposes `Slider` semantics with accessible label/value, numeric current/min/max,
  deterministic numeric step/jump, and increment/decrement/set-value actions.
- Continuous Slider semantics use the same 1% step and 10% page jump behavior as the keyboard path.
- RangeSlider exposes a non-focusable group plus separate start/end slider thumbs. Each thumb's
  numeric range is constrained by the peer thumb: start max is the current end value, and end min is
  the current start value.
- Slider publishes stable `.track`, `.active-track`, and `.handle` part ids, and the active draw
  region follows pointer updates.
- Pressed Slider state-layer opacity animates through fixed frames instead of appearing only as a
  settled broad golden outcome.

## Sources

- Compose Material3 `Slider.kt`: `SliderImpl` applies `minimumInteractiveComponentSize()`,
  `requiredSizeIn(...)`, `sliderSemantics(...)`, focusability, keyboard events, tap, and drag.
- Compose Material3 `Slider.kt`: continuous keyboard sliders use a 1% delta of the value range and
  page movement of `actualSteps / 10`; range slider keyboard movement clamps the start thumb to
  `valueRange.start..valueEnd` and the end thumb to `valueStart..valueRange.endInclusive`.
- Compose Material3 `Slider.kt`: `sliderSemantics`, `rangeSliderStartThumbSemantics`, and
  `rangeSliderEndThumbSemantics` publish state descriptions, progress/set-progress semantics, and
  range metadata for each interactive thumb.
- Compose Material3 `SliderTokens.kt` defines 44dp handle height, 4dp default handle width, 2dp
  pressed/focus handle width, 16dp active/inactive track height, full-rounded track/handle shapes,
  stop indicator size, and value-indicator spacing.
- Base UI `SliderRoot` / `SliderThumb` sources confirm the headless shape: a grouped slider root
  with separately focusable thumbs and range-specific value constraints.

MUI Material UI is not mirrored in this worktree's `repo-ref/`; this packet used local Compose
Material3 and Base UI references plus the existing Material Web v30 tokenized Fret implementation.

## Layer Finding

This packet found a Material recipe accessibility/proof-density gap, not a core mechanism gap:

- `fret-core` / `fret-ui` already expose numeric semantics metadata and derive `SetValue` for
  range-like roles when numeric value/range/step data is present.
- Material Slider already had pointer drag, keyboard step/page/home/end behavior, RTL behavior,
  range thumbs, stable part ids, tokenized track/handle rendering, and a state-layer animator.
- The recipe did not publish numeric step/jump for continuous sliders, so the semantic action model
  was weaker than the existing keyboard behavior.
- RangeSlider thumb semantics used the full component range for both thumbs, while Compose
  constrains each thumb's semantics to the peer thumb.
- The missing motion/layout work was focused proof: stable active-track/handle bounds and
  fixed-frame state-layer opacity evidence.

No core or kit primitive change was required in this packet.

## Artifacts

- `ecosystem/fret-ui-material3/src/slider.rs`
- `ecosystem/fret-ui-material3/tests/slider_state.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test slider_state
```

The new tests initially failed because single-thumb Slider had no numeric step metadata for the
continuous default, and RangeSlider start/end thumbs exposed the full max/min instead of their
peer-constrained semantic ranges. The draw-region and state-layer probes already passed, confirming
that the implementation gap was accessibility wiring plus proof density.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test slider_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_slider_suite_goldens_v1
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Residual Risk

- This packet proves current horizontal Slider and RangeSlider surfaces; vertical sliders are not
  exposed by the current Fret Material3 API.
- Value-indicator enter/exit choreography remains broad-golden covered rather than a dedicated
  fixed-frame motion assertion.
- Exact Material Web active/inactive track gap/corner-shrinking parity remains a future style packet
  if the design surface exposes those advanced track variants.
