# Text Interactive Spans v1 — TODO

Status: Draft
Start: 2026-02-19

This checklist tracks `docs/workstreams/text-interactive-spans-v1.md`.

## Product-facing repros

- [ ] UI Gallery: add a minimal “rich paragraph with interactive links” page (no editor).
- [ ] Diag script: narrow window screenshots that prove long URL spans wrap within the paragraph.
- [ ] Diag script: click a link span and assert the activation callback fires (bundle evidence).

## Mechanism surfaces

- [ ] Decide the v1 API surface for per-span tags (core vs ui-kit).
- [ ] Implement pointer hit-testing: local point → text index → span tag.
- [ ] Add hover affordances (cursor change + optional underline) without per-component hacks.
- [ ] Ensure redaction mode can still run scripts (prefer `test_id` tags for spans).

## Decorations

- [ ] Ensure underline/strikethrough paint is correct across wrapped lines for:
  - plain text,
  - link spans,
  - mixed spans with different fonts/weights.
- [ ] Remove ad-hoc decoration overlays in `fret-markdown` once mechanism supports it.

## Tests (fast, deterministic)

- [ ] Unit test: span hit-testing maps points on wrapped lines to the correct span tag.
- [ ] Regression test: long link token wraps under `TextWrap::WordBreak`.
- [ ] Selection/hit-test invariants remain valid with tagged spans enabled.
