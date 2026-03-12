# Text Interactive Spans v1 — TODO

Status: Active
Start: 2026-02-19

This checklist tracks `docs/workstreams/text-interactive-spans-v1/text-interactive-spans-v1.md`.

## Product-facing repros

- [ ] UI Gallery: add a minimal “rich paragraph with interactive links” page (no editor).
- [x] Diag script: narrow window screenshots that prove long URL spans wrap within the paragraph.
- [x] Diag script: click a link span and assert the activation callback fires (bundle evidence).

## Mechanism surfaces

- [x] Decide the v1 API surface for per-span tags (core vs ui-kit).
- [x] Implement pointer hit-testing: local point → text index → span tag.
- [x] Add hover affordances (cursor change + optional underline) without per-component hacks.
- [ ] Ensure redaction mode can still run scripts (prefer `test_id` tags for spans).

## Decorations

- [x] Ensure underline/strikethrough paint is correct across wrapped lines for link spans.
- [ ] Add a span-hover policy surface (underline-on-hover vs always-underline).

## Tests (fast, deterministic)

- [ ] Unit test: span hit-testing maps points on wrapped lines to the correct span tag.
- [x] Regression test: long link token wraps under `TextWrap::WordBreak`.
- [x] Selection/hit-test invariants remain valid with tagged spans enabled.
- [x] Semantics v1: inline spans metadata is present in `SemanticsSnapshot` (ADR 0283).
