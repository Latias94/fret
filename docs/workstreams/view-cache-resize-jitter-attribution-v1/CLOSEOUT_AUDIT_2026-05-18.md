# ViewCache Resize-Jitter Attribution v1 - Closeout Audit

Date: 2026-05-18
Status: Closed

## Verdict

No `ViewCache` runtime change should land from this lane.

The starting evidence made `ViewCache` look like the next retained-layout owner after the
`Pressable` clean-geometry closeout. The fresh VCRJ-030 bundle changed that owner verdict:

- the worst frame is paint-dominated, not layout-dominated;
- `Scroll` is the top layout hotspot in the current layout summary;
- dedicated `ViewCache` layout work is small;
- cache-root reuse is active and healthy;
- contained view-cache relayout does not run;
- the first clean-geometry skip remains `Text/text_reflow`.

## Fresh Evidence

Bundle:

- `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/1779091052963/bundle.schema2.json`

Layout summary:

- `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/layout.perf.summary.v1.json`

Stats command:

```bash
target/release/fretboard-dev diag stats \
  target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/1779091052963/bundle.schema2.json \
  --sort time --top 20
```

Worst-frame split:

```text
total=362814us
layout=1070us
prepaint=1349us
paint=360395us
```

Layout phase fields:

```text
layout_roots_time_us=930
layout_view_cache_time_us=33
layout_repair_view_cache_bounds_time_us=1
layout_contained_view_cache_roots_time_us=0
layout_collapse_layout_observations_time_us=31
```

Cache-root fields:

```text
view_cache_roots_reused=1
view_cache_contained_relayouts=0
view_cache_roots_layout_invalidated=0
view_cache_roots_cache_key_mismatch=0
```

Layout hotspots:

```text
Scroll layout_us=224 inclusive_us=572
ViewCache layout_us=184 inclusive_us=888
Flex layout_us=96 inclusive_us=303
```

Paint hotspot:

```text
Canvas paint_time_us=360009 inclusive_us=360009 scene_ops_delta=20009
```

## Decision

Keep `ElementInstance::ViewCache(_)` classified as a clean-geometry side-effect boundary.

The safe next runtime target is not a `ViewCache` allowlist change and not
`layout_contained_view_cache_roots_if_needed(...)`. If another `ViewCache` optimization appears
later, it needs a new focused proof where `ViewCache` is the top owner and cache-root liveness,
state retention, boundary tracing, and scroll extent repair are explicitly protected.

## Follow-On

Continue with:

- `docs/workstreams/ui-gallery-code-editor-canvas-paint-tail-attribution-v1/`

Keep possible future lanes separate:

- `Scroll` resize layout attribution, if a fresh bundle shows layout is the north-star bottleneck.
- `Text/text_reflow` clean-geometry proof, if the solve skip remains the bottleneck after paint is
  controlled.
- Diagnostics schema cleanup, if the `Canvas` paint tail proves to be an attribution gap rather
  than real work.
