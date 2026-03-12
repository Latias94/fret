# Text Shaping Surface v1 — TODO

Scope: `docs/workstreams/text-shaping-surface-v1/text-shaping-surface-v1.md`

## M0 — Contracts + plumbing

- [x] Add `TextFontFeatureSetting` in `crates/fret-core/src/text/mod.rs`.
- [x] Extend `TextShapingStyle` with `features: Vec<TextFontFeatureSetting>`.
- [x] Define canonicalization:
  - [x] tag validation (4 ASCII bytes),
  - [x] last-writer-wins for duplicates,
  - [x] deterministic ordering for hashing.
- [x] Plumb mapping in `crates/fret-render-text/src/parley_shaper.rs`:
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
- [x] Add a minimal demo/harness (recommended, bundled-font-friendly):
  - [x] a "feature toggle" UI to flip `calt`/`liga` (and one `ssXX`) on a known sample string:
    - `apps/fret-ui-gallery/src/ui/previews/pages/editors/text/feature_toggles.rs`
    - `apps/fret-ui-gallery/src/spec.rs` (`PAGE_TEXT_FEATURE_TOGGLES`)
  - [x] document which bundled/system fonts show a visible difference:
    - `apps/fret-ui-gallery/src/docs.rs` (`DOC_TEXT_FEATURE_TOGGLES`)

## Open questions

- [x] Add a feature behavior conformance gate beyond cache-key correctness.
  - Gate: toggling `liga`/`calt` must be **behavior-visible** (not only key-visible) under a
    bundled-font-only environment (no system font dependency):
    - shaping output changes for at least one known ligature candidate string.
    - word wrap breakpoints can change under `TextWrap::Word` for at least one known candidate.
  - Evidence: `crates/fret-render-wgpu/src/text/mod.rs`
    (`open_type_feature_overrides_can_change_shaped_glyph_output_for_known_font_fixture`)
  - Evidence: `crates/fret-render-wgpu/src/text/mod.rs`
    (`open_type_feature_overrides_can_change_word_wrap_breakpoints_for_known_font_fixture`)
  - Note: this supersedes the deferred question below; keep it for historical rationale.

- [x] Do we need a feature behavior conformance fixture beyond “keying correctness”?
  - Answer: yes — and we now have one.
  - Rationale: keying correctness alone is not enough; we also want to know that feature overrides
    are actually applied by the shaping pipeline.
  - Evidence: `crates/fret-render-wgpu/src/text/mod.rs`
    (`open_type_feature_overrides_can_change_shaped_glyph_output_for_known_font_fixture`)
  - Evidence: `crates/fret-render-wgpu/src/text/mod.rs`
    (`open_type_feature_overrides_can_change_word_wrap_breakpoints_for_known_font_fixture`)
- [~] Do we want to support a CSS-like `font-feature-settings` parser, or keep the struct-only API?
  - Status: deferred.
  - Recommendation: keep the struct-only API for v1. Introduce a parser only when there is a
    product requirement for string-based configuration and we are ready to freeze an input grammar.
