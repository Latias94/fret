# Text Shaping Surface v1 — TODO

Scope: `docs/workstreams/text-shaping-surface-v1.md`

## M0 — Contracts + plumbing

- [x] Add `TextFontFeatureSetting` in `crates/fret-core/src/text/mod.rs`.
- [x] Extend `TextShapingStyle` with `features: Vec<TextFontFeatureSetting>`.
- [x] Define canonicalization:
  - [x] tag validation (4 ASCII bytes),
  - [x] last-writer-wins for duplicates,
  - [x] deterministic ordering for hashing.
- [x] Plumb mapping in `crates/fret-render-wgpu/src/text/parley_shaper.rs`:
  - [x] emit `StyleProperty::FontFeatures(FontSettings::List(...))`.
- [x] Update shaping fingerprint/keying:
  - [x] features participate in the shaping key,
  - [x] no paint-only fields leak into shaping key.
- [x] Add tests:
  - [x] canonicalization unit test,
  - [x] cache key test: toggling a feature changes the key and prepared output.

## M1 — Ecosystem adoption

- [x] Decide the default editor feature policy (code vs UI):
  - [x] disable `liga`/`calt` by default for code (common editor baseline),
  - [x] keep UI defaults unchanged.
- [x] Implement the policy at the ecosystem layer (avoid expanding mechanism-layer APIs):
  - [x] `ecosystem/fret-code-view`: apply the policy to code blocks (plain + syntax spans).
    - Evidence: `ecosystem/fret-code-view/src/code_block.rs` (`disable_ligatures`, `disable_contextual_alternates`)
  - [x] `ecosystem/fret-code-editor`: apply the policy to code text (plain + syntax + preedit).
    - Evidence: `ecosystem/fret-code-editor/src/editor/mod.rs` (`CodeFontFeaturePolicy`)
    - Evidence: `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (code shaping applied to spans)
- [x] Regression gates:
  - [x] a feature toggle changes shaping keys (no stale shaping/layout reuse).
    - Evidence: `crates/fret-render-wgpu/src/text/mod.rs` feature key tests.
  - [x] paint-only span changes do not trigger reshaping (shape cache hit under palette-only edits).
    - Evidence: `crates/fret-render-wgpu/src/text/mod.rs` + editor paint gates.
- [ ] Add a minimal demo/harness (recommended, bundled-font-friendly):
  - [ ] a “feature toggle” UI to flip `calt`/`liga` (and one `ssXX`) on a known sample string,
  - [ ] document which bundled/system fonts show a visible difference.

## Open questions

- [ ] Do we need a feature behavior conformance fixture beyond “keying correctness”?
  - Current tests validate deterministic canonicalization and cache invalidation.
  - A behavior-visible fixture should likely use bundled fonts (`fret_fonts`) to avoid platform
    font drift.
- [ ] Do we want to support a CSS-like `font-feature-settings` parser, or keep the struct-only API?
