# M0 Baseline Audit - 2026-05-13

Status: baseline recorded; first code slice is boundary diagnostics for the code-editor content root.

Status note (2026-05-14): the source inventory remains valid historical baseline evidence. The ADR
review gap mentioned below is now closed by `M0_CONTRACT_FREEZE_2026-05-14.md`, which accepts ADR
0327 as the target contract while keeping global implementation work active.

Related:

- `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md`
- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## Scope

This audit records the current runtime path before Frame Pipeline v2 code migration starts.

It focuses on the post-contained-layout state proven by the 2026-05-13 macOS
`ui-code-editor-resize-probes` run:

- layout solve has been reduced enough that it is no longer the dominant bottleneck,
- `paint.widget` and code-editor row replay/content resolution are now the first vertical slice,
- renderer prepare/encode/upload counters are not the active bottleneck for this surface,
- and old cache/containment paths must now be treated as migration candidates, not permanent
  architecture.

Linux-specific closure, Hotpath adoption, and public compatibility for private runtime shims are out
of scope for this audit.

## Assumptions-first Read

### 1) ADR 0327 is the correct target contract for this lane

- Evidence:
  - `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md`
  - `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/DESIGN.md`
  - `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- Confidence:
  - Likely
- Consequence if wrong:
  - the first implementation slice must revise or supersede ADR 0327 before broad migration, but
    the source inventory below remains useful because it names the old paths that any replacement
    contract must cover.

### 2) The current performance problem is structural, not a single local slow function

- Evidence:
  - `target/fret-diag-code-editor-resize-probes-contained-layout-20260513/check.perf_thresholds.json`
  - `target/fret-diag-code-editor-resize-probes-contained-layout-20260513/1778661520873/bundle.schema2.json`
  - `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the first slice could over-invest in boundary diagnostics; the perf gate will catch this because
    `paint.widget`, total p95/max, and code-editor paint counters must improve or clearly explain
    why they did not.

### 3) The runtime already has boundary-like pieces, but no single boundary owner

- Evidence:
  - `crates/fret-ui/src/element.rs:1386`
  - `crates/fret-ui/src/elements/cx.rs:1444`
  - `crates/fret-ui/src/tree/mod.rs:359`
  - `crates/fret-ui/src/tree/ui_tree_view_cache.rs:89`
  - `crates/fret-ui/src/tree/paint_cache.rs:89`
- Confidence:
  - Confident
- Consequence if wrong:
  - a smaller rename/consolidation might be enough; however, the source map shows layout,
    prepaint, paint, liveness, and diagnostics are currently owned by different stores and phases.

### 4) Code-editor paint state is already acting like a local prepaint phase

- Evidence:
  - `ecosystem/fret-code-editor/src/editor/state.rs:556`
  - `ecosystem/fret-code-editor/src/editor/mod.rs:1056`
  - `ecosystem/fret-code-editor/src/editor/paint/scene.rs:183`
  - `ecosystem/fret-code-editor/src/editor/syntax.rs:166`
- Confidence:
  - Confident
- Consequence if wrong:
  - moving it to shared prepaint could add churn without reducing paint work. The first slice should
    therefore add boundary attribution before moving ownership.

### 5) The first source change should be diagnostic and ownership-shaping, not deletion

- Evidence:
  - ADR 0327 requires reuse/reject attribution and deletion notes for each slice.
  - Current `UiTree` diagnostics expose useful cache/prepaint/paint counters, but not stable
    boundary-level reasons.
  - The code-editor surface needs a before/after proof for `paint.widget` or total p95/max.
- Confidence:
  - Likely
- Consequence if wrong:
  - if a direct code-editor replay optimization is obvious during implementation, it can be landed
    first only if it still leaves correctness, perf, worst-bundle, layering, and deletion evidence.

## Current Frame Pipeline (Observed)

Current high-level execution path:

```text
app/model/event invalidation
  -> declarative build through ElementContext
  -> mount into retained UiTree
  -> view-cache reuse or closure execution
  -> layout invalidation propagation / contained relayout
  -> prepaint outputs and interaction-cache work
  -> paint widget work / paint-cache replay / code-editor row-scene replay
  -> renderer prepare / encode / upload / present
```

