# Text Line Breaking v1 — TODO

Scope: `docs/workstreams/text-line-breaking-v1.md`

## M0 — Conformance suite

- [x] Add a fixture-driven wrap conformance harness (baseline goldens):
  - `crates/fret-render-wgpu/src/text/wrapper.rs` (`text_wrap_conformance_v1_fixtures`)
  - `crates/fret-render-wgpu/src/text/tests/fixtures/text_wrap_conformance_v1.json`
- [x] Expand fixture coverage and expectations (initial set):
  - [x] CJK punctuation (leading/trailing forbiddens),
  - [x] identifiers (snake/camel),
  - [x] paths/URLs,
  - [x] emoji ZWJ/VS16,
  - [x] long-token emergency wraps.
- [ ] Add a dedicated “wrap invariants” module (optional):
  - [ ] trailing whitespace at soft wrap is selectable (keep existing gate in `crates/fret-render-wgpu/src/text/mod.rs` and expand coverage).

## M1 — Wrapper heuristic upgrade

- [x] Replace `is_word_char`-based candidate selection in:
  - `crates/fret-render-wgpu/src/text/wrapper.rs`
  - with Unicode line break opportunities (UAX#14) via `swash::text::analyze` (keep a small heuristic fallback for now).
- [x] Keep the current behavior for:
  - [x] newline splitting,
  - [x] ellipsis truncation.
- [x] Add a micro perf regression guard for long paragraphs / resize jitter.

## M2 — Parley line breaking integration

- [ ] Prototype a new Parley shaper entry point:
  - [ ] “shape paragraph with wrap width” returning multiple lines + cluster mapping.
- [ ] Stage rollout:
  - [ ] LTR-only first,
  - [ ] keep compatibility wrapper for RTL.
- [ ] Update caret/selection/hit-test tests to exercise the new path.

## Open questions

- [ ] Which Unicode line-breaking implementation should we standardize on for M1 (if not Parley)?
- [ ] Do we want a dedicated “code wrap mode” distinct from UI `TextWrap::Word`?
