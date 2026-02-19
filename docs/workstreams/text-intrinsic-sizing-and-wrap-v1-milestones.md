# Text Intrinsic Sizing + Wrap Semantics v1 — Milestones

Status: Active
Start: 2026-02-19

## M0 — Spec + regression map (1–2 days)

- [x] Land ADR 0251 scope and semantics.
- [x] Accept v1 segmentation rules for `TextWrap::Word` unbreakable segments (Parley/UAX#14).
- [ ] Identify the top 5 UI Gallery pages affected by shrink-wrap + wrap (repros + expected).
- [x] Add/confirm at least 1 deterministic gate:
  - a unit test for intrinsic widths (preferred), and/or
  - a diag script + screenshot bundle for the most visible regression.
  - [x] Landed: `crates/fret-render-wgpu/src/text/wrapper.rs` tests + UI Gallery diag scripts (see evidence anchors).

## M1 — True `min-content` for `TextWrap::Word` (core landing)

- [x] Implement “longest token width” measurement for `TextWrap::Word` intrinsic sizing.
- [ ] Remove/relax UI-level placeholder-width normalization where it becomes redundant.
- [ ] Add unit tests covering:
  - ASCII labels (no spaces),
  - spaced phrases,
  - mixed CJK + Latin,
  - emoji/ZWJ safety (no invalid boundaries),
  - attributed spans (shaping attributes affect widths; paint-only does not).

## M2 — Ecosystem authoring helpers (reduce drift)

- [x] Add UI kit helpers for common patterns:
  - prose with break-words (`WordBreak`),
  - code/editor surfaces (`Grapheme`),
  - labels/tabs/menu rows (`nowrap` + optional truncation).
- [ ] Update shadcn recipes to use the helpers rather than ad-hoc wrap selections.

## M3 — Explicit multiline clamp (separate feature)

- [ ] Design and land an explicit `line-clamp` contract (API + geometry rules).
- [ ] Add conformance tests and/or web-vs-fret layout parity where applicable.
