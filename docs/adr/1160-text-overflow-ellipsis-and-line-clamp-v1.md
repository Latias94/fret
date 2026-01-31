# ADR 1160: Text Overflow (Ellipsis) and Line Clamp (v1)

Status: Accepted

## Context

Editor-grade UI surfaces rely heavily on deterministic text truncation:

- tabs and breadcrumbs (single-line, ellipsis)
- table cells and list rows (single-line, ellipsis)
- descriptions/snippets (wrapped, sometimes clamped to N lines)

Fret currently exposes `TextOverflow` and `TextWrap` in the core text constraints surface. However,
**multiline + ellipsis** (often called “line clamp”) is a distinct feature from single-line
ellipsis: it requires explicit `max_lines` semantics, deterministic truncation rules, and stable
caret/hit-test behavior at the truncation boundary.

If the framework treats “wrap + ellipsis” as an implicit line clamp, it risks:

- measurement vs paint divergence (parents reserve the wrong height → overlap)
- unstable geometry queries (`hit_test_point`, caret/selection rects)
- inconsistent UX across platforms/renderers

## Decision

### D1 — `TextOverflow::Ellipsis` is **single-line only** in v1

In v1, `TextOverflow::Ellipsis` is defined only when:

- `TextWrap::None` (no soft wrapping), and
- `max_width` is set (or otherwise a definite single-line constraint exists).

If `TextWrap != None`, `TextOverflow::Ellipsis` must **behave as `TextOverflow::Clip`** (no
ellipsis is rendered). Implementations may emit a debug-only diagnostic to encourage explicit line
clamp usage once available.

This matches the “separate feature” model seen in web/CSS (`text-overflow: ellipsis` vs `line-clamp`)
and avoids silently creating a hard-to-change multiline contract without an explicit API.

### D2 — Multiline clamping is a separate, explicit feature (future)

When the framework needs “wrap but clamp to N lines with ellipsis”, it must be represented as an
explicit contract that includes:

- `max_lines: u16` (or similar)
- ellipsis string policy (default `…`, or custom)
- geometry query rules at the truncation boundary:
  - caret/hit-test must not “land” inside elided content
  - selection rects should only include visible glyph runs

This ADR does not define the final API shape for line clamp, but it **forbids** treating
`TextOverflow::Ellipsis` + `TextWrap != None` as implicit multiline clamp in v1.

## Consequences

- UI kit “ellipsis” helpers must continue to pair `TextWrap::None` with `TextOverflow::Ellipsis`.
- Multiline truncation remains out of scope for v1 unless we add a dedicated `line_clamp` surface.
- Renderers can keep `TextOverflow` handling simple and deterministic:
  - ellipsis path only for single-line layout
  - multiline always uses clip unless/until line-clamp is implemented

## References

- Workstream: `docs/workstreams/text-layout-integration-v1.md` (Failure mode F2).
- Geometry queries: `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`,
  `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`.
- Text system v2 direction: `docs/adr/0157-text-system-v2-parley-attributed-spans-and-quality-baseline.md`.

