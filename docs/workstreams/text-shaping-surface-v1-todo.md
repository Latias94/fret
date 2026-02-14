# Text Shaping Surface v1 — TODO

Scope: `docs/workstreams/text-shaping-surface-v1.md`

## M0 — Contracts + plumbing

- [ ] Add `TextFontFeatureSetting` in `crates/fret-core/src/text/mod.rs`.
- [ ] Extend `TextShapingStyle` with `features: Vec<TextFontFeatureSetting>`.
- [ ] Define canonicalization:
  - [ ] tag validation (4 ASCII bytes),
  - [ ] last-writer-wins for duplicates,
  - [ ] deterministic ordering for hashing.
- [ ] Plumb mapping in `crates/fret-render-wgpu/src/text/parley_shaper.rs`:
  - [ ] emit `StyleProperty::FontFeatures(FontSettings::List(...))`.
- [ ] Update shaping fingerprint/keying:
  - [ ] features participate in the shaping key,
  - [ ] no paint-only fields leak into shaping key.
- [ ] Add tests:
  - [ ] canonicalization unit test,
  - [ ] cache key test: toggling a feature changes the key and prepared output.

## M1 — Ecosystem adoption

- [ ] Decide the default editor feature policy (code vs UI):
  - [ ] disable `liga`/`calt` by default for code (common editor baseline),
  - [ ] keep UI defaults unchanged.
- [ ] Wire the policy in `ecosystem/fret-code-view` and/or `ecosystem/fret-code-editor`.
- [ ] Add a minimal demo string and a “feature toggle” harness page (optional).

## Open questions

- [ ] Which bundled font(s) should we use for feature conformance tests?
- [ ] Do we want to support a CSS-like `font-feature-settings` parser, or keep the struct-only API?

