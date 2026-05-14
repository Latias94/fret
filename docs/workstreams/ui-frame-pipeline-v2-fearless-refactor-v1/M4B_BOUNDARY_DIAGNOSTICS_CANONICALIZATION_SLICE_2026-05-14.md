# M4B Boundary Diagnostics Canonicalization Slice - 2026-05-14

Status: landed as a deletion/diagnostics ownership slice; no new perf claim.

## Summary

This slice retires the M1 transitional `debug.cache_roots[].boundary` bundle field and makes
`debug.boundaries[]` the canonical diagnostics surface for boundary-owned outcomes.

Before this slice:

- Bootstrap emitted a nested boundary summary under each cache root.
- `debug.boundaries[]` existed, but part of its cache-root outcome data was copied from the nested
  transitional summary.
- `fret-diag stats` read cache-root boundary report fields from the nested bundle path.

After this slice:

- `debug.cache_roots[]` remains only a cache-root compatibility/debug view.
- `debug.boundaries[]` owns boundary diagnostics, including build/reuse/layout/paint outcome fields
  joined from matching cache-root stats when the boundary id matches the cache-root node id.
- `fret-diag stats` preserves its `top_cache_roots[].boundary` report summary by joining from
  `debug.boundaries[]` at report-compute time, not by requiring the old nested bundle field.

## Code Changes

- `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs`
  - removes `UiCacheRootStatsV1::boundary`,
  - removes `UiBoundaryCacheRootDiagnosticsV1`,
  - derives boundary build/layout/paint outcomes directly in
    `UiBoundaryDiagnosticsV1::from_boundary_stats(...)`.
- `crates/fret-diag/src/stats/bundle_stats_compute.inc.rs`
  - builds a `debug.boundaries[]` lookup keyed by boundary id,
  - fills cache-root report boundary fields from the canonical top-level boundary object.
- `crates/fret-diag/src/tests.rs`
  - updates the boundary summary regression fixture to use `debug.boundaries[]` rather than the
    retired nested field.

## Correctness Gates

Bootstrap diagnostics gate:

```bash
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics \
  cache_root_boundary \
  boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes \
  --no-fail-fast
```

Diag stats join gate:

```bash
cargo nextest run -p fret-diag bundle_stats_preserves_cache_root_boundary_summary --no-fail-fast
```

Compile/layering gates:

```bash
cargo check -p fret-bootstrap -p fret-diag --features diagnostics
cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-bootstrap --features syntax-rust,diagnostics
python3 tools/check_layering.py
```

Observed result:

- `fret-bootstrap` boundary diagnostics nextest: `5 passed, 97 skipped`.
- `fret-diag` stats join nextest: `1 passed, 818 skipped`.
- `cargo check -p fret-bootstrap -p fret-diag --features diagnostics`: passed.
- `cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-bootstrap --features syntax-rust,diagnostics`:
  passed.
- `python3 tools/check_layering.py`: passed.
- `cargo fmt`: passed.
- `python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null`:
  passed.
- `git diff --check`: passed.

## Deletion Note

Deleted or retired in this slice:

- the serialized `debug.cache_roots[].boundary` bundle path,
- `UiBoundaryCacheRootDiagnosticsV1`,
- bootstrap-side copying from nested cache-root boundary diagnostics into top-level boundaries,
- `fret-diag stats` dependence on the old nested bundle field.

Still intentionally not deleted:

- `debug.cache_roots[]` remains as a cache-root compatibility/debug view and for report summaries.
- `fret-diag` report JSON still exposes `top_cache_roots[].boundary` as a derived report summary,
  sourced from canonical `debug.boundaries[]`.
- Historical slice notes still mention `debug.cache_roots[].boundary` as old evidence for M1/M2.

## Perf Evidence

This slice changes diagnostics shape only. It does not make a new optimization claim.

The latest closeout perf evidence is:

- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514`
- Threshold report:
  `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/check.perf_thresholds.json`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/1778700500609/bundle.schema2.json`

Observed result from the latest closeout run:

- gate failures: `[]`,
- total p50/p95/max: `1205/1396/1396us`,
- layout p50/p95/max: `231/320/320us`,
- prepaint p50/p95/max: `243/339/339us`,
- paint p50/p95/max: `710/839/839us`,
- row scene replay hit rate: `99-100%`,
- renderer prepare/encode/upload counters stayed at `0`.

Worst-bundle attribution:

- `target/release/fretboard-dev diag stats target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/1778700500609/bundle.schema2.json --sort time --top 15`
- time sum: total `11285us`, layout `1009us`, prepaint `2905us`, paint `7371us`
- time p50/p95: total `1151/1396us`, layout `34/337us`, prepaint `255/375us`,
  paint `661/875us`
- hot p50/p95: `layout.engine_solve=0/132us`, `paint.widget=443/650us`,
  `paint.text_prepare=9/11us`
- `code_editor.paint_perf` planned/used replay entries: `2090/2090`
- `code_editor.paint_perf` rows replayed: `2885`
- `code_editor.paint_perf` p50/p95 `us_row_text`: `0/13us`
- `code_editor.paint_perf` p50/p95 `us_row_scene_prepaint_plan`: `54/86us`
- `code_editor.paint_perf` p50/p95 total: `182/385us`

Compared with the M1 boundary-diagnostics bottleneck evidence (`paint.widget` p95 `1494us`,
paint p95 `1737us`), the latest closeout run shows the selected paint-side bottleneck has moved to
`paint.widget` p95 `650us` and paint p95 `875us`, which exceeds the target 20-30% improvement on
the selected bottleneck. Total p95 also improved from `1811us` to `1396us`, exceeding the 20% threshold.

## Remaining Gaps

- The completion audit is recorded in `CLOSEOUT_AUDIT_2026-05-14.md`.
- Broader view-cache/build-boundary consolidation remains future work beyond the code-editor
  vertical slice.
- The broader ADR 0327 lane remains active for follow-on build-boundary/view-cache/paint-cache
  consolidation.
