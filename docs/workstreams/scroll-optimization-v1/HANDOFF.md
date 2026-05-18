# Scroll Optimization v1 Handoff

Date: 2026-05-18
Status: Active umbrella workstream; local clean-geometry resize-jitter phase closed.

## Current Verdict

Do not close the entire `scroll-optimization-v1` workstream. Its original scope still covers scroll
correctness, wheel coalescing, scrollbar drag baseline, and extent-probing contracts.

Do close the local no-4090 clean-geometry resize-jitter phase. The remaining blockers are now
classified as stop conditions or separate follow-ons, not unfinished local mechanism work.

## Closed Local Phase

The local resize-jitter clean-geometry phase closed after:

- retained/windowed scroll updates stopped forcing parent view-cache rerenders,
- clean root-solve propagation and per-solve rejection attribution landed,
- Container, stable auto-height wrappers, horizontal Flex subsets, card-header-like Grid,
  absolute overlay chrome, ViewCache boundary propagation, explicit zero driver leaves, and gallery
  content-header authoring were proven with focused gates,
- wrapped `TextWrap::Word` text was documented as an authoritative-solve stop condition.

Most recent evidence:

- `target/fret-diag/local-next-gallery-header-stretch-clean-geometry-20260518-r2/1779069580027/bundle.schema2.json`
- `target/fret-diag/local-next-gallery-header-stretch-clean-geometry-20260518-r2/worst.stats.json`

## Remaining Follow-Ons

Open a new narrow lane for any of these:

- Text proof: line-break / computed-box stability, likely starting from `TextWrap::None` or a
  dedicated cached line-break contract, not from shadcn `CardDescription`.
- Canvas proof: only if fresh evidence makes the `PointerRegion -> Canvas` solve worth the risk;
  current local evidence is about `3-4us`.
- Root `Scroll` redesign: only as a side-effect-boundary architecture lane; do not skip `Scroll`
  layout by name.
- RTX4090 or other-machine evidence: collect as closeout/perf-baseline calibration, not as a local
  completion blocker.
- `measured_size: Option<Size>` migration: only if legal zero-size ambiguity recurs outside the
  already-explicit driver leaves.

## Gates To Reconfirm Before Reopening Clean Geometry

- `cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_auto_height_text_reflow --no-fail-fast`
- `python3 -m json.tool docs/workstreams/scroll-optimization-v1/WORKSTREAM.json`
- `python3 tools/check_layering.py`
- `git diff --check`
