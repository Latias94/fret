# Retained Layout Orchestration v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Starting Evidence

From `fret-ui-layout-architecture-audit-v1`:

- Worst frame: `total=2803us`, `layout=2304us`, `layout_roots=2181us`,
  `layout_engine_solve=202us`, `prepaint=202us`, `paint=297us`.
- Renderer text remained bounded: `renderer_prepare_text_us=65us`.
- `ViewCache` root reuse was stable: `cache_roots_reused=1/1`.
- Top layout hotspots pointed at retained-tree/barrier orchestration:
  `Semantics` inclusive around `2177us`, `Scroll` around `281us`, and `ViewCache` around `373us`.

This is local orientation evidence, not a cross-machine formal baseline.

## Fresh Attribution

Captured on 2026-05-18 from the resize-jitter baseline:

- Bundle: `target/fret-diag/retained-layout-orchestration-v1-baseline/1779080825844/bundle.schema2.json`
- Stats: `target/fret-diag/retained-layout-orchestration-v1-baseline/1779080825844/worst.stats.json`
- Layout summary: `target/fret-diag/retained-layout-orchestration-v1-baseline/1779080825844/layout-perf-summary.json`

Fresh worst-frame shape:

- `total=3050us`
- `layout=2477us`
- `layout_roots=2349us`
- `layout_engine_solve=220us`
- `prepaint=235us`
- `paint=338us`
- `cache_roots=1`
- `cache_roots_reused=1`
- `contained_relayouts=0`
- `barrier(set_children/scheduled/performed)=0/0/0`

Top layout hotspots in the summary:

- `Semantics` inclusive `2345us`, layout `1894us`
- `Scroll` inclusive `299us`, layout `197us`
- `ViewCache` inclusive `411us`, layout `77us`

Classification:

- Dominant owner is retained root solve scheduling / root orchestration around the `Semantics` root.
- `ViewCache` is stable in this sample and does not look like the dominant owner.
- `Scroll` is secondary and still must be treated as a side-effect boundary.
- No local runtime change is justified yet; RLO-030 should only start from a narrower owner slice.

## RLO-030 Semantics Wrapper Fast Path

Implementation:

- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
  - `clean_geometry_node_contract(...)` already classified `ElementInstance::Semantics(_)` as a
    pure `PreserveLocalOrigins` wrapper.
  - RLO-030 adds `ElementInstance::Semantics(_)` to
    `clean_engine_geometry_propagation_supported_element(...)`, allowing the existing
    clean-geometry propagation path to execute instead of falling back to wrapper/subtree layout.
- `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
  - Added `clean_geometry_small_resize_propagates_through_semantics_wrapper`.
  - The RED state skipped the root Taffy solve but still performed wrapper/subtree layout
    (`layout_nodes_performed=10`, then `9` after only adding `Semantics` while the fixture still
    used `Pressable` leaves).
  - The final fixture isolates `Semantics -> Flex -> Container rows`, proving the Semantics wrapper
    itself propagates clean geometry, refreshes descendant element bounds, and performs at most the
    scheduling parent layout on a small width-only resize.

After bundle captured on 2026-05-18:

- Bundle: `target/fret-diag/retained-layout-orchestration-v1-rlo030-after/1779083266980/bundle.schema2.json`
- Command: same resize-jitter script and environment as the baseline, with
  `--dir target/fret-diag/retained-layout-orchestration-v1-rlo030-after`.
- Worst-frame shape:
  - `total=1442us`
  - `layout=848us`
  - `layout_roots=747us`
  - `layout_engine_solve=200us`
  - `prepaint=252us`
  - `paint=342us`
  - `cache_roots=1`
  - `cache_roots_reused=1`
  - `contained_relayouts=0`
  - `barrier(set_children/scheduled/performed)=0/0/0`

Before/after `diag stats --diff` highlights:

- `p95.total_time_us`: `3050 -> 1442` (`-52.7%`)
- `p95.layout_time_us`: `2479 -> 885` (`-64.3%`)
- `p95.layout_roots_time_us`: `2349 -> 747` (`-68.2%`)
- `p95.layout_engine_solve_time_us`: `220 -> 214` (`-2.7%`)

Interpretation:

- The win came from avoiding retained wrapper/subtree layout work around the `Semantics` root, not
  from materially changing Taffy solve cost.
- `Scroll` remains a side-effect boundary and was not skipped by name.
- `ViewCache` contained relayout semantics remained stable (`contained_relayouts=0`,
  `cache_roots_reused=1`).
- The after layout summary still shows follow-on owners (`Pressable`, `Scroll`, `ViewCache`). Those
  should be split into a new task or workstream rather than expanding RLO-030.

## Smallest Current Repro

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
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --dir target/fret-diag/retained-layout-orchestration-v1-baseline \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Inspect the resulting bundle with:

```bash
target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 20
```

## Gate Set

### Attribution Gate

```bash
target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 20
```

Proves the next implementation slice has a concrete owner.

### Targeted Correctness Gate

```bash
cargo nextest run -p fret-ui layout_engine scroll view_cache --no-fail-fast
```

Proves the layout engine, scroll, and view-cache invariants still hold for any orchestration change.

RLO-030 result: passed, `257 tests run: 257 passed`.

### Boundary Gate

```bash
python3 tools/check_layering.py
```

Proves the fix stays in the mechanism layer and does not introduce reverse dependencies.

RLO-030 result: passed.

### Format And Diff Gates

```bash
cargo fmt --check
git diff --check
```

RLO-030 result: both passed.

### Closeout Gates

```bash
python3 -m json.tool docs/workstreams/retained-layout-orchestration-v1/WORKSTREAM.json
python3 tools/check_workstream_catalog.py
```

RLO-040 result: both passed after closeout edits.

## Evidence Anchors

- `docs/workstreams/retained-layout-orchestration-v1/DESIGN.md`
- `docs/workstreams/retained-layout-orchestration-v1/TODO.md`
- `docs/workstreams/retained-layout-orchestration-v1/MILESTONES.md`
- `docs/workstreams/retained-layout-orchestration-v1/CLOSEOUT_AUDIT_2026-05-18.md`
- `docs/workstreams/fret-ui-layout-architecture-audit-v1/CLOSEOUT_AUDIT_2026-05-18.md`
- `docs/workstreams/scroll-optimization-v1/HANDOFF.md`
- `crates/fret-ui/src/tree/layout/entrypoints.rs`
- `crates/fret-ui/src/tree/layout/solve.rs`
- `crates/fret-ui/src/tree/layout/node.rs`
- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
- `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs`
- `crates/fret-ui/src/layout/engine.rs`

## Notes

Do not treat a `Scroll` node name as permission to skip layout. The point of this lane is to
separate required side effects from avoidable retained orchestration work.
