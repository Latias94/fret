# Subtree Layout Dirty Aggregation (Fearless Refactor v1)

Status: **draft**

Last updated: **2026-03-01**

## Motivation

Some correctness bugs manifest as “content grew, but the scroll range didn’t” (e.g. switching a
Preview/Code tab near the bottom of a long docs page and being unable to scroll further).

At the mechanism level, these failures are usually caused by an implicit assumption:

> If a descendant needs layout, the scroll’s direct child root will also be marked layout-dirty.

When that assumption breaks (due to caching boundaries, truncation, bookkeeping bugs, or other
edge cases), a scroll container can reuse last-frame extent caches and “pin” its `max_offset` to a
stale value.

This workstream proposes a **mechanism-level** aggregation primitive that makes it cheap and
reliable to answer:

- “Does this node’s subtree contain any layout-dirty nodes?” (O(1))

…without requiring deep subtree scans or relying on invalidation bubbling to the nearest root.

## Non-goals

- Redesigning scroll extent computation end-to-end (tracked in
  `docs/workstreams/scroll-extents-dom-parity.md`).
- Changing view-cache semantics or contained-layout contracts:
  - Cache roots + cached subtree semantics:
    `docs/adr/0213-cache-roots-and-cached-subtree-semantics-v1.md`
  - Dirty views + notify:
    `docs/adr/0165-dirty-views-and-notify-gpui-aligned.md`
- Introducing CSS-like “percent under auto-height” semantics in the layout engine.

## Scope

- Primary: `crates/fret-ui` invalidation bookkeeping + scroll layout fast paths.
- Secondary: view-cache repair passes and diagnostics (optional, as needed for correctness).

## Documents

- Design: `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/MILESTONES.md`
- TODO tracker: `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/TODO.md`
- Evidence and gates: `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- Open questions: `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/OPEN_QUESTIONS.md`

## Recommended v1 decisions

For the current recommended direction on the major open questions (propagation strategy, counter vs
epoch, invariants), see:

- `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/OPEN_QUESTIONS.md`
