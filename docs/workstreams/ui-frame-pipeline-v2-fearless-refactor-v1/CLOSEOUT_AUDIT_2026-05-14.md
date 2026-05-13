# Closeout Audit - 2026-05-14

Status: code-editor vertical slice complete; broader ADR 0327 lane remains active.

## Scope

This audit closes the current goal's code-editor vertical slice, not the full ADR 0327 program.

ADR 0327 remains `Proposed` and still describes a broader architecture direction: future
build-boundary consolidation, public naming review, wider paint-cache/view-cache unification, and
eventual ADR acceptance or supersession. Those broader items stay in this workstream as follow-on
work and must not be mistaken for missing evidence in this vertical slice.

## Prompt-to-Artifact Checklist

| Requirement | Evidence | Verdict |
| --- | --- | --- |
| Land the minimal `ViewBoundaryState` / `BoundaryId` runtime state. | `crates/fret-ui/src/tree/view_boundary.rs` defines `BoundaryId`, `ViewBoundaryState`, `BoundaryLayoutDependencies`, `BoundaryPrepaintState`, `BoundarySceneFragmentState`, and `BoundaryDirtyState`. `M2B_VIEW_BOUNDARY_PREPAINT_STATE_SLICE_2026-05-13.md` records the first state slice. | Complete for this vertical slice. |
| Move node-scoped `PrepaintOutputs` / `RowSceneReplayPlan` into boundary-owned prepaint/fragment state or delete transitional carriers. | `crates/fret-ui/src/tree/ui_tree_invalidation.rs` routes prepaint outputs and scene fragments through `ViewBoundaryState`; `crates/fret-ui/src/tree/tests/prepaint.rs` proves prepaint output ownership. `ecosystem/fret-code-editor/src/editor/mod.rs` writes row replay plans with `cx.set_scene_fragment_debug(plan)`, and `ecosystem/fret-code-editor/src/editor/paint/mod.rs` consumes them with `painter.scene_fragment_mut::<RowSceneReplayPlan>()`. | Complete. Generic prepaint outputs remain as the shared boundary-owned mechanism for non-fragment prepaint data. |
| Convert layout containment from standalone flag into boundary dependency metadata. | `BoundaryLayoutDependencies::from_view_cache_flags(...)` maps current flags into boundary dependency metadata; `boundary_allows_contained_relayout(...)` is the boundary query used by reuse logic; `debug.boundaries[].layout_dependency` reports the dependency. | Complete for the vertical slice. The public/exemplar `contained_layout` hint remains as an authoring input until a broader boundary-hint API replaces it. |
| Change code-editor row scene replay to boundary-owned scene fragment reuse. | `CanvasSceneFragment<RowSceneFragmentPayload>` is the row replay carrier (`ecosystem/fret-code-editor/src/editor/state.rs`, `paint/scene.rs`); paint records used/rejected fragment entries through `CanvasPainter`. `M3C_BOUNDARY_SCENE_FRAGMENT_CARRIER_SLICE_2026-05-14.md` records the migration. | Complete. |
| Migrate `debug.cache_roots[].boundary` to first-class boundary diagnostics. | `M4B_BOUNDARY_DIAGNOSTICS_CANONICALIZATION_SLICE_2026-05-14.md`; `UiCacheRootStatsV1` no longer has a nested `boundary` field; `debug.boundaries[]` owns build/reuse/layout/paint outcomes; `fret-diag stats` joins cache-root report summaries from top-level boundaries. | Complete. |
| Delete or retire v2-replaced private paths and migration env knobs. | Deleted/retired in-scope paths: node-owned prepaint output storage, code-editor-owned row replay-plan carrier, `dirty_cache_roots` / `dirty_cache_root_reasons` / `mark_cache_root_dirty(...)`, serialized `debug.cache_roots[].boundary`, and `UiBoundaryCacheRootDiagnosticsV1`. `rg` confirms no live `UiBoundaryCacheRootDiagnosticsV1`, `pub boundary:`, or `r.get("boundary")` producer/consumer path remains. | Complete for in-scope replaced paths. No migration-only env knob was introduced by this slice. Existing paint-cache/layout diagnostic env knobs are explicitly out-of-scope retained mechanisms with separate evidence and follow-on ownership. |
| Update ADR 0327 alignment and workstream TODO/MILESTONES/EVIDENCE. | `docs/adr/IMPLEMENTATION_ALIGNMENT.md` row for ADR 0327 updated on 2026-05-14; `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and M4B slice note updated. | Complete. |
| Prove correctness and performance with focused tests, layering, cargo check, perf gate, and worst-bundle stats. | Correctness and check gates are listed below. Perf closeout evidence is in `EVIDENCE_AND_GATES.md` and M4B. Worst bundle stats were rerun with current `fretboard-dev`. | Complete. |
| Reach 20-30% improvement on selected bottleneck. | Selected bottleneck: paint-side `paint.widget` p95 after M1 made paint dominant. M1 evidence: `paint.widget` p95 `1494us`, paint p95 `1737us`. Closeout evidence: `paint.widget` p95 `650us`, paint p95 `875us`. | Complete. Both selected paint-side bottleneck p95 and total p95 exceed the 20-30% improvement target. |
| Keep `fret-ui` mechanism-only and ecosystem policy outside core. | `crates/fret-ui` owns boundary stores, typed prepaint outputs, scene-fragment carrier mechanics, dirty state, and diagnostics. Code-editor validation/replay policy remains in `ecosystem/fret-code-editor`. No shadcn/Radix/component policy moved into `crates/fret-ui`. `python3 tools/check_layering.py` passed. | Complete. |

## Final Gate Evidence

Correctness:

```bash
cargo nextest run -p fret-ui \
  declarative::tests::canvas::canvas_scene_fragment_is_boundary_owned_and_keyed_by_prepaint_key \
  --no-fail-fast
