---
title: "ADR 0070: Virtualization Contract (Virtual List Metrics + Range)"
---

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- gpui-component: https://github.com/longbridge/gpui-component
- virtualizer (Rust): https://github.com/Latias94/virtualizer
- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

# ADR 0070: Virtualization Contract (Virtual List Metrics + Range)

## Status

Accepted (MVP).

## Context

Fret targets “Tailwind primitives + shadcn recipes”, which implies high-frequency UI patterns that
require virtualization to be performant and consistent:

- command palette lists (cmdk-style),
- tables and trees (variable-height rows, expand/collapse),
- menus with long lists,
- log viewers / inspectors.

To avoid “virtual list drift” across components, the runtime must provide a deterministic,
testable virtualization substrate aligned with an industry-standard vocabulary.

In the UI ecosystem, TanStack Virtual originally defined the de-facto contract vocabulary. In Fret, we standardize on the Rust `virtualizer` engine and port the vocabulary and outcomes.

- item count, estimate size, measured size,
- item offsets and total size,
- visible range + overscan,
- scroll-to-item behavior with margins.

Fret is not a DOM runtime, but can port these outcomes as pure algorithms and stable data
structures.

## Decision

`crates/fret-ui` provides a stable virtualization contract surface in `fret_ui::virtual_list`:

- `VirtualListMetrics`: deterministic, variable-size metrics (Fenwick-backed) for item offsets and
  total size.
- `VirtualRange`: visible range (inclusive indices) + overscan parameters.
- `VirtualItem`: resolved item geometry (`start/end/size`) for a given key + index.
- `default_range_extractor(VirtualRange) -> Vec<usize>`: TanStack-like overscan expansion.
- `VirtualListMetrics::visible_range(...)`: visible range computation for a viewport.
- `VirtualListMetrics::scroll_offset_for_item(...)`: scroll-to-item target offset computation
  (Start/Center/End/Nearest).

### Invariants

- Given identical inputs, outputs are deterministic.
- `VirtualRange` uses **inclusive** indices:
  - `start_index..=end_index` is the visible range before overscan expansion.
  - `count == 0` implies an empty range.
- `VirtualListMetrics` offsets include `scroll_margin` and per-row `gap`:
  - `offset_for_index(0) == scroll_margin`
  - `total_height()` includes the final scroll margin and does not double-count the last gap.
- Measuring an item updates offsets incrementally; it must be safe to call repeatedly.
- `visible_range(...)` must clamp safely for empty lists and zero-height viewports.

### Relationship to the declarative element layer

The declarative `VirtualList` element (ADR 0039) is a **consumer** of this contract:

- The element state stores metrics and a key→measured-size cache.
- The element computes `VirtualRange` + `visible_items` each frame.
- Painting clips each visible row to avoid overdraw.

The virtualization contract is intentionally independent of authoring style (retained widgets vs.
declarative elements): any component may use the same metrics/range API.

## Non-goals (P0)

- Sticky headers/columns.
- Two-dimensional virtualization (grid virtualization).
- Smart “scroll anchoring” across insertions/removals (P1).
- Accessibility-specific semantics for virtualized content (handled by a11y layer; requires a
  separate ADR).

## References

- virtualizer (Rust): `repo-ref/virtualizer` (primary)
- GPUI patterns (engineering reference): `repo-ref/gpui-component`
- Runtime contract surface: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Zed/GPUI runtime list primitives (non-normative):
  - `repo-ref/zed/crates/gpui/src/elements/list.rs`
  - `repo-ref/zed/crates/gpui/src/elements/uniform_list.rs`
