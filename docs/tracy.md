# Tracy Timeline Profiling (via `tracing`)

This repository supports a native (desktop) Tracy workflow by streaming `tracing` spans/events into Tracy.
This provides a **timeline view** of frame phases (runner + UI), with nested spans and per-thread tracks.

Scope:

- Native only (non-`wasm32`).
- This complements (not replaces) diagnostics bundles (`FRET_DIAG`) and scripted UI repros.

## Quick start

1. Launch the Tracy Profiler UI and wait for a client connection.
2. Run an app with the `fret-bootstrap/tracy` feature and `FRET_TRACY=1`.

Example (UI gallery):

```powershell
$env:FRET_TRACY=1

# Optional: include fine-grained `TRACE` spans inside `fret-ui`.
$env:RUST_LOG="info,fret_ui=trace"

cargo run -p fret-ui-gallery --release --features fret-bootstrap/tracy
```

## Optional call stacks

Tracy zones can optionally collect call stacks (useful for answering “who called this?”).
This adds overhead, so it is **off by default**.

```powershell
$env:FRET_TRACY=1
$env:FRET_TRACY_CALLSTACK=1

# Optional: adjust stack depth (default: 16)
$env:FRET_TRACY_CALLSTACK_DEPTH=16

cargo run -p fret-ui-gallery --release --features fret-bootstrap/tracy
```

## What spans to expect

At a high level you should see:

- `fret.frame` (per frame, includes `tick_id`, `frame_id`, `window`)
- `fret.ui.view`, `fret.ui.overlay`, `fret.ui.layout`, `fret.ui.paint`
- `fret.runner.redraw`, `fret.runner.prepare`, `fret.runner.render`,
  `fret.runner.record`, `fret.runner.present`, `fret.runner.render_scene`

If you enable `fret_ui=trace`, you should also see:

- `fret_ui.layout_all`
- `fret_ui.paint_all`
- `fret.ui.layout_engine.solve`
- `fret.ui.paint_cache.replay`
- cache-root spans under `ui.cache_root.*` (when view cache is active)
  - `ui.cache_root.mount` / `ui.cache_root.reuse` (cache-root mount decisions)

## Correlating Tracy with `diag perf` / `bundle.json`

Recommended workflow:

1. Use scripted UI actions to reproduce and locate the slow path:

```powershell
cargo run -p fretboard -- diag perf ui-gallery `
  --launch -- cargo run -p fret-ui-gallery --release --features fret-bootstrap/tracy
```

2. `diag perf` prints (per script) the slowest frame summary and the bundle path.
3. In Tracy, filter to `fret.frame` spans and jump to the matching `frame_id` / `tick_id`.
4. Once you are on the slow frame, expand:
   - `fret.ui.layout` vs `fret.ui.paint` to split UI cost,
   - `fret.runner.render_scene` / `fret.runner.present` to see GPU-side submission boundaries.

Tip: after identifying the bundle, you can also inspect the slowest snapshots directly:

```powershell
cargo run -p fretboard -- diag stats <bundle_dir> --sort time --top 20
```

## Instrumentation overhead (what happens when it's "off")

The profiling story in this repository intentionally separates **span emission** (via `tracing`)
from **per-frame debug counters/timers** (diagnostics `debug_stats`), so the default experience
remains fast.

### `tracing` / Tracy spans

- When Tracy is disabled (`FRET_TRACY` unset) and `TRACE` level is not enabled for a module,
  spans are gated behind `tracing::enabled!(Level::TRACE)` (or equivalent checks).
- In the "off" case, a scope typically becomes a single cheap branch, and avoids:
  span allocation, entering/exiting zones, and field recording.

Recommended defaults:

- Day-to-day runs: do not enable `fret_ui=trace` unless you are actively investigating a
  timeline issue.
- Profiling runs: enable Tracy + selectively enable `TRACE` for the module you're investigating
  (e.g. `fret_ui=trace`).

### `debug_stats` timing

- Frame stats (`debug_stats`) are only measured and accumulated when the UI debug/perf stats
  mode is enabled.
- When it is disabled, `fret_perf` helpers return early without calling `Instant::now()`.

If you need **minimal perturbation**, prefer:

1. Tracy timeline with a narrow set of `TRACE` spans enabled, and
2. avoid enabling broad debug/perf stats unless you need the counters for attribution.

## Tuning trace volume

- Default `RUST_LOG` may not include `TRACE` spans.
- Prefer narrowing scope over enabling `trace` globally:
  - `RUST_LOG="info,fret_ui=trace"` (UI tree internals)
  - `RUST_LOG="info,fret_launch=trace"` (runner internals, if needed)

If Tracy becomes noisy, disable `TRACE` and rely on the higher-level `info` spans first.
