# WGPU Text Measure Dead Branch Prune v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`crates/fret-render-wgpu/src/text/measure.rs` still carried two large `#[cfg(any())]` blocks after
`TextSystem::measure` and `TextSystem::measure_attributed` had been reduced to facades over
`fret-render-text::TextMeasureCaches`.

`#[cfg(any())]` is an always-false cfg expression. In this file it kept an old inline measurement
implementation as unreachable source text, making the WGPU text module look like it still owned
measurement caching, wrapping, and shaping-cache policy.

## Assumptions First

- Confident: the shipped measurement path is `TextLayoutCacheState::measure`, backed by
  `fret_render_text::TextMeasureCaches`. Evidence: both public methods return before the dead
  blocks.
- Confident: the dead blocks are stale implementation copies, not feature-gated target code.
  Evidence: repo scan found no feature/target reference for these branches and `#[cfg(any())]`
  cannot compile on any target.
- Confident: WGPU should remain a facade over the shared text measurement owner. Evidence:
  `TextLayoutCacheState` stores `TextMeasureCaches`, and `text_measure_matches_prepare` already
  verifies renderer-facing measure/prepare parity.
- Likely: keeping the dead copy increases future refactor risk because maintainers may update the
  real shared implementation while missing the unreachable duplicate.

## Target State

- `TextSystem::measure` and `TextSystem::measure_attributed` directly return the shared
  `TextMeasureCaches` results without dead code after the call.
- No `#[cfg(any())]` remains in `crates/fret-render-wgpu/src/text/measure.rs`.
- Text measurement behavior remains unchanged and is verified through existing measure/prepare
  parity tests.

## Out Of Scope

- Changing `fret-render-text::TextMeasureCaches` behavior.
- Retuning measurement cache sizes or shaping-cache thresholds.
- Broad wasm/native text cfg consolidation.
- Text atlas/runtime diagnostics cleanup.

## Closure Policy

Close this lane once the dead branches are deleted, targeted text measurement tests pass, and the
workstream catalog/JSON/diff checks are clean.

## Closure

Closed on 2026-05-18 after pruning the unreachable WGPU text measurement implementation copies.
