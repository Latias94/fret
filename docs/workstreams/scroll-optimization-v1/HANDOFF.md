# Scroll Optimization v1 Handoff

Date: 2026-05-21
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

- Post-push baseline:
  `target/fret-diag/local-post-push-clean-geometry-closeout-20260518-r1/1779072457079-ui-gallery-code-editor-window-resize-drag-jitter-steady/bundle.schema2.json`
- Post-push stats:
  `target/fret-diag/local-post-push-clean-geometry-closeout-20260518-r1/worst.stats.json`
- Phase-closeout reference:
  `target/fret-diag/local-next-gallery-header-stretch-clean-geometry-20260518-r2/1779069580027/bundle.schema2.json`
- Phase-closeout stats:
  `target/fret-diag/local-next-gallery-header-stretch-clean-geometry-20260518-r2/worst.stats.json`

Post-push result:

- p95/max total/layout/layout-roots/solve/prepaint/paint/text-prepare:
  `3117/2538/2401/221/267/400/71us`.
- Remaining clean-geometry rejections are still `text_reflow / Text`, `unsupported_kind / Canvas`,
  and `side_effect_boundary / Scroll`.
- View-cache and code-editor guardrails stay stable: cache root reused `1`, rows replayed/stored
  `289/0`, row replay hit rate `100%`.

## Remaining Follow-Ons

Use the existing narrow lane for text, and open new narrow lanes for the remaining items:

- Text proof: `docs/workstreams/text-clean-geometry-stability-v1/` is now the closed boundary
  record for cached `TextWrap::None` eligibility and text rejection diagnostics. Start a new
  behavior-changing line-break / computed-box lane only after fresh evidence shows wrapped text is a
  material perf owner, and do not start from shadcn `CardDescription` authoring.
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
