# Text Clean Geometry Stability v1

Date: 2026-05-21
Status: Closed boundary record split from `scroll-optimization-v1`.

Closeout verdict (2026-05-21): this lane decides and proves the current text clean-geometry
boundary. It does not expand wrapped-text eligibility. Future wrapped-text clean geometry requires
fresh material perf evidence plus a dedicated line-break/computed-box signature lane.

## Problem

The local clean-geometry resize-jitter phase in `scroll-optimization-v1` closed with one intentional
text stop condition still visible in the UI Gallery preview card: `Semantics -> Text` rejects clean
geometry as `text_reflow / Text`. That path comes from shadcn `CardDescription`, which correctly
authors description text as full-width `TextWrap::Word`.

The remaining question is narrower than scroll optimization:

- which text boxes are stable enough to propagate clean geometry without authoritative layout,
- what evidence proves a text node's computed box and paint output are unchanged,
- and which wrapped or width-derived text must stay on the authoritative solve path.

## Current Boundary

Current `fret-ui` clean geometry only accepts text through cached nowrap metrics:

- element kind: `Text`, `StyledText`, or `SelectableText`,
- `wrap == TextWrap::None`,
- `overflow == TextOverflow::Clip`,
- `align == TextAlign::Start`,
- height unchanged,
- cached wrap-none measure fingerprint unchanged,
- cached measured size matches the propagated bounds.

Anything outside that proof rejects as `text_reflow`.

This is deliberate. A wrapped full-width text node can change line breaks, measured height, glyph
positions, selection geometry, and paint output when parent width changes. It is not a safe
clean-geometry candidate unless the line-break signature and computed box are explicitly proven
unchanged.

## Eligibility Matrix

This matrix describes the current width-delta clean-geometry proof for text. The trivial same-size
early return in `clean_nowrap_text_cached_metrics_supported` is not evidence that wrapped text is
width-stable; it only means there is no text box size delta to prove for that helper call.

| Case | Elements | Required facts | Result |
| --- | --- | --- | --- |
| Stable nowrap text | `Text`, `StyledText`, `SelectableText` | `TextWrap::None`, `TextOverflow::Clip`, `TextAlign::Start`, unchanged height, existing wrap-none measure cache, cached size equals propagated bounds, recomputed fingerprint equals cached fingerprint | clean geometry may propagate |
| Wrapped text | `Text`, `StyledText`, `SelectableText` | any `wrap != TextWrap::None`, including `TextWrap::Word` | reject as `text_reflow` |
| Non-clip overflow | `Text`, `StyledText`, `SelectableText` | any `overflow != TextOverflow::Clip`, including ellipsis | reject as `text_reflow` |
| Non-start alignment | `Text`, `StyledText`, `SelectableText` | any `align != TextAlign::Start` | reject as `text_reflow` |
| Height-changing text | `Text`, `StyledText`, `SelectableText` | propagated height differs from previous height by more than `0.01px` | reject as `text_reflow` |
| Missing cached metrics | `Text`, `StyledText`, `SelectableText` | no `text_wrap_none_measure_cache` recorded for the node | reject as `text_reflow` |
| Cached-size mismatch | `Text`, `StyledText`, `SelectableText` | cached measured size does not match the propagated bounds size | reject as `text_reflow` |
| Stale fingerprint | `Text`, `StyledText`, `SelectableText` | content pointer/length, rich spans, resolved text style, font stack, scale factor, overflow, or alignment changed since the cached measure | reject as `text_reflow` |

Measure-side cache rules are intentionally narrower than "was measured before":

- `measure_text`, `measure_styled_text`, and `measure_selectable_text` only keep the cache while
  `wrap == TextWrap::None`.
- wrapped text clears the cache.
- ellipsis can write a height-only cache for measurement reuse, but clean geometry still rejects it
  because the runtime fast path requires `TextOverflow::Clip`.

## Non-Goals

- Do not change `CardDescription` to `TextWrap::None` to satisfy a perf gate.
- Do not special-case shadcn recipes by element name.
- Do not allow `TextWrap::Word` clean geometry by assuming a small resize keeps line breaks stable.
- Do not move text policy into `fret-imui`; this lane belongs to `fret-ui` layout contracts and
  text evidence.

## Design Direction

Keep two proof classes separate:

1. Stable computed-box text:
   - nowrap text with cached metrics and unchanged height,
   - already covered by focused tests,
   - allowed to propagate clean geometry.
2. Line-break-stable wrapped text:
   - future work only,
   - needs an explicit line-break or layout-fragment signature,
   - must prove unchanged measured size, line count, glyph cluster positions, and paint-relevant
     metrics before it can skip authoritative layout.

The first useful slice is therefore a boundary lock, not a behavior expansion. It prevents future
performance work from broadening text eligibility without naming the required proof.

## Evidence Anchors

- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
- `ecosystem/fret-ui-shadcn/src/card.rs`
- `apps/fret-ui-gallery/src/ui/content.rs`
- `docs/workstreams/scroll-optimization-v1/HANDOFF.md`
- `docs/workstreams/text-clean-geometry-stability-v1/CLOSEOUT_AUDIT_2026-05-21.md`
