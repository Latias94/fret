# WGPU Custom Effect V3 Raw Wanted Shape v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`CustomEffectV3Pass::raw_wanted` encoded shader-requested raw-source semantics, but the render-plan
field existed only on native builds. That forced cfg-shaped literals and summaries even though the
concept is part of the cross-platform render-plan data model.

The mismatch made wasm builds structurally different from native builds for a diagnostic/reporting
flag that should not affect whether V3 source views are available.

## Assumptions First

- Confident: `raw_wanted` is render-plan semantics, not a native-only execution resource. Evidence:
  it is populated from `sources.want_raw` beside `pyramid_wanted` and is consumed by reporting and
  summary tests.
- Confident: Custom Effect V3 execution still prepares both source views. Evidence:
  `prepare_custom_effect_v3_source_views` reads `src_raw` and `src_pyramid` independently of the
  wanted flags.
- Confident: lifecycle validation must keep unconditional reads for `src_raw` and `src_pyramid`.
  If the reads were gated by `raw_wanted` or `pyramid_wanted`, the validator would stop matching
  executor resource requirements.
- Likely: wasm should retain the same render-plan literal shape as native for diagnostics, summary,
  and future dump tooling even when shader capabilities differ.

## Target State

- `CustomEffectV3Pass::raw_wanted` exists on all targets.
- V3 render-plan construction and tests no longer need cfg attributes around this field.
- Plan lifecycle validation keeps `src_raw` and `src_pyramid` as unconditional reads.
- Native and wasm package checks compile without warnings from the unified field shape.

## Out Of Scope

- Changing Custom Effect V3 shader source selection.
- Changing source pyramid allocation or fallback behavior.
- Reworking render-plan reporting formats beyond the field shape cleanup.
- Restructuring test-only dead-code allowances.

## Closure Policy

Close this lane once native and wasm `fret-render-wgpu` checks pass, targeted Custom Effect V3 tests
pass, and the workstream catalog/JSON checks are clean.

## Closure

Closed on 2026-05-18 after unifying the Custom Effect V3 raw-source flag shape across native and
wasm builds.
