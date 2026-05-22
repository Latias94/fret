# Text Clean Geometry Stability v1 Handoff

Date: 2026-05-21
Status: Closed boundary record.

## Current State

This lane was split from `scroll-optimization-v1` after the local clean-geometry resize-jitter phase
closed. The remaining text rejection is not a bug in shadcn authoring: `CardDescription` is
full-width wrapped description text, and wrapped text remains width-derived until a dedicated
line-break/computed-box proof exists.

Current accepted fast path:

- `Text`, `StyledText`, or `SelectableText`,
- `TextWrap::None`,
- `TextOverflow::Clip`,
- `TextAlign::Start`,
- unchanged height,
- cached wrap-none measure fingerprint and measured size still match.

Current rejected paths:

- `TextWrap::Word` / wrapped text,
- height-changing text,
- missing or stale text measure cache,
- changed text style, font-stack, overflow, alignment, scale, or content fingerprint.

## Next Action

Do not continue implementation in this lane by default. T1/T2/T3/T4 are closed for the boundary
record: the lane has a compact eligibility matrix, diagnostics carry optional `detail`, no behavior
change landed, and future wrapped-text work is a separate proof lane. Start a new behavior-changing
follow-on only after fresh evidence shows wrapped text is a material perf owner. Do not widen
wrapped-text eligibility until that follow-on has a real line-break/computed-box proof.

Latest artifact evidence:

- `target/fret-diag/text-clean-geometry-detail-20260521-r1/sessions/1779305958600-170448/1779306373288/bundle.schema2.json`
- `target/fret-diag/text-clean-geometry-detail-20260521-r1/sessions/1779305958600-170448/share/1779306373288.zip`
- `target/fret-diag/text-clean-geometry-detail-20260521-r1/sessions/1779305958600-170448/1779306373288/ai.packet`

The UI Gallery text-measure-overlay resize script passed and the bundle contains
`clean_geometry_solve_skip_rejection.detail` entries for `text_wrap_not_none` and
`text_overflow_not_clip`. Treat this as T2 diagnostics proof only, not as a T3/T4 wrapped-text
eligibility or perf-closeout result.

## Fast Revalidation

```bash
cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_auto_height_text_reflow clean_geometry_small_resize_skips_nowrap_text_width_delta_when_height_stable clean_geometry_small_resize_rejects_nowrap_text_height_delta --no-fail-fast
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics clean_geometry_rejection_detail_is_additive --no-fail-fast
cargo fmt -p fret-ui -p fret-bootstrap --check
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/text-wrap/ui-gallery-text-measure-overlay-window-resize-drag-jitter-steady.json --dir target/fret-diag/text-clean-geometry-detail-20260521-r1 --session-auto --pack --ai-packet --launch -- cargo run -p fret-ui-gallery --features gallery-dev
python -m json.tool docs/workstreams/text-clean-geometry-stability-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```
