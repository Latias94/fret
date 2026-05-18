# ViewCache Resize-Jitter Attribution v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane is closed as a no-runtime-change attribution lane. No runtime code was changed.

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

## Closeout

VCRJ-030 captured fresh UI Gallery code-editor resize-jitter evidence:

- Bundle:
  `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/1779091052963/bundle.schema2.json`
- Layout summary:
  `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/layout.perf.summary.v1.json`

Result:

- Worst frame: `total=362814us`, `layout=1070us`, `prepaint=1349us`, `paint=360395us`.
- Top layout hotspots: `Scroll layout_us=224`, `ViewCache layout_us=184`,
  `Flex layout_us=96`.
- Dedicated view-cache phase: `layout_view_cache_time_us=33`.
- `view_cache_roots_reused=1`, `view_cache_contained_relayouts=0`,
  `view_cache_roots_layout_invalidated=0`.
- First clean-geometry skip: `Text/text_reflow`.

Conclusion:

- Do not add `ViewCache` to the clean-geometry allowlist from this evidence.
- Do not optimize contained view-cache relayout from this evidence.
- Continue in
  `docs/workstreams/ui-gallery-code-editor-canvas-paint-tail-attribution-v1/`.

## Guardrails

- Keep `ViewCache` as a clean-geometry side-effect boundary unless a future focused proof proves a
  narrower safe case.
- Keep `Scroll` as a separate possible follow-on.
- Keep the next lane focused on the current `Canvas` paint tail first.
