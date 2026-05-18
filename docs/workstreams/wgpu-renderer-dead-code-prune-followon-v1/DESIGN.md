# WGPU Renderer Dead Code Prune Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`crates/fret-render-wgpu` still carried a small set of production `#[allow(dead_code)]` suppressions
after the WGPU conformance helper sweep and image-registry metadata cleanup.

The suppressions hid two different situations:

- stale production code with no runtime reader or caller,
- test-only helpers that still need local allowances because of Rust's integration-test/module
  compilation model.

This lane removes only the production residue with clear evidence and keeps the test-only
allowances explicit.

## Assumptions First

- Confident: `BindGroupCaches::invalidate_all` is stale. Evidence: repo-wide reference search found
  only the method definition. If wrong, the `fret-render-wgpu` test compile gate would fail.
- Confident: `TextSystem::prepare_input` is stale. Evidence: text preparation call sites use
  `prepare`, `prepare_attributed`, or lower-level wrapping/shaping paths directly.
- Confident: `subpixel_mask_to_alpha` is stale production code. Evidence: its only caller was a
  unit test for the helper itself; no text atlas/render path called it.
- Confident: `DownsampleHalfQuarter.half_size` is stale retained output state. Evidence: the helper
  still uses the local half-size value to build passes and stack entries, but no caller reads the
  returned field.
- Likely: `tests/support::render_scene_rgba8` and `src/renderer/tests.rs` keep legitimate
  `dead_code` allowances because those modules are compiled in contexts where not every helper is
  used by every test binary.

## Target State

- No production `#[allow(dead_code)]` remains in `crates/fret-render-wgpu/src`.
- Stale helper code is deleted instead of hidden behind suppressions.
- Called production helpers no longer carry stale `dead_code` suppressions.
- Test-only suppressions remain out of this lane unless their compilation model changes.

## Out Of Scope

- Reworking test support module structure.
- Moving renderer unit-test helpers into separate fixtures.
- Changing render-plan, text shaping, or bind-group cache behavior.

## Closure Policy

Close this lane once production dead-code suppressions are removed and targeted text/render-plan
tests plus the backend test compile gate pass.

## Closure

Closed on 2026-05-18 after pruning production dead-code residue from `fret-render-wgpu`.