This path is workable, but the phase boundaries are implicit:

- `view_cache(...)` decides whether the authoring closure runs during declarative build.
- `UiTree` owns retained layout, invalidation, paint cache, interaction cache, and debug counters.
- `contained_layout` is represented as a view-cache flag and is checked by reuse, invalidation, and
  layout code.
- prepaint outputs are node-local typed values, not boundary-owned phase state.
- paint cache stores op ranges from the previous full `Scene`, not boundary-owned scene fragments.
- the code editor prepares visible-window and overlay state through paint hooks.

## Current Boundary-like Concepts

### View-cache root

Source anchors:

- `crates/fret-ui/src/element.rs:1386`
- `crates/fret-ui/src/elements/cx.rs:1444`
- `crates/fret-ui/src/tree/ui_tree_view_cache.rs:63`
- `crates/fret-ui/src/tree/ui_tree_view_cache.rs:89`

Current behavior:

- `ViewCacheProps` carries `layout`, `contained_layout`, and `cache_key`.
- `ElementContext::view_cache(...)` computes a key from theme revision, scale factor, explicit key,
  environment deps, and layout-query deps.
- reuse skips closure execution and touches recorded state/model/global/environment/layout-query
  dependencies.
- `should_reuse_view_cache_node(...)` can reuse across layout invalidation only when
  `contained_layout`, `layout_definite`, and known bounds are all true.

Frame Pipeline v2 implication:

- this is the closest existing identity/build boundary, but it is still cache-centric. It should
  become or feed `ViewBoundary` identity, dependency keys, and build dirty state.

### Contained layout root

Source anchors:

- `crates/fret-ui/src/tree/ui_tree_view_cache.rs:108`
- `crates/fret-ui/src/tree/ui_tree_subtree_layout_dirty.rs:70`
- `crates/fret-ui/src/tree/ui_tree_subtree_layout_dirty.rs:133`
- `crates/fret-ui/src/tree/ui_tree_invalidation_walk/mark.rs:164`
- `crates/fret-ui/src/tree/layout/entrypoints.rs:1543`

Current behavior:

- containment is a boolean on `ViewCacheFlags`.
- dirty roots are tracked in `dirty_cache_roots`.
- subtree dirty aggregation can decide that a parent node's dirty descendants are covered by
  contained view-cache roots.
- invalidation walks repeatedly special-case `contained_layout`.

Frame Pipeline v2 implication:

- this should migrate to boundary layout dependency metadata. A boundary must explain whether parent
  constraints changed, whether intrinsic content changed, and whether a relayout can stay local.

### Runtime liveness and dependency store

Source anchors:

- `crates/fret-ui/src/elements/runtime.rs:421`
- `crates/fret-ui/src/elements/runtime.rs:685`
- `crates/fret-ui/src/elements/runtime.rs:1178`
- `crates/fret-ui/src/elements/runtime.rs:1193`

Current behavior:

- `WindowElementState` keeps rendered/next maps for view-cache state keys, authoring identities,
  elements, observed models, globals, layout queries, and environment queries.
- `begin_view_cache_scope(...)` and `end_view_cache_scope(...)` maintain a stack for nested cache
  roots.
- cache-hit frames touch recorded dependencies so skipped closures keep liveness.

Frame Pipeline v2 implication:

- these rendered/next stores should be consolidated behind boundary identity/dependency state
  instead of remaining a separate declarative-runtime cache sidecar.

### Prepaint output owner

Source anchors:

- `crates/fret-ui/src/widget.rs:916`
- `crates/fret-ui/src/tree/ui_tree_invalidation.rs:133`
- `crates/fret-ui/src/tree/prepaint/mod.rs:36`
- `crates/fret-ui/src/tree/prepaint/interaction.rs:74`

Current behavior:

- `PrepaintCx` runs after layout and can set typed node outputs.
- prepaint outputs are keyed by a paint-cache-style key at the node level.
- interaction-cache replay also uses paint-cache keys.
- debug output records prepaint actions, but the owner is still a node, not a boundary.

Frame Pipeline v2 implication:

- prepaint should become the canonical place for geometry-derived state, including visible windows,
  overlay anchors, hitbox inputs, and resource touch plans.

