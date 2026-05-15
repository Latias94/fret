# Phase timeline coverage audit (2026-05-15)

## Scope

This audit closes the current coverage-mapping task for `diag-perf-profiling-infra-v1`.
It does not claim the profiling infra workstream is complete. The goal is narrower: map the current
frame timeline and trace surfaces to concrete source anchors, then identify what is still missing
before real-span trace export work starts.

## Assumptions

- Confident: `diag trace` / `diag perf --trace` is a bundle-derived Chrome trace exporter, not a live
  `tracing` span exporter.
  - Evidence: `crates/fret-diag/src/trace.rs`; `trace_source=bundle_synthetic_phases`;
    `real_spans_included=false`.
  - Consequence if wrong: the real-span follow-up would duplicate an already-shipped capability.
- Confident: Tracy support exists as an opt-in native profiling path.
  - Evidence: `ecosystem/fret-bootstrap/src/lib.rs`; `docs/tracy.md`; `crates/fret-diag/src/diag_repro/launch.rs`.
  - Consequence if wrong: the follow-up would need bootstrap plumbing before capture/export work.
- Likely: most editor-grade UI runtime phases already have stable span names or `fret_perf` measurement hooks.
  - Evidence: table below.
  - Consequence if wrong: the next implementation slice should add missing timers before adding export tooling.
- Confident: automated Tracy capture-to-file is not implemented.
  - Evidence: `docs/tracy.md`; `crates/fret-diag/src/diag_repro/summary.rs`; `tracy.note.md` copy says the capture
    file is not recorded automatically.
  - Consequence if wrong: the next slice should be a docs/gate correction, not new capture integration.

## Coverage table

| Area | Current coverage | Evidence anchors | Verdict |
| --- | --- | --- | --- |
| Frame boundary | `fret.frame` info span with frame/window identifiers | `ecosystem/fret-bootstrap/src/ui_app_driver.rs` | Covered |
| High-level UI phases | `fret.ui.view`, `fret.ui.overlay`, `fret.ui.layout`, `fret.ui.paint`, diagnostics drive span | `ecosystem/fret-bootstrap/src/ui_app_driver.rs` | Covered |
| Layout sub-phases | `fret_perf::measure_span` timers for visible-root collection, scroll-handle invalidation, view-cache invalidation expansion, request-build roots, roots, pending barriers, view-cache, repair/contained roots/collapse observations, prepaint-after-layout, focus repair, semantics refresh, deferred cleanup. `layout_all` regular, fast-path, and skipped-engine final tail phases now share `run_layout_post_layout_phases`. | `crates/fret-ui/src/tree/layout/entrypoints.rs` | Covered for current major layout phases |
| Layout engine solve | `fret.ui.layout_engine.solve` spans around Taffy solves use `fret_perf::measure_span_with_finish` so solve stats and span fields close together (`elapsed_us`, measure counts/cache hits/time, additive `outcome`) | `crates/fret-perf/src/lib.rs`; `crates/fret-ui/src/layout/engine.rs` | Covered |
| Cache-root layout | `ui.cache_root.layout` span | `crates/fret-ui/src/tree/layout/node.rs` | Covered |
| Prepaint | `fret.ui.prepaint.after_layout` and `fret.ui.prepaint.after_layout_stable_frame` | `crates/fret-ui/src/tree/prepaint/entry.rs` | Covered |
| Paint high-level | `fret.ui.paint_all`; entry-layer sub-phases for input context, scroll-handle invalidation, root collection, visual-bounds flushing, text-input snapshot publishing, and paint-observation collapse | `crates/fret-ui/src/tree/paint/entry.rs` | Covered |
| Paint cache replay | `fret.ui.paint_cache.replay` plus cache-root paint span | `crates/fret-ui/src/tree/paint/node.rs` | Covered |
| Dispatch | `fret.ui.dispatch.*` spans for event body, context build, target routing, pointer arbitration, widget capture/bubble, hover/cursor, and post-dispatch snapshot | `crates/fret-ui/src/tree/dispatch/window.rs` | Covered for common input paths |
| Hit-test | `fret.ui.hit_test.*` spans for layer walk, cached path, bounds-tree query, fallback traversal, and candidate checks | `crates/fret-ui/src/tree/hit_test.rs` | Covered |
| Renderer | `fret.renderer.*` spans for render scene, prepare text/SVG, scene encode, plan compile, upload, record passes, passes, encoder finish, targets, pools, and pipeline creation. Frame-level text/SVG prepare, scene encode, and upload timers now route through `fret_perf::measure_span` so `RenderPerfStats` buckets and span boundaries share one call site. | `crates/fret-render-wgpu/src/renderer/**` | Covered for WGPU renderer internals |
| Chrome trace artifact | Stable additive JSON schema, Chrome events generated from bundle stats. UI layout/paint sub-phase events are driven by trace metadata in the perf key registry and covered by a synthetic-event test. | `crates/fret-diag/src/perf_keys.rs`; `crates/fret-diag/src/trace.rs`; `docs/workstreams/diag-perf-profiling-infra-v1/perf-key-registry.frame-stats.json` | Covered as synthetic timeline only |
| Tracy capture/export | `diag repro --with tracy` enables `FRET_TRACY=1` and can inject `fret-bootstrap/tracy`; capture saving remains manual in Tracy UI | `crates/fret-diag/src/diag_repro/launch.rs`; `crates/fret-diag/src/diag_repro/summary.rs`; `docs/tracy.md` | Partial |

## Missing or weakly verified areas

- Real-span export into a local artifact is not implemented. Current Chrome trace output is explicitly synthetic.
- Tracy capture-to-file is not automated or gateable; the UI must connect and save a capture manually.
- The typed perf key registry now covers the `debug.stats` frame fields consumed by `diag stats` and keeps a
  generated inventory in sync. The trace-exported UI layout/paint timing subset now carries stable Chrome event
  names, and `chrome_trace_synthetic_ui_subphases_cover_registered_timing_events` guards against registering a
  UI timing event without emitting it from the bundle-derived trace exporter. Threshold/config `max_*`/`min_*`
  gate keys now have a separate generated registry, keeping gate config explicit without mixing it into frame
  metric keys.
- The audit proves source coverage, not runtime quality. It does not replace running a specific perf repro and
  comparing bundle stats / trace output for that repro.

## Next action

Do not add another broad trace format yet. The next implementation slice should either:

- add an explicit real-span capture/export follow-on that owns automated Tracy capture or `tracing` JSON export, or
- close a smaller known timer gap only after a failing bundle shows an uninstrumented phase.
