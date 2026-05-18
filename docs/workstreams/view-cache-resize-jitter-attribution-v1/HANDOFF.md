# ViewCache Resize-Jitter Attribution v1 - Handoff

Status: Active
Last updated: 2026-05-18

## Current State

The lane is open and scoped as a narrow follow-on after `pressable-clean-geometry-propagation-v1`.
No runtime code has been changed.

Starting evidence:

- `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/1779088062238/bundle.schema2.json`
- `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/layout.perf.summary.v1.json`

Starting hotspot verdict:

- `ViewCache layout_us=380 inclusive_us=723`
- `Scroll layout_us=205 inclusive_us=331`
- `Flex layout_us=83 inclusive_us=122`

VCRJ-020 is complete:

- `docs/workstreams/view-cache-resize-jitter-attribution-v1/VCRJ_020_SOURCE_ATTRIBUTION_2026-05-18.md`

Current verdict:

- The starting `ViewCache` hotspot is recorded in the main `layout_roots` pass, not in contained
  view-cache relayout.
- The dedicated view-cache phase is small in the starting bundle (`layout_view_cache_time_us` about
  `29-30us`).
- `view_cache_roots_reused=1`, `view_cache_contained_relayouts=0`, and
  `view_cache_roots_layout_invalidated=0`.
- The first clean-geometry rejection is `Text/text_reflow`, so a direct `ViewCache` allowlist change
  is not justified by current evidence.

## Next Task

Run VCRJ-030.

Goal:

- Capture a fresh UI Gallery resize-jitter bundle with current code and confirm whether the same
  main-root `ViewCache` hotspot signature repeats.
- Preserve phase stats and clean-geometry rejection fields so VCRJ-040 can choose between a narrow
  `ViewCache` proof, a text-reflow clean-geometry split, or a no-change verdict.

Start with:

```bash
target/release/fretboard-dev diag perf \
  tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json \
  --repeat 1 \
  --warmup-frames 5 \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_RENDERER_PERF=1 \
  --env FRET_LAYOUT_NODE_PROFILE=1 \
  --env FRET_LAYOUT_NODE_PROFILE_TOP=20 \
  --env FRET_LAYOUT_NODE_PROFILE_MIN_US=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --dir target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

## Guardrails

- Do not add `ElementInstance::ViewCache(_)` to the clean-geometry allowlist as a first move.
- Do not optimize `layout_contained_view_cache_roots_if_needed(...)` unless a fresh bundle shows
  `view_cache_contained_relayouts > 0`.
- Keep `Scroll` as a separate possible follow-on.
- Keep UI Gallery recipe changes out unless evidence proves the demo composition owns the cost.
- Preserve cache-root liveness, state retention, boundary tracing, and scroll extent repair.
