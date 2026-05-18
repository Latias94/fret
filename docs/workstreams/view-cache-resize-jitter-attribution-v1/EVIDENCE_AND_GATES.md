# ViewCache Resize-Jitter Attribution v1 - Evidence And Gates

Status: Active
Last updated: 2026-05-18

## Starting Evidence

Fresh resize-jitter confirmation from `pressable-clean-geometry-propagation-v1`:

- Bundle:
  `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/1779088062238/bundle.schema2.json`
- Layout summary:
  `target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/layout.perf.summary.v1.json`

Stats command:

```bash
target/release/fretboard-dev diag stats \
  target/fret-diag/pressable-clean-geometry-propagation-v1-pgp050-after/1779088062238/bundle.schema2.json \
  --sort time --top 20
```

Observed local result:

- `p95.total_time_us=1477`
- `p95.layout_time_us=930`
- `p95.layout_engine_solve_time_us=215`
- Worst frame: `total_us=1477`, `layout_us=897`, `solve_us=211`

Layout hotspot summary:

```text
ViewCache layout_us=380 inclusive_us=723
Scroll layout_us=205 inclusive_us=331
Flex layout_us=83 inclusive_us=122
```

Interpretation:

- `Pressable` is no longer a worst-frame layout hotspot in this sample.
- `ViewCache` is the largest remaining layout owner, but this evidence does not yet distinguish
  cache-root phases or prove that a runtime optimization is safe.
- The evidence is local orientation, not a cross-machine performance baseline.

## Source Evidence To Preserve

- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
  - `ElementInstance::ViewCache(_)` is classified as a `side_effect_boundary()`.
- `crates/fret-ui/src/element.rs`
  - `ViewCacheProps` owns `layout`, `boundary_hints`, and `cache_key`.
- `crates/fret-ui/src/elements/runtime.rs`
  - `ViewCacheBuildBoundaryStore` tracks build-time view-cache boundary state.
  - Runtime paths keep cached root elements, action-hook state, membership, keys, and debug paths
    alive across reuse frames.
- `crates/fret-ui/src/tree/view_boundary.rs`
  - `ViewBoundaryKind::ViewCacheRoot` identifies cache-root boundaries and layout dependency
    metadata.
- `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - View-cache layout phases include invalidation expansion, root-bound repair, contained root
    relayout, observation collapse, and scroll follow-up scheduling.

## Canonical Gates

Workstream state:

```bash
python3 -m json.tool docs/workstreams/view-cache-resize-jitter-attribution-v1/WORKSTREAM.json
python3 tools/check_workstream_catalog.py
git diff --check
```

Boundary and formatting gates for runtime slices:

```bash
python3 tools/check_layering.py
cargo fmt --check
```

Focused source audit command:

```bash
rg -n "ViewCache|view_cache|ViewBoundaryKind::ViewCacheRoot|contained_relayout|cache_root" \
  crates/fret-ui/src/element.rs \
  crates/fret-ui/src/elements/runtime.rs \
  crates/fret-ui/src/tree/view_boundary.rs \
  crates/fret-ui/src/tree/layout \
  -S
```

Focused Rust proof placeholder:

```bash
cargo nextest run -p fret-ui view_cache layout_engine --no-fail-fast
```

Fresh perf capture template:

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

Stats command for a captured bundle:

```bash
target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 20
```

## Evidence Rules

- Do not treat `ViewCache` as a pure wrapper until the source audit proves a narrower safe case.
- Do not merge `Scroll` into this lane. A scroll follow-up relayout can be related to
  `ViewCache`, but the owner verdict must stay separate.
- Prefer a no-change or diagnostics-first verdict over a speculative runtime optimization.
- Any runtime change must preserve cache-root liveness, state retention, boundary tracing, and
  scroll extent correctness.
