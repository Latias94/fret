---
title: Gesture Recognizers (v1) — TODO
status: draft
date: 2026-02-11
---

# Gesture Recognizers (v1) — TODO

Workstream entry:

- `docs/workstreams/gesture-recognizers-v1.md`

## Contract / layering

- [ ] Ensure all gesture code lives in `ecosystem/` (policy), not in `crates/fret-ui` (mechanism).
- [ ] Add a short “why this is policy” note and link to ADR 0066.

## Pan recognizer (M0)

- [ ] Define a minimal `Pan` state machine:
  - idle → tracking → dragging → ended/canceled.
- [ ] Implement threshold arming and pointer capture once dragging starts.
- [ ] Expose a small, declarative-friendly API for attaching to `PointerRegion`.
- [ ] Add tests for:
  - threshold arming,
  - delta direction,
  - cancel behavior (pointer cancel clears state).

## Scroll integration (M1)

- [ ] Integrate into `fret-ui-shadcn::ScrollArea` touch scrolling.
- [ ] Add a conformance test: tap does not scroll; drag scrolls and suppresses click.

