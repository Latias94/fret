# WGPU Custom Effect V3 Raw Wanted Shape v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Raw Flag Shape Unified

Exit criteria:

- `CustomEffectV3Pass::raw_wanted` is present on native and wasm targets.
- V3 pass literals no longer need cfg attributes for the raw-source flag.
- Native and wasm package checks compile.

Status: Complete on 2026-05-18.

## M1 - V3 Source Semantics Preserved

Exit criteria:

- Plan validation still treats `src_raw` and `src_pyramid` as executor-required reads.
- Targeted Custom Effect V3 planning and summary tests pass.

Status: Complete on 2026-05-18.

## M2 - Workstream Closed

Exit criteria:

- Workstream catalog, JSON validation, and diff whitespace checks pass.
- Closeout note records the native/wasm shape invariant and the lifecycle-validation rationale.

Status: Complete on 2026-05-18.