### Paint cache root and replay source

Source anchors:

- `crates/fret-ui/src/tree/paint_cache.rs:4`
- `crates/fret-ui/src/tree/paint_cache.rs:69`
- `crates/fret-ui/src/tree/paint_cache.rs:89`
- `crates/fret-ui/src/tree/ui_tree_view_cache.rs:44`
- `crates/fret-ui/src/tree/paint/node.rs:54`

Current behavior:

- paint-cache keys include size, scale factor, theme revision, foreground/text style fingerprints,
  and child transform bits.
- entries store previous-frame op start/end indexes and origin.
- `UiTree::ingest_paint_cache_source(...)` destructively swaps previous `Scene` storage into the
  tree before the next frame.
- debug counters record paint-cache replays by node.

Frame Pipeline v2 implication:

- replay should converge on boundary-owned `SceneFragment` state with side indexes such as text
  blobs/resources and explicit reuse/reject reasons.

### Code-editor local frame and row replay state

Source anchors:

- `ecosystem/fret-code-editor/src/editor/state.rs:556`
- `ecosystem/fret-code-editor/src/editor/mod.rs:1056`
- `ecosystem/fret-code-editor/src/editor/mod.rs:2015`
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs:183`
- `ecosystem/fret-code-editor/src/editor/paint/rich.rs:1`
- `ecosystem/fret-code-editor/src/editor/syntax.rs:166`

Current behavior:

- `begin_paint_frame(...)` computes visible row window, cache floor, paint-perf frame state,
  selection/caret overlay projection, and overlay prepare timing.
- row paint calls `paint::paint_row(...)` once per visible row.
- row scene replay compares full or fast syntax keys, touches hosted resources, and replays scene
  ops translated with text blob ids.
- syntax/rich caches use the visible row/window as an implicit prepaint input.

Frame Pipeline v2 implication:

- this is the first ecosystem-side vertical slice. The runtime should own the boundary phase and
  attribution, while editor policy/data structures remain in `ecosystem/fret-code-editor`.

## Mismatch Against ADR 0327

- There is no unified `BoundaryId` that owns build, layout, prepaint, paint, hit-test, semantics,
  renderer-resource dirty state, and diagnostics together.
- `contained_layout` is a flag checked in several subsystems, not a boundary dependency contract.
- cache identity and liveness are split between declarative runtime maps and retained `UiTree`
  flags.
- prepaint exists, but it is node-local and not yet the general owner for editor visible windows,
  overlay anchors, or resource touch plans.
- paint replay is tree-global previous-scene op-range replay, not boundary-owned scene-fragment
  replay.
- code-editor row scene replay is an ecosystem-local cache with useful mechanics, but it is not
  yet integrated into boundary-level phase attribution.
- diagnostics are phase-rich enough to guide performance work, but they do not yet answer
  per-boundary reuse/reject questions.

## Old-path Migration Candidates

| Current path | Source anchor | Target owner | First migration move |
| --- | --- | --- | --- |
| `ViewCacheProps::contained_layout` | `crates/fret-ui/src/element.rs:1391` | boundary layout dependencies | add boundary diagnostics that report containment reason and dependency class |
| `ViewCacheFlags` and `should_reuse_view_cache_node` | `crates/fret-ui/src/tree/ui_tree_view_cache.rs:89` | boundary build/layout dirty state | split reuse reason into explicit debug enum before changing behavior |
| `dirty_cache_roots` | `crates/fret-ui/src/tree/mod.rs:365` | boundary dirty set | mirror the existing dirty roots into boundary diagnostics, then migrate writes |
| view-cache rendered/next maps | `crates/fret-ui/src/elements/runtime.rs:421` | boundary identity/dependency store | introduce boundary identity metadata backed by current maps |
| prepaint typed node outputs | `crates/fret-ui/src/tree/ui_tree_invalidation.rs:133` | boundary prepaint state | add owner attribution and dependency key reporting |
| paint-cache previous op ranges | `crates/fret-ui/src/tree/paint_cache.rs:69` | boundary `SceneFragment` | add fragment-shaped debug output before moving storage |
| `UiTree::ingest_paint_cache_source` | `crates/fret-ui/src/tree/ui_tree_view_cache.rs:44` | boundary scene-fragment ingestion | keep `Scene` contract, but narrow ingestion to fragment owners |
| code-editor `begin_paint_frame` | `ecosystem/fret-code-editor/src/editor/state.rs:556` | boundary prepaint phase plus editor-owned payload | first expose it as boundary prepaint-like work in diagnostics |
| row scene replay | `ecosystem/fret-code-editor/src/editor/paint/scene.rs:183` | editor scene fragments under runtime boundary attribution | preserve editor cache policy, move attribution and replay plan into prepaint/paint phases |
| UI Gallery page containment policy | `apps/fret-ui-gallery/src/spec.rs:867` | exemplar boundary hints | retain only as first-slice authoring hint until boundary metadata replaces it |

## Delete or Narrow Candidates

These should not be deleted in M0. They become deletion targets after the replacement path has
correctness and perf evidence.

- `ViewCacheProps::contained_layout` as a standalone runtime decision.
- `dirty_cache_roots` as a cache-specific dirty set once boundary dirty sets cover the same cases.
- Duplicate view-cache and paint-cache debug counters once boundary diagnostics report reuse/reject
  by phase.
- Private paint-cache env knobs after boundary diagnostics cover their original experiments:
  - `FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING`
  - `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY`
- Private layout subtree/sweep knobs after boundary dirty propagation replaces their diagnostic
  purpose:
  - `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION`
  - `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE`
  - `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE_PANIC`
  - `FRET_UI_LAYOUT_ENGINE_SWEEP`
- Code-editor paint-hook ownership for visible-window/overlay projection after boundary prepaint
  state owns the phase entry.
- UI Gallery's historical page-id-specific `page_content_cache_contained_layout(...)`; this was
  replaced by `page_content_cache_contain_layout_when_bounds_known(...)` in
  `M4C_BOUNDARY_HINT_API_SLICE_2026-05-14.md`, with remaining cleanup focused on internal runtime
  flags.

## First Slice Proposal

Start with boundary diagnostics, not behavior change.

Required first-slice outputs:

1. Add a narrow internal boundary/reuse diagnostic record for the UI Gallery code-editor content
   root.
2. Record current reuse/reject reason for build/view-cache reuse, layout containment, prepaint
   output availability, and paint-cache/row-scene replay participation.
3. Export the record through existing diagnostics snapshots without moving policy into
   `crates/fret-ui`.
4. Gate with the existing `ui-code-editor-resize-probes` command and worst-bundle attribution.
5. Use the resulting evidence to choose whether the next code slice should move code-editor
   visible-window state to prepaint or start with scene-fragment replay.

This ordering is intentional: it makes the old path measurable before deletion, and it keeps the
first implementation small enough to review.

## Gate Set for First Code Slice

Correctness and layering:

```bash
cargo nextest run -p fret-ui view_cache
cargo nextest run -p fret-ui prepaint
python3 tools/check_layering.py
```

Perf and attribution:

```bash
cargo run -p fretboard-dev --release -- diag perf ui-code-editor-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

```bash
target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 15
```

Every perf claim must still report:

- total/layout/prepaint/paint p50 and p95,
- `layout.engine_solve`,
- `paint.widget`,
- `paint.text_prepare`,
- renderer prepare/encode/upload counters,
- `code_editor.paint_perf`,
- and boundary-level reuse/reject attribution once the first diagnostic slice exists.

## Baseline Verdict

Proceed with the Frame Pipeline v2 lane.

The current implementation has enough targeted optimizations to show the correct direction, but its
execution model is still too indirect for the next 20-30% improvement. The next code change should
make the code-editor content root boundary-attributable first, then move state and replay ownership
one phase at a time:

1. boundary diagnostics,
2. prepaint ownership for visible-window and editor frame-derived state,
3. scene-fragment replay attribution and storage,
4. layout containment dependency metadata,
5. deletion audit and removal of replaced private paths.

M0 source inventory is complete here. The later contract-freeze note
`M0_CONTRACT_FREEZE_2026-05-14.md` closes the ADR acceptance gap.
