# WGPU Conformance Harness v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`crates/fret-render-wgpu/tests/` has many GPU readback conformance tests. Each integration-test file
currently owns a copy of the same low-level harness work:

- create an `Rgba8Unorm` output texture,
- render a `Scene` into it,
- copy the texture into a padded readback buffer,
- map the buffer,
- strip row padding,
- and sample individual RGBA pixels.

The duplication is not a renderer behavior bug, but it is an architecture problem for the test
surface: every new semantic gate becomes noisy, and harness fixes can drift across tests. This lane
extracts a small integration-test support module and proves it against the path-related tests that
were just used to close ADR 0080 / ADR 0277 / ADR 0278 evidence.

## Assumptions First

- Confident: integration-test support belongs under `crates/fret-render-wgpu/tests/support/` so it
  is shared by test crates without becoming production API. Evidence: Cargo treats nested modules
  under `tests/` as support files only when referenced by each integration test. If wrong, the helper
  should move to a dev-only crate or a public test-support feature.
- Confident: the first slice should migrate only path-related tests. Evidence:
  `path_base_conformance.rs`, `path_stroke_style_v2_conformance.rs`, `path_paint_conformance.rs`,
  and `path_material_paint_conformance.rs` all duplicate the same readback/render helpers while
  sharing the same renderer contract family. If wrong, the slice should stop before touching effect
  or text tests.
- Likely: the helper should stay deliberately boring: `render_scene_rgba8`, `render_scene_rgba8_at_1x`,
  `read_texture_rgba8`, and `pixel_rgba`. If wrong, richer fixture APIs can be a follow-on after
  multiple callsites prove a common shape.
- Likely: this is a test-surface refactor only and should not update ADR semantics. Evidence: no
  renderer production behavior changes are intended. If wrong, a failing gate should either fix the
  path contract lane or split a separate semantic follow-on.

## Target State

- A small `crates/fret-render-wgpu/tests/support/mod.rs` owns common render/readback/pixel sampling
  helpers.
- Path-related conformance tests use that support module:
  - `path_base_conformance.rs`
  - `path_stroke_style_v2_conformance.rs`
  - `path_paint_conformance.rs`
  - `path_material_paint_conformance.rs`
- The tests keep their existing assertions and semantics.
- The lane closes after the first path-related batch unless a concrete second batch is worth a
  narrower follow-on.

## Out Of Scope

- Migrating every WGPU conformance test in one commit.
- Adding new renderer behavior coverage.
- Introducing fixtures or macros.
- Moving support code into production crates.
- Changing GPU formats, clear colors, adapter policy, or skip behavior.

## First Slice

`WCH-010`: add `tests/support/mod.rs`, migrate the four path-related tests, and run the affected
test batch plus backend compile checks.

## Closure

Closed on 2026-05-18 after the shared helper extraction and path batch migration landed. If a new
test family needs the same treatment, split a narrower follow-on instead of reopening this lane.
