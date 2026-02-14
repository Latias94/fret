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

- [ ] Decide the default editor feature policy (code vs UI):
  - [ ] disable `liga`/`calt` by default for code (common editor baseline),
  - [ ] keep UI defaults unchanged.
- [ ] Wire the policy in `ecosystem/fret-code-view` and/or `ecosystem/fret-code-editor`.
- [ ] Add a minimal demo string and a “feature toggle” harness page (optional).

## Open questions

- [ ] Do we need a feature behavior conformance fixture beyond “keying correctness”?
  - Current tests validate deterministic canonicalization and cache invalidation.
  - A behavior-visible fixture should likely use bundled fonts (`fret_fonts`) to avoid platform
    font drift.
- [ ] Do we want to support a CSS-like `font-feature-settings` parser, or keep the struct-only API?
