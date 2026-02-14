# Text Line Breaking v1 — TODO

Scope: `docs/workstreams/text-line-breaking-v1.md`

## M0 — Conformance suite

- [ ] Add unit test fixtures for wrap breakpoints:
  - [ ] CJK punctuation (leading/trailing forbiddens),
  - [ ] identifiers (snake/camel),
  - [ ] paths/URLs,
  - [ ] emoji ZWJ/VS16,
  - [ ] long-token emergency wraps.
- [ ] Add a “wrap invariants” test module that reasserts:
  - [ ] trailing whitespace at soft wrap is selectable (keep existing test and expand coverage).

## M1 — Wrapper heuristic upgrade

- [ ] Replace `is_word_char`-based candidate selection in:
  - `crates/fret-render-wgpu/src/text/wrapper.rs`
  - with Unicode line break opportunities (UAX#14).
- [ ] Keep the current behavior for:
  - [ ] newline splitting,
  - [ ] ellipsis truncation.
- [ ] Add a micro perf regression guard for long paragraphs / resize jitter.

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

