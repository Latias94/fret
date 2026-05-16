# Code Editor Row Fragment Replay Contract v1

Status: Active
Date: 2026-05-16

## Why This Lane Exists

Local editor-paint attribution after inline-preedit replay recovery moved the remaining code-editor
owner to row-scene prepaint planning:

- `code_editor_paint_perf.p95.us_row_scene_prepaint_plan`: `95us`
- `code_editor_paint_perf.p95.us_row_scene_prepaint_probe`: `77us`
- `code_editor_paint_perf.p95.us_row_scene_prepaint_key_compare`: `7us`
- `renderer_prepare_text_us` stayed below the row-scene owner on the same probe.

A smaller experiment that used one mutable cache lookup and preallocated plan entries did not
materially reduce the owner. The next change is therefore structural: stop paying per-row
HashMap-probe and replay-plan assembly cost in the hot prepaint loop when a contiguous visible row
fragment can be represented and validated at a coarser boundary.

## Scope

Owned by this lane:

- code-editor row-scene replay plan shape and consumption,
- ViewBoundary scene-fragment carrier usage for code-editor row fragments,
- diagnostics that prove planned/used/rejected row-fragment entries,
- perf evidence for the local complex-wheel editor paint probe.

Out of scope:

- renderer text/glyph/atlas residency,
- broad Canvas display-list replacement,
- VirtualList semantics,
- text shaping cache semantics,
- formal Windows RTX4090 closeout.

The Windows closeout remains owned by `ui-perf-zed-smoothness-v1`; this lane may produce local
baseline-neutral evidence only.

## Current Contract

Today prepaint builds a `RowSceneReplayPlan` by iterating visible row rects, probing
`CodeEditorState::row_scene_cache` by row, validating the cached replay key, and pushing one
`CanvasSceneFragment<RowSceneFragmentPayload>` per row. Paint consumes that plan row-by-row inside
`paint_row(...)`.

This is correct and debuggable, but it makes a steady retained frame pay O(visible rows) cache probe
and plan assembly cost before paint can replay the same retained rows.

## Target Contract

Represent a contiguous row-scene replay fragment as a boundary-owned scene fragment with enough
metadata for paint to answer:

- Does this fragment cover the current `WindowedRowsPaintFrame` visible range and row rect shape?
- Which rows are fully covered by base row replay?
- Which rows must still run paint-time overlay/preedit work?
- How many retained entries were used or rejected, and why?

The first implementation should preserve the existing row-level fallback. A bad or stale fragment
must degrade to the current per-row paint path without changing selection, caret, preedit, hit-test,
or hosted-resource lifetime behavior.

## M1 Decision

The first shipped shape is Candidate A in a conservative form: a replay-plan entry points at a
retained per-row scene fragment via `Arc<RowSceneRetainedFragment>` and carries only the current
frame's row/local-bounds metadata. This keeps the existing row-level fallback and avoids cloning the
full row content, geometry, scene ops, and hosted-resource list during prepaint plan assembly.

Candidate B, a precomposed visible-window scene fragment, remains a later optimization. It would
need a stronger overlay/preedit representation and coarser invalidation diagnostics before it is
worth the additional contract surface.

Overlay and preedit behavior:

- inline preedit keeps the caret/preedit row on the paint-time path;
- planned base row replay can still be used for selected rows, but paint continues past base replay
  to draw overlays;
- no-overlay planned rows return immediately after replay and only clone geometry if the row-geom
  cache needs a fill.

Hosted-resource behavior stays unchanged: retained text blobs, paths, and SVGs are touched before
replaying retained scene ops.

## Assumptions

- Confident: Renderer text prepare is not the current local owner. Evidence:
  `target/fret-diag/local-next-editor-paint-20260516-prepaint-probe-attrib-complex-wheel-r3/worst.stats.json`.
  If wrong, the lane should pause and reopen renderer text attribution.
- Confident: Key comparison is not the local owner. Evidence: p95 key compare is `7us`.
  If wrong, a key-fingerprint slice would be smaller than this lane.
- Likely: A coarse row-fragment contract needs boundary state, not another editor-local cache.
  Evidence: `M3C_BOUNDARY_SCENE_FRAGMENT_CARRIER_SLICE_2026-05-14.md` already moved the carrier to
  ViewBoundary scene-fragment state.
- Unclear: The best first shape is either a contiguous run descriptor over existing per-row cached
  entries, or a precomposed scene-op fragment for the entire visible window. The first milestone must
  answer this with a small prototype and measured fallback behavior.

## Exit Criteria

This lane can close when:

- the complex-wheel local p95 `us_row_scene_prepaint_probe` moves materially below `77us`,
- planned/used/rejected fragment diagnostics explain every fallback,
- focused row replay tests pass,
- local perf evidence is recorded,
- and `ui-perf-zed-smoothness-v1` either consumes the result or records why the lane did not beat the
  transitional row-plan path.
