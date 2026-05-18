# WGPU Test Support Dead Code Prune v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`crates/fret-render-wgpu/tests/support/mod.rs` kept the last `#[allow(dead_code)]` in
`fret-render-wgpu`. The allowance existed because Rust integration tests compile each test file as a
separate crate: readback-only and explicit-format tests imported the whole shared support module but
did not call the default `render_scene_rgba8` helper.

The previous production dead-code prune intentionally left this test-only residue for a separate
support-module restructuring follow-on.

## Assumptions First

- Confident: the remaining allowance is caused by integration-test crate granularity, not by stale
  runtime code. Evidence: repo scan found the only `dead_code` hit in `tests/support/mod.rs`.
- Confident: most conformance tests still need the default `support/mod.rs` facade. Evidence: many
  tests import `render_scene_rgba8`.
- Confident: a small number of tests only need readback helpers, while one test needs explicit
  output-format rendering. Evidence: usage scan found `read_texture_rgba8` callers and one
  `render_scene_rgba8_with_format` caller.
- Likely: splitting narrow support entry modules is safer than copying helpers into individual test
  files because readback remains a single source.

## Target State

- No `#[allow(dead_code)]` remains in `crates/fret-render-wgpu/src` or `crates/fret-render-wgpu/tests`.
- `tests/support/readback.rs` owns raw texture readback and pixel indexing helpers.
- `tests/support/mod.rs` remains the default scene-rendering facade and reuses the readback helper.
- `tests/support/render_format.rs` owns the explicit output-format render helper for tests that do
  not need the default facade.
- Readback-only tests import the narrow readback support module.

## Out Of Scope

- Rewriting all WGPU conformance tests to a fixture framework.
- Changing readback semantics, formats, or pixel assertions.
- Moving test helpers into a published crate.
- Running the full WGPU conformance suite.

## Closure Policy

Close this lane once the allowance is removed, test targets compile, representative tests for all
three support entry points pass, and the dead-code scan plus workstream gates are clean.

## Closure

Closed on 2026-05-18 after splitting narrow test support entry modules and removing the final
`dead_code` allowance.
