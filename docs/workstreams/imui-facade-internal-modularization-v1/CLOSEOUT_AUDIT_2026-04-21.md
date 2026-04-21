# ImUi Facade Internal Modularization v1 - Closeout Audit (2026-04-21)

Status: closed
Last updated: 2026-04-21

## Closeout verdict

This lane is closed.

It achieved the intended narrow goal: reduce internal `fret-ui-kit::imui` refactor hazard without
widening the public surface, reopening runtime contracts, or hiding new helper behavior inside
structural cleanup.

The shipped owner split now looks like:

- `options.rs` -> smaller private owner files
- `response.rs` -> smaller private owner files
- `interaction_runtime.rs` -> owner files under `interaction_runtime/`
- root `imui.rs` support/type blocks -> `facade_support.rs` and `floating_options.rs`
- root facade writer glue -> `facade_writer.rs`

## Why the lane closes here

The original hotspot pressure came from a few files mixing unrelated concerns.

That is no longer true:

- the root `imui.rs` file is now a thin outward hub,
- support helpers, floating facade types, runtime bookkeeping, options, responses, and writer glue
  each have explicit ownership,
- and further changes would now be better scoped by helper family or policy question rather than by
  generic internal modularization.

## Future-work rule

Do not reopen this lane by default.

If future pressure appears, open a narrower follow-on instead:

- writer-family decomposition if `facade_writer.rs` itself becomes the review bottleneck,
- helper/policy parity if a specific menu/tab/collection/runtime behavior needs to grow,
- or a proof lane if first-party demos/tests expose a new gap.

## Closed-with evidence

- `docs/workstreams/imui-facade-internal-modularization-v1/M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/M1_OPTIONS_RESPONSE_SLICE_2026-04-21.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/EVIDENCE_AND_GATES.md`
