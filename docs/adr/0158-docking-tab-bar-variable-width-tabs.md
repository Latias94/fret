---
title: "ADR 0158: Docking Tab Bar Variable-Width Tabs and Scroll Semantics"
---

# ADR 0158: Docking Tab Bar Variable-Width Tabs and Scroll Semantics

Status: Proposed

Scope: `ecosystem/fret-docking` tab bar geometry, hit testing, scrolling, and insertion semantics. This ADR
does not change the `DockOp` transaction vocabulary.

Related:

- ADR 0017: `docs/adr/0017-multi-window-display-and-dpi.md`
- ADR 0013: `docs/adr/0013-docking-ops-and-persistence.md`
- ADR 0041: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- ADR 0072: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- ADR 0155: `docs/adr/0155-docking-tab-dnd-contract.md`
- ADR 0157: `docs/adr/0157-headless-dnd-v1-contract-surface.md`

## Context

Fret’s docking tab bar is currently implemented with a fixed per-tab width constant. This simplifies:

- rendering,
- hit testing (index = `floor(x / W)`),
- insert index computation (insert line at `i * W`),
- tab scrolling (content width = `n * W`).

However, editor-grade shells often need variable-width tabs:

- longer file names should consume more space (within bounds),
- short names should be compact,
- close buttons and indicators (dirty, pinned) should not overlap labels,
- insertion and reordering must remain deterministic even with mixed tab widths.

If we do not lock a contract early, variable-width support tends to cause repeated refactors across
layout/paint/hit-test/dnd, and behavior drift across platforms.

## Goals

1. Support variable-width tabs without changing `DockOp` shapes.
2. Preserve determinism: hit-testing and insert indices are stable for identical event streams.
3. Keep coordinate semantics explicit: window-local logical pixels (ADR 0017).
4. Keep insertion semantics aligned with headless DnD (`InsertionSide`) and docking DnD contracts.
5. Maintain reasonable performance (linear or better) for typical tab counts.

## Non-Goals

- Implementing “tab shrink-to-fit with multi-row wrapping”.
- Defining a global tab styling/theming API (policy remains in docking).
- RTL semantics and bidi text shaping policy (tracked as an open question).

## Decision

### 1) Canonical tab geometry is expressed as rects in window-local logical px

For a given `DockNode::Tabs`, the docking UI computes a `TabBarGeometry` conceptually containing:

- `tab_bar_rect: Rect`
- `scroll_px: Px` (horizontal offset)
- `tabs: Vec<TabGeom>` where `TabGeom` includes:
  - `panel: PanelKey`
  - `rect: Rect` (the clickable tab bounds in window-local logical px)
  - `close_rect: Rect` (if a close button is shown)

This geometry must be the single source of truth for:

- painting,
- hit-testing (tab index under pointer),
- insertion computation (before/after),
- scroll bounds.

### 2) Tab widths are derived from measured content and clamped

Tab width is computed from:

- measured title width (text metrics in logical px),
- fixed chrome widths (padding, close button, indicators),
- and clamped by a docking policy range:
  - `min_tab_w_px <= tab_w_px <= max_tab_w_px`

Defaults:

- `min_tab_w_px`: small but non-zero (e.g. 72px) to keep click targets usable.
- `max_tab_w_px`: prevents one tab from monopolizing the bar (e.g. 240px).

If text measurement is unavailable for a frame, the implementation may fall back to a conservative default
width, but must remain deterministic given the available inputs.

### 3) Scroll semantics are defined over the total content width (sum of tab widths)

The tab bar scroll offset is:

- expressed in window-local logical pixels (`Px`),
- clamped to `[0 .. total_width - tab_bar_rect.width]` (or `0` if content fits),
- applied as a translation when mapping pointer positions to “content space”.

“Ensure visible” behavior (optional policy):

- when a tab becomes active, docking may adjust `scroll_px` to ensure the active tab rect is fully visible.

### 4) Hit-testing uses geometry, not tab indices times a constant

Given `position: Point` in window-local logical px, and a tab bar geometry:

- first check `tab_bar_rect.contains(position)`,
- then map into content space by adding `scroll_px`,
- then find the first tab rect containing the mapped point.

Performance guidance:

- an O(n) scan is acceptable for small `n`,
- for larger `n`, a prefix-sum + binary search is recommended (implementation detail; not a hard requirement).

### 5) Insertion index uses `InsertionSide` over the *over* tab rect

Insertion uses the stable before/after semantic from ADR 0157:

- determine the *over* tab rect under the pointer (using hit-testing),
- compute `InsertionSide` by splitting the rect along `Axis::X`,
- derive `insert_index` as:
  - `Before` → `over_index`
  - `After` → `over_index + 1`

Edge behavior:

- pointer before the first tab → `insert_index = 0`
- pointer after the last tab → `insert_index = tab_count`

This keeps docking’s `DockOp::MovePanel { insert_index }` unchanged while making the meaning of the index
stable under variable-width geometry.

## Consequences

### Benefits

- Variable-width tabs become a mechanical refactor (geometry-driven) rather than a behavior rewrite.
- Insertion semantics remain stable and align with headless DnD (`InsertionSide`).
- Scroll behavior becomes well-defined and testable.

### Costs

- Requires a shared geometry path for paint + hit-test + insert computation.
- Adds a new set of edge cases (scroll bounds, measurement availability) that must be covered by tests.

## Migration Plan (Use-Case Driven)

1. Introduce internal tab geometry computation and make hit-test/insert-index consume it while keeping
   fixed-width defaults (no UX change).
2. Add clamped variable-width computation sourced from measured tab titles.
3. Add/expand tests:
   - hit-test under scroll,
   - before/after insertion under variable widths,
   - deterministic insert index behavior.

## Open Questions

1. Do we need RTL-aware insertion semantics (`Axis::X` direction inversion) at the docking policy layer?
2. Should `min/max tab width` become user-configurable settings, or stay theme/policy constants initially?
3. How do “dirty/pinned” indicators affect width and close button placement (policy vs mechanism split)?