cargo nextest run -p fret-code-editor --features syntax-rust \
  prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint \
  --no-fail-fast
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics \
  cache_root_boundary \
  --no-fail-fast
cargo nextest run -p fret-diag \
  bundle_stats_preserves_cache_root_boundary_summary \
  --no-fail-fast
```

Observed results:

- `fret-ui` scene-fragment test: `1 passed`.
- `fret-code-editor` row-scene replay test: `1 passed`.
- `fret-bootstrap` boundary diagnostics tests: `5 passed, 97 skipped`.
- `fret-diag` stats join test: `1 passed, 818 skipped`.

Compile/layering:

```bash
cargo check -p fret-bootstrap -p fret-diag --features diagnostics
cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-bootstrap --features syntax-rust,diagnostics
python3 tools/check_layering.py
cargo fmt
python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null
git diff --check
```

Observed results:

- All commands passed in the final M4B closeout loop.

Perf:

```bash
target/release/fretboard-dev diag perf ui-code-editor-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --dir target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514 \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Evidence:

- `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/check.perf_thresholds.json`
- `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/1778700500609/bundle.schema2.json`

Observed result:

- gate failures: `[]`,
- total p50/p95/max: `1205/1396/1396us`,
- layout p50/p95/max: `231/320/320us`,
- prepaint p50/p95/max: `243/339/339us`,
- paint p50/p95/max: `710/839/839us`,
- row scene replay hit rate: `99-100%`,
- renderer prepare/encode/upload counters stayed at `0`.

Worst-bundle attribution, rerun with current `fretboard-dev`:

```bash
cargo run -p fretboard-dev --release -- diag stats \
  target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/1778700500609/bundle.schema2.json \
  --sort time \
  --top 15
```

Observed result:

- time sum: total `11285us`, layout `1009us`, prepaint `2905us`, paint `7371us`,
- time p50/p95: total `1151/1396us`, layout `34/337us`, prepaint `255/375us`,
  paint `661/875us`,
- hot p50/p95: `layout.engine_solve=0/132us`, `paint.widget=443/650us`,
  `paint.text_prepare=9/11us`,
- `code_editor.paint_perf` planned/used replay entries: `2090/2090`,
- `code_editor.paint_perf` rows replayed: `2885`,
- `code_editor.paint_perf` p50/p95 `us_row_text`: `0/13us`,
- `code_editor.paint_perf` p50/p95 `us_row_scene_prepaint_plan`: `54/86us`,
- `code_editor.paint_perf` p50/p95 total: `182/385us`.

## Deletion Audit

Deleted or retired during the vertical slice:

- node-owned typed prepaint output storage as the active owner; storage now lives under
  `ViewBoundaryState::prepaint`,
- code-editor-owned `RowSceneReplayPlan` frame carrier; replay plans now use
  `ViewBoundaryState::scene_fragment`,
- generic prepaint-output carrier for row-scene replay; the row-scene path now uses
  `CanvasSceneFragment<RowSceneFragmentPayload>`,
- code-editor-local fixed-row rect reconstruction in replay planning; `WindowedRowsPaintFrame`
  owns row rect geometry,
- `dirty_cache_roots`, `dirty_cache_root_reasons`, and `mark_cache_root_dirty(...)`,
- serialized `debug.cache_roots[].boundary`,
- `UiBoundaryCacheRootDiagnosticsV1`,
- `fret-diag stats` dependence on nested cache-root boundary diagnostics.

Retained intentionally:

- `debug.cache_roots[]`: compatibility/debug view for cache-root-level summaries; boundary outcomes
  now live in `debug.boundaries[]`.
- `top_cache_roots[].boundary` in `fret-diag` report JSON: report-only derived summary sourced
  from `debug.boundaries[]`, not a bundle schema path.
- `ViewCacheProps::contained_layout` and UI Gallery page containment hints: still the authoring
  input for the first boundary slice; runtime now mirrors this into boundary dependency metadata.
- `FRET_UI_GALLERY_VIEW_CACHE` / `FRET_UI_GALLERY_VIEW_CACHE_SHELL`: perf/script setup flags used by
  `fret-diag` to make the selected repro deterministic, not v2 migration compatibility shims.
- `FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING`,
  `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY`,
  `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION*`, and `FRET_UI_LAYOUT_ENGINE_SWEEP`: older
  paint-cache/layout diagnostics or default-mechanism controls tracked by separate workstreams.
  The code-editor row-scene vertical slice did not introduce them and did not replace their
  underlying mechanisms end-to-end, so deleting them here would remove unrelated diagnostics
  without an equivalent gate.

## Residual Follow-On Work

These are real architecture follow-ons, but they are outside the current code-editor closeout goal:

- accept or supersede ADR 0327 after review,
- design a public/non-page-specific boundary hint API that can eventually replace direct
  `contained_layout` authoring hints,
- consolidate broader view-cache rendered/next maps and paint-cache previous-op-range replay into
  final boundary-owned build/paint stores,
- decide the future of older paint-cache/layout env knobs in their owning workstreams,
- add a stricter code-editor paint stressor only if `ui-code-editor-resize-probes` stops catching
  regressions.

## Verdict

The current goal's code-editor vertical slice satisfies ADR 0327's first vertical-slice success
criteria: explicit boundary attribution, boundary-owned prepaint/scene-fragment state for the row
replay path, retired transitional private diagnostics, correctness gates, layering/check gates,
perf gate, and worst-bundle attribution with more than 20-30% improvement on the selected
paint-side bottleneck.
