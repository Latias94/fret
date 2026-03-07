# UI Memory Footprint Closure (v1) — TODO

## Diagnostics (tool-side)

- [x] Parse `resource.vmmap_summary.txt` region table into structured JSON (top N by resident/dirty).
- [x] Parse `MALLOC ZONE` allocated + frag into structured JSON when present.
- [x] Capture a bounded `vmmap -sortBySize -wide -interleaved -noCoalesce` region list to break down large buckets like `owned unmapped memory`.
- [x] Add `vmmap` parsing fields to `resource.footprint.json` schema (best-effort; macOS-only).
- [x] Add a `fretboard diag compare --footprint` view that prints deltas for the structured fields.
- [x] Add `fretboard diag memory-summary` to summarize distributions across multiple `--session-auto` samples.
- [x] Ensure `fretboard diag repeat` materializes per-run `evidence.index.json` so `memory-summary` can aggregate repeat outputs.
- [x] Capture Apple `/usr/bin/footprint --json` output in bundles (macOS-only) and surface a summary under `macos_footprint_tool_steady`.
- [x] Add `fretboard diag memory-summary --footprint-categories-agg` to aggregate `footprint` category dirty bytes across samples.
- [x] Surface renderer attribution fields in `memory-summary` (`renderer_gpu_images_bytes_estimate`, `renderer_gpu_render_targets_bytes_estimate`, `renderer_intermediate_peak_in_use_bytes`).
- [x] Add `tools/sample_external_process_memory.py` for external macOS GUI-process sampling (`footprint -j` + `vmmap -summary`).
  - Supports `--sample-at-secs 2,6,12` timeline capture in a single process.

## Diagnostics (app-side)

- [x] Add heap byte estimates for render_text caches (shape cache + blob payload slices).
- [ ] Extend heap byte estimates for text caches further (measure caches, line layout internals beyond best-effort).
- [ ] Add cache byte estimates for images/assets where feasible (distinguish CPU decoded bytes vs GPU textures).
- [ ] Keep all new fields behind a “diagnostics” surface (non-contract; best-effort).

## Scripted repro matrix

- [x] Add `tools/diag-scripts/tooling/empty/empty-idle-memory-steady.json` (schema v2).
- [x] Add `tools/diag-scripts/tooling/text/text-heavy-memory-steady.json` (forces emoji/color glyphs).
- [x] Add `tools/diag-scripts/tooling/images/image-heavy-memory-steady.json` (forces texture cache).
- [x] Add `tools/diag-scripts/tooling/images/image-heavy-memory-steady-after-drop.json` (drops registered images + idle).
- [x] Add `hello_world_compare_demo` (`apps/fret-demo --bin hello_world_compare_demo`) as a minimal 500x500 Fret compare target for external baselines.

## Attribution experiments (macOS / Metal)

- [x] Sweep `FRET_DIAG_WGPU_ALLOCATOR_REPORT_EVERY_N_FRAMES` and `FRET_DIAG_WGPU_REPORT_EVERY_N_FRAMES`
  (60 vs 600) and measure:
  - outlier frequency for `wgpu_metal_current_allocated_size_bytes_{min,max}`
  - stability of `macos_vmmap_steady.regions.io_surface_dirty_bytes` / `io_accelerator_dirty_bytes`
  - overhead (bundle size + tooling time)
- [x] Confirmed: cadence 60 produces outliers; cadence 600 is stable (see workstream README snapshot).
- [x] Default memory scripts to cadence 600 and keep a separate “high-frequency attribution” script for deep dives:
  - Baseline: `tools/diag-scripts/tooling/todo/todo-memory-steady.json` (cadence 600)
  - Deep dive: `tools/diag-scripts/tooling/todo/todo-memory-steady-wgpu-highfreq.json` (cadence 60)
  - Baseline scripts also include `empty-idle`, `text-heavy`, `image-heavy` (cadence 600)

- [ ] Correlate the `vmmap` headline bucket (`owned unmapped memory` dirty) with `footprint` categories:
  - Determine which `footprint` categories rise/fall with `owned unmapped memory` across scenarios.
  - If one category dominates, add a dedicated gate for it (monitor-only at first).

- [x] Attribution: sweep `FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY` (1/2/3) on `empty-idle` and record the impact.
- [x] Attribution: sweep `FRET_WGPU_MEMORY_HINTS` (`performance` vs `memory`) on `text-heavy` and record the impact.
- [x] Attribution: release images + idle (image-heavy) and confirm `Owned physical footprint (unmapped) (graphics)` returns close to baseline after `renderer.unregister_image`.
- [x] Attribution: A/B `FRET_IMAGE_HEAVY_DEMO_POLL_AFTER_DROP` (1 vs 0; idle 1200 frames) and confirm no material delta in post-drop steady state.
- [ ] Attribution: sweep `FRET_IMAGE_HEAVY_DEMO_COUNT` (6/12/24/48 at 1024px) and fit a simple slope:
  - `Owned physical footprint (unmapped) (graphics)` vs `renderer_gpu_images_bytes_estimate`
  - `wgpu_metal_current_allocated_size_bytes_max` vs `renderer_gpu_images_bytes_estimate`
  - Goal: separate baseline intercept (swapchain/driver/allocator) from per-image growth.
- [x] Captured initial sweep (local 2026-03-05; N=3 each) and observed ~1:1 scaling with a ~100 MiB Metal baseline and ~217 MiB footprint baseline.
- [ ] Attribution: sweep `FRET_IMAGE_HEAVY_DEMO_SIZE_PX` (count=24; size=512/1024/2048) and validate whether:
  - the ~1:1 bytes/byte slope still holds (or if there is a material tiling/alignment multiplier),
  - the intercept remains stable (baseline driver/swapchain).
- [x] Captured initial size sweep (local 2026-03-05; N=3 each) and observed ~1:1 bytes/byte scaling with stable intercepts.

- [x] Capture the first external Fret-vs-GPUI hello-world-class baseline (local 2026-03-06) and verify whether the gap survives when Fret text/swatches are removed.
- Observed (sample):
  - Fret `hello_world_compare_demo`: ~303–318 MiB current physical footprint, dominated by ~208–222 MiB current `owned unmapped memory` dirty plus ~24–41 MiB `IOAccelerator` / ~20–24 MiB `IOSurface`.
  - Fret `hello_world_compare_demo` with `FRET_HELLO_WORLD_COMPARE_NO_TEXT=1` or both `NO_TEXT=1` + `NO_SWATCHES=1`: headline graphics bucket remains near the same level, so fonts/text are not the primary driver.
  - GPUI `hello_world` (macOS Blade/Metal, not `wgpu`): repeated 6s/12s runs settle near ~18.5–18.6 MiB current physical footprint with no visible current `owned unmapped memory` dirty region.
- [x] Extend external sampling to repeated/timeline mode and emit a compact summary table automatically.
- Observed (timeline sample):
  - Fret `hello_world_compare_demo`: same-process `2s/6s/12s` stays around ~263.1 → ~266.2 → ~269.3 MiB physical with ~219.1 → ~221.8 → ~221.8 MiB current `owned unmapped memory` dirty (`target/diag/external-fret-hello-world-compare-timeline-20260306/`).
  - GPUI `hello_world` rerun: same-process `2s/6s/12s` now sits around ~29.8 → ~29.8 → ~27.8 MiB physical with ~4.1 → ~4.1 → ~2.2 MiB current `owned unmapped memory` dirty (`target/diag/external-gpui-hello-world-timeline-20260306-r2/`).
- [x] Pair compare-demo external runs with internal `wgpu` / Metal allocator counters.
- Observed (paired sample):
  - Full compare timeline at `6s`: external graphics buckets total about **~237.9 MiB**, while internal `metal_current_allocated_size_bytes` is only about **~42.5 MiB**.
  - `NO_TEXT=1` at `6s`: external graphics buckets still total about **~227.4 MiB**, while internal `metal_current_allocated_size_bytes` falls only to about **~38.3 MiB**.
  - This leaves a persistent **~186–196 MiB** residual outside the app-visible Metal allocation we currently expose.
- [x] Extend the new timeline capture to Fret `NO_TEXT=1` / `NO_SWATCHES=1` variants for same-process attribution.
- [x] Also capture the fully empty compare variant (`NO_TEXT=1` + `NO_SWATCHES=1`) for the same paired timeline.
- [x] Sweep compare-demo surface knobs (`FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY`, MSAA, window size) with paired external+internal sampling.
- [x] Add an Apple `xctrace` helper for the compare demo (`tools/capture_hello_world_compare_xctrace.py`) so we can jump from the compare repro directly into `Metal System Trace` without hand-driving Instruments.
- Observed (local helper validation; 2026-03-06):
  - `Metal System Trace` smoke capture succeeds for the empty compare case (`target/diag/test-hello-world-xctrace-smoke-20260306-r4/summary.json`).
  - The first full helper run also succeeds for `baseline` / `empty` / `size1000` (`target/diag/hello-world-compare-xctrace-20260306-r1/summary.json`).
  - `Game Memory` is still useful but finalizes much more slowly on this machine, so the helper keeps it opt-in instead of the default path.
- [x] Add a bounded app-side cadence sample for `hello_world_compare_demo` and reconcile it with the first `xctrace` interpretation.
- Observed (runtime cadence sample; local 2026-03-06):
  - `target/diag/hello-world-compare-runtime-sample-20260306-r1.json` stays at `render_count=2` / `last_frame_id=1` across `1s` / `2s` / `3s` without any `FRET_DIAG*` environment, so the minimal compare scene is **not** continuously re-rendering in steady idle.
  - This means the earlier `xctrace`-derived `~119 Hz` signal cannot be treated as a direct “Fret idle redraw loop” proxy.
- [x] Add a compare-demo `xctrace` summarizer and re-label the cadence evidence conservatively.
- Observed (xctrace interpretation update; local 2026-03-06):
  - `tools/summarize_hello_world_compare_xctrace.py` now emits bounded per-schema summaries for `display-surface-queue`, `ca-client-present-request`, `displayed-surfaces-interval`, and `metal-application-encoders-list`.
  - `display-surface-queue` has no process column in the export, so it should be treated as a global compositor table, not an app-only cadence proxy.
  - `ca-client-present-request` / `metal-application-encoders-list` still show high-frequency activity for the compare demo, but they now conflict with the app-side `render_count=2` evidence and should be treated as **CA/compositor/present activity signals** until we close their Apple-side semantics.
- [x] Expose continuous-frame lease / animation-frame-request counts through diag bundle evidence and `diag memory-summary`, and teach `memory-summary` to resolve nested bundle evidence under session roots.
- Observed (diag tooling validation; local 2026-03-06):
  - `target/diag/test-hello-world-compare-repeat-20260306-r2/` now works with `target/debug/fretboard diag memory-summary ...` even though the session root itself has no `evidence.index.json`; the tool resolves nested sample bundles automatically.
  - The resulting summary shows `ui_element_runtime_continuous_frame_lease_owners_count=0`, `ui_element_runtime_continuous_frame_lease_count_total=0`, `ui_element_runtime_continuous_frame_lease_count_max=0`, and `ui_element_runtime_animation_frame_request_roots_count=0` for the minimal compare idle capture, which matches the app-side cadence story.
- [x] Add a same-backend control baseline (plain `wgpu` hello world or another tiny `wgpu` UI stack) so the GPUI Blade/Metal comparison is no longer the only external baseline.
- Observed (same-backend control; local 2026-03-06):
  - `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` reuses `fret_render::WgpuContext` + `SurfaceState`, so adapter/surface policy stays close to Fret's current `wgpu` backend.
  - The control settles near **~31.0 MiB** physical / **~12.8 MiB** graphics visible to macOS / **~9.5 MiB** internal Metal current allocation at `6s`, with `redraw_count=2` / `present_count=2` (`target/diag/external-wgpu-hello-world-control-timeline-20260306-r1/`).
  - Fret `hello_world_compare_demo` still sits near **~266.2 MiB** physical / **~238.0 MiB** graphics in the full case and **~249.0 MiB** physical / **~221.9 MiB** graphics in the empty case on the same backend (`target/diag/external-fret-hello-world-compare-timeline-20260306-r4-samebackend/`, `target/diag/external-fret-hello-world-compare-empty-timeline-20260306-r1-samebackend/`).
  - Summary artifact: `target/diag/wgpu-hello-world-control-vs-fret-20260306-r1/summary.json` / `summary.md`.
  - Conclusion: the headline gap is **not** the raw `wgpu` surface baseline; even the empty Fret scene still carries about **~183.6 MiB** residual beyond the app-visible Metal allocation we currently expose.
- [ ] Capture `Game Memory` on the compare demo (and ideally the same-backend `wgpu` control) and map the residual floor into Apple categories (`IOSurface`, `IOAccelerator`, driver/private heaps, VM reservation / unmapped graphics) before concluding “driver floor”.
  - Local note: `xcrun xctrace list templates` exposes `Game Memory` on this machine, but not a standalone `VM Tracker` template, so `Game Memory` is the first scripted Apple-side attribution path.
  - `vmmap -sortBySize` / `vmmap -v` is now captured via `tools/sample_external_process_memory.py --capture-vmmap-regions`, but it still keeps the main `owned unmapped memory` bucket opaque; it only closed the smaller `IOSurface` question (Fret empty currently shows three drawables vs two in the same-backend control).
  - The first `Game Memory` bundle inspection already surfaces promising stores (`metal-current-allocated-size`, `metal-resource-allocations`, `metal-io-surface-access`, `virtual-memory`, `metal-residency-set-*`), so the next best tooling step is a parser/summarizer for those stores rather than more content-level scene toggles.
  - The scripted attach summary (`target/diag/hello-world-compare-game-memory-20260306-r1/empty/game-memory-attach.summary.json`) now confirms that `virtual-memory` is tiny (~`1.09 MiB` cumulative event sizes), `metal-residency-set-*` is empty/zero-row on this trace, and `metal-resource-allocations` is still dominated by small `(wgpu internal) Staging` buffers. Attach mode therefore still does **not** explain the remaining Apple-side residual.
  - The new same-backend control note (`target/diag/apple-direct-store-same-backend-20260306-r3/summary.json`) also sharpens the scale claim: best-available app-visible delta is still only about **one extra drawable + ~29.33 MiB** of Metal current allocation, so the unexplained **~183–196 MiB** bucket is still elsewhere.
  - The first attach-based exports already show that `metal-current-allocated-size` matches the app-side `~38.33 MiB` and `metal-io-surface-access` sees exactly three app-owned `1000x1000` surfaces, but `metal-resource-allocations` is dominated by small `(wgpu internal) Staging` buffer churn. That strongly suggests we also need a **launch-from-start** capture mode to see the initial large allocations that created the steady-state floor.
- [x] Add a Game Memory store summarizer/parser so Apple trace bundles can be reduced into scriptable category reports without manual Instruments clicking.
  - Tooling: `tools/summarize_hello_world_compare_xctrace.py --preset game-memory-attach --process-contains hello_world_compare_demo --export-dir <dir> --export-timeout-secs <secs>`
  - Validation artifact: `target/diag/hello-world-compare-game-memory-20260306-r1/empty/game-memory-attach.summary.json`
  - Added bounded export caching / timeout reporting, so reruns no longer depend on manual Instruments navigation and slow schemas surface as structured `timed_out`/`empty` results instead of hanging silently.
- [x] Add a direct indexed-store fallback for slow `Game Memory` schemas such as `metal-current-allocated-size`, which times out through `xctrace export` on this machine.
  - The fallback now reads `indexed-store-41/bulkstore` directly, using the descriptor-declared fixed `56`-byte record layout and detected `4096`-byte data offset.
  - Validation: the fallback recovers the same `40189952`-byte (`38.33 MiB`) steady value over `6690` rows and marks the store as single-target via `target-pid=SINGLE`.
- [x] Turn the same-backend `Game Memory` comparison into a startup-inclusive control baseline.
  - Control binary now has `FRET_WGPU_HELLO_WORLD_CONTROL_EXIT_AFTER_SECS`, `FRET_WGPU_HELLO_WORLD_CONTROL_CONTINUOUS_REDRAW`, and `FRET_WGPU_HELLO_WORLD_CONTROL_PRE_INIT_SLEEP_SECS`; runtime validation for the startup hook is captured in `target/diag/test-wgpu-control-pre-init-sleep-runtime-20260306-r1.json`.
  - `tools/capture_binary_xctrace.py` now records pid lineage and bounded `ps` snapshots, which closed the earlier ownership question: in idle attach mode, `spawned_target_pid == xctrace_attached_pid`, but the control app has already gone quiet, so the store is dominated by WindowServer / GPU-helper accesses.
  - Startup-inclusive continuous control (`target/diag/wgpu-control-startup-continuous-attach-20260306-r1/summary.json`, `analysis/summary.json`) now gives us the best current app-owned control slice: `metal-io-surface-access=922 rows`, `2 x 1000x1000` surfaces, `metal-current-allocated-size≈9.0 MiB` steady / `≈9.6 MiB` max, and non-zero `metal-resource-allocations=29 rows`.
  - Summary note: `target/diag/apple-direct-store-same-backend-20260306-r3/summary.json` / `summary.md`.
  - Result: the best current same-backend evidence still bounds Fret's app-visible delta to roughly **one extra drawable + ~29.33 MiB** of Metal current allocation, which is meaningful but still much smaller than the unresolved **~183–196 MiB** Apple-side residual floor.
- [x] Add a generic binary `xctrace` helper plus direct-store metadata fallback so partial/full control captures become repeatable and bounded artifacts.
  - Helper: `tools/capture_binary_xctrace.py`
  - Metadata fallback: `tools/summarize_hello_world_compare_xctrace.py` now emits `fallback-direct-store-metadata` / `fallback-direct-store-empty` instead of only bare `timed_out`.
- [x] Make the Fret compare-side `Game Memory` capture startup-inclusive too, so the same-backend comparison is startup-inclusive on both sides.
  - `apps/fret-examples/src/hello_world_compare_demo.rs` now accepts `FRET_HELLO_WORLD_COMPARE_PRE_INIT_SLEEP_SECS`, and `tools/capture_hello_world_compare_xctrace.py` now accepts `--pre-init-sleep-secs` so the compare demo can be attached before GPU initialization.
  - Runtime validation: `target/diag/test-hello-world-compare-pre-init-sleep-runtime-20260306-r2.json` shows the startup delay explicitly (`startup.pre_init_sleep_secs=2.0`) and pushes the first internal GPU sample to `captured_since_launch_ms≈2150`.
  - Startup-inclusive Fret capture: `target/diag/hello-world-compare-game-memory-startup-attach-20260306-r1/summary.json`, summarized in `target/diag/hello-world-compare-game-memory-startup-attach-20260306-r1/empty/game-memory-startup-attach.summary.json`.
  - Aggregated same-backend note: `target/diag/apple-direct-store-same-backend-20260306-r4/summary.json` / `summary.md`.
  - Result: startup-inclusive Fret coverage reveals much denser `metal-resource-allocations`, but `metal-current-allocated-size` still tops out at the same ~`38.33 MiB` plateau, so startup inclusion still does **not** expose any app-visible Metal spike remotely close to the unresolved **~183–196 MiB** Apple-side residual floor.
  - Local build note: a fresh debug build for this demo currently needs `CARGO_INCREMENTAL=0 cargo build -p fret-demo --bin hello_world_compare_demo` on this branch because the normal incremental path can produce an arm64 linker failure with missing internal LLVM symbols.
- [x] Reduce or explain the remaining cadence asymmetry between startup-inclusive Fret and control when it is worth the cost.
  - New runtime artifacts: `target/diag/test-hello-world-compare-runner-frame-drive-runtime-normal-20260306-r4.json`, `target/diag/test-hello-world-compare-redraw-callsites-runtime-20260306-r1.json`, `target/diag/test-hello-world-compare-global-changes-runtime-20260306-r1.json`, `target/diag/test-hello-world-compare-global-changes-runtime-20260306-r2.json`.
  - Result: the asymmetry was an app-owned steady-idle continuous-present bug. `Effect::RequestAnimationFrame` was ruled out; the dominant steady-idle path was `Effect::Redraw` from `ui_app_handle_global_changes(...)`, driven by `CommandPaletteService` bookkeeping showing up in `changed_globals` almost every frame.
  - Fix: `CommandPaletteService::set_gating_handle` / `take_gating_handle` now use untracked mutation, and `command_palette_cleanup_does_not_mark_service_changed_each_frame` keeps the regression covered.
  - Post-fix verification: the default compare-demo sample now stays flat at `runner_present.total_present_count=5` across `1s/2s/3s`, and `CommandPaletteService` no longer dominates global-change diagnostics.
  - Post-fix steady memory fell accordingly: empty/full compare now settle near `48.9 / 52.2 MiB` physical at `6s` instead of the earlier `~249 / ~266 MiB` pre-fix steady numbers (`target/diag/hello-world-compare-post-fix-empty-20260306-r1/summary.json`, `target/diag/hello-world-compare-post-fix-full-20260306-r1/summary.json`).
- [ ] Re-run Apple-side category attribution on the post-fix head and treat startup peak separately from steady state.
  - Current same-backend steady delta versus `wgpu_hello_world_control` is now only about `+17.6 MiB` to `+20.9 MiB` physical at `6s`, but both sides still report much larger startup `physical_footprint_peak` values (`~236–276 MiB`).
  - Best current post-fix comparison artifact: `target/diag/wgpu-hello-vs-fret-post-fix-20260306-r1/summary.json` / `summary.md`.
  - New tooling path (local 2026-03-07): `tools/capture_binary_xctrace.py` now supports `--record-mode launch`, repeatable `--instrument`, `--dry-run`, and emits `trace_size_bytes` / `trace_complete_guess` / `trace_only_run_dir_guess` so we can distinguish full bundles from `Trace1.run`-only partials without opening Instruments.
  - Validated helper artifacts:
    - `target/diag/test-capture-binary-xctrace-launch-timeprofiler-20260307-r1/summary.json`
    - `target/diag/test-capture-binary-xctrace-launch-vmtracker-20260307-r1/summary.json`
    - `target/diag/test-capture-binary-xctrace-launch-vm-plus-metal-20260307-r1/summary.json`
  - First useful fallback: `Virtual Memory Trace + Metal Application` produces a full launch trace for `wgpu_hello_world_control`, and the paired summary already exposes both `virtual-memory` and `metal-io-surface-access` from one run (`target/diag/test-capture-binary-xctrace-launch-vm-plus-metal-20260307-r1/analysis/summary.json`).
  - First Fret empty smoke also records through the same path (`target/diag/test-capture-binary-xctrace-launch-fret-empty-vm-plus-metal-20260307-r1/summary.json`), but its `metal-io-surface-access` rows are still dominated by WindowServer / GPU helpers, so post-fix same-backend attribution still needs careful process filtering.
- [x] Add idle-present check groundwork for the compare demo.
  - Sidecar path: `hello_world_compare.internal_gpu.json` can now be auto-written into `FRET_DIAG_DIR`, and `fret-diag` can emit `check.hello_world_compare_idle_present.json` from that report.
  - Exploratory script/suite: `tools/diag-scripts/tooling/hello-world/hello-world-compare-idle-present-gate.json`, `tools/diag-scripts/suites/hello-world-compare-idle-present/suite.json`, redirect `tools/diag-scripts/hello-world-compare-idle-present-gate.json`.
  - Direct validation on current head: a no-diag run stays flat at `runner_present.total_present_count=4` across `2s/3s/4s` (`target/manual-hello-world-compare-idle/hello_world_compare.internal_gpu.json`).
- [x] Finish an apples-to-apples idle-present regression gate path for `diag run` and `diag suite`.
  - `fretboard diag run tools/diag-scripts/tooling/hello-world/hello-world-compare-idle-present-gate.json --launch -- cargo run -p fret-demo --bin hello_world_compare_demo` now takes an external / no-diagnostics launch path, writes `hello_world_compare.internal_gpu.json` via `FRET_HELLO_WORLD_COMPARE_INTERNAL_REPORT_PATH`, and evaluates `check.hello_world_compare_idle_present.json` after the demo self-exits.
  - `fretboard diag suite hello-world-compare-idle-present --launch -- cargo run -p fret-demo --bin hello_world_compare_demo` now uses the same external / no-diagnostics post-run path and writes a passing `suite.summary.json` instead of relying on the in-band diagnostics frame loop.
  - Verified closure artifacts: `target/fret-diag-hello-world-compare-idle-gate-r2/` (`diag run`) and `target/fret-diag-hello-world-compare-idle-suite-r3/` (`diag suite`) both record `diag_env_enabled_guess=false`, flat `runner_present.total_present_count`, and `present_delta=0`.

- [x] Add a cadence-matched active continuous-present baseline and remove the desktop RAF self-spin from the compare path.
  - Compare demo active switch: `FRET_HELLO_WORLD_COMPARE_CONTINUOUS_REDRAW=1` now holds a declarative continuous-frames lease instead of manually re-requesting redraw from `render()`.
  - First proof-of-activity artifact: `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r5/summary/summary.json`.
  - Root cause of the runaway active loop: desktop `Effect::RequestAnimationFrame` requested an immediate redraw in `crates/fret-launch/src/runner/desktop/runner/effects.rs`, and `about_to_wait` requested another redraw again, so light scenes could self-spin into thousands of presents.
  - Fix: `Effect::RequestAnimationFrame` now only records RAF intent into `raf_windows`; `about_to_wait` remains the pacing point for the next redraw.
  - Post-fix cadence-matched artifact: `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r7-control8ms/summary/summary.json`.
  - Result: with the control also paced at `8ms`, present counts now line up closely at `6s` (`563` control vs `575` empty vs `577` full). The active Fret plateau remains materially higher than the same-backend control, but it is now stable rather than runaway.
- [x] Add a second active baseline with real per-frame content mutation (paint-only vs layout-affecting) so continuous-present residency can be separated from rerender/layout cost.
  - Compare demo now supports `FRET_HELLO_WORLD_COMPARE_ACTIVE_MODE=present-only|paint-model|layout-model|idle`.
  - `present-only` keeps `render_count≈2` while runner presents continue climbing; `paint/layout` now drive `render_count≈present_count≈560` via `request_animation_frame()`.
  - Final artifacts:
    - `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r13-present-final/summary/summary.json`
    - `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r14-paint-final/summary/summary.json`
    - `target/diag/wgpu-control-vs-fret-active-redraw-debug-20260307-r15-layout-final/summary/summary.json`
  - Result: the large active plateau already exists in `present-only`; real paint/layout adds only a single-digit extra cost on the full hello-world scene.
- [x] Expose effective Fret surface config and renderer target/intermediate bytes in the compare-demo internal report for active runs.
  - `hello_world_compare.internal_gpu.json` now includes `surface` and `renderer_perf` per sample.
  - `tools/run_wgpu_hello_world_control_vs_fret.py` now accepts `--fret-active-mode`, and the summarizer now surfaces active mode + effective surface info in `summary.md`.
  - Current hello-world steady samples show `present_mode=Fifo`, `desired_maximum_frame_latency=2`, `configure_count=1`, and zero renderer-owned image / render-target / intermediate bytes.
- [x] Run the first same-scene Fret-vs-GPUI active matrix on a locally materialized GPUI compare scene (debug/debug; local 2026-03-07).
  - Tooling: `tools/run_fret_vs_gpui_hello_world_compare.py`, `tools/summarize_fret_vs_gpui_hello_world_compare.py`, template `tools/external-templates/gpui_hello_world_compare.rs`.
  - Final artifacts:
    - `target/diag/fret-vs-gpui-hello-world-same-scene-debug-20260307-r2-idle/summary/summary.json`
    - `target/diag/fret-vs-gpui-hello-world-same-scene-debug-20260307-r2-rerender-only/summary/summary.json`
    - `target/diag/fret-vs-gpui-hello-world-same-scene-debug-20260307-r2-paint-model/summary/summary.json`
    - `target/diag/fret-vs-gpui-hello-world-same-scene-debug-20260307-r2-layout-model/summary/summary.json`
  - Result: idle shrinks to about `+14 MiB` physical / `+5 MiB` graphics, but cadence-aligned active rows (`rerender-only`, `layout-model`) still leave Fret about `~+99–118 MiB` physical / `~+90–107 MiB` graphics above GPUI on the same scene.
  - Caveat: GPUI `paint-model full` only reached about `221` renders by `6s` in debug, so treat that cell as under-paced; `rerender-only` and `layout-model` are the cleaner apples-to-apples rows.
- [ ] Run the same active-mode matrix against an external same-backend UI framework control (GPUI first) so Fret-vs-framework active residency can be stated apples-to-apples rather than only versus the raw `wgpu` control.

- [ ] Explain why `Game Memory` alternates between full bundles and partial `Trace1.run`-only bundles for both `wgpu_hello_world_control` and `hello_world_compare_demo`, then turn that finding into a stable same-backend control capture path.
  - Continuous control is a good stress case here: `target/diag/wgpu-control-pid-audit-continuous-20260306-r2/summary.json` still collapsed into a partial `Trace1.run`-only bundle, while the same config in `target/diag/wgpu-control-pid-audit-continuous-20260306-r3/summary.json` produced a full bundle.
- [x] Add a direct per-row parser for fixed-width stores such as `metal-io-surface-access` and `virtual-memory`, so full bundles remain analyzable even when `xctrace export` hangs.
  - Tooling: `tools/summarize_hello_world_compare_xctrace.py` now parses both stores directly from `bulkstore` and also supports `--pid-equals <pid>` for numeric pid filtering on stores that expose a `pid` column.
  - Validation: `target/diag/apple-direct-store-same-backend-20260306-r1/fret-metal-io-surface-access-pid-93488.json`, `target/diag/apple-direct-store-same-backend-20260306-r1/fret-virtual-memory.json`.
- [x] Add a launch-mode Game Memory capture path (`xctrace record --launch -- ...`) plus a compare-demo auto-exit knob so startup capture attempts are scriptable rather than hand-driven.
  - Tooling: `tools/capture_hello_world_compare_xctrace.py --record-mode launch --target-exit-after-secs <secs>`
  - Demo knob: `FRET_HELLO_WORLD_COMPARE_EXIT_AFTER_SECS`
  - Validation: `target/diag/test-hello-world-compare-auto-exit-20260306-r1.json`, `target/diag/test-hello-world-game-memory-launch-helper-20260306-r3/summary.json`
  - Current limitation: on this machine, `Game Memory` launch captures still finalize into tiny partial bundles (~`56 KiB`) with `Document Missing Template Error`, so the path is automated but not yet analytically useful.
- [ ] Explain/fix why `Game Memory` launch-mode finalization still produces empty partial bundles on this machine even when the compare demo self-exits successfully.
- [x] Add a runner-side present counter so `ca-client-present-request` can be compared directly against app `render_count` / `frame_id`.
  - New portable store: `fret_runtime::RunnerPresentDiagnosticsStore`.
  - Runner updates now land in both desktop and web present paths, and desktop close tears the per-window entry down.
  - Validation artifacts:
    - `target/diag/test-hello-world-compare-runner-present-runtime-normal-20260306-r1.json`
    - `target/diag/test-hello-world-compare-runner-present-runtime-20260306-r1.json`
  - Result: the compare demo can stay at `render_count=2` / `last_frame_id=1` while runner-side presents still climb to about `98` / `216` / `335` at `1s` / `2s` / `3s`, so `ca-client-present-request` now clearly tracks runner present cadence much more closely than declarative rerender cadence.

- Observed (surface sweep; local 2026-03-06):
  - `desired_maximum_frame_latency=1` trims about ~4 MiB of `IOSurface` / internal Metal current allocation at `500x500`, but the residual gap is effectively unchanged (~195.5 MiB full; ~183.5 MiB empty).
  - `FRET_RENDER_WGPU_PATH_MSAA_SAMPLES=1` roughly halves internal Metal current allocation at `500x500`, but it does not remove the headline graphics bucket; residual gap remains huge (~215.6 MiB full; ~209.2 MiB empty).
  - Window size strongly scales app-visible Metal + `IOSurface`, yet even `256x256` still leaves ~207.9 MiB full / ~201.3 MiB empty residual and `1000x1000` still leaves ~147.1 MiB full / ~135.2 MiB empty residual.

### Evidence (captured)

- `empty-idle-memory-steady` (macOS native)
  - Script: `tools/diag-scripts/tooling/empty/empty-idle-memory-steady.json`
  - Demo: `target/debug/empty_idle_demo` (from `apps/fret-demo`)
  - Sample run output (local): `target/fret-diag-mem-empty-idle-steady/`
  - GPU sampling (optional):
    - Run with `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`
    - Sample run output (local): `target/fret-diag-mem-empty-idle-steady-wgpu/`
  - Observed (sample):
    - `resources.process_footprint.macos_physical_footprint_bytes`: ~288 MB
    - `resources.process_footprint.macos_vmmap_top_dirty_region_type`: `owned unmapped memory` (~216 MB dirty)
    - `macos_vmmap.tables.malloc_zones.top_allocated[0]`: Default malloc zone ~24.5 MB allocated, ~15.4 MB frag
    - `resources.bundle_last_frame_stats.wgpu_metal_current_allocated_size_bytes`: ~30.7 MiB (with GPU sampling enabled)

## Optimization candidates

- [x] Run allocator A/B locally (mimalloc/jemalloc) and record impact on:
  - `resources.process_footprint.macos_owned_unmapped_memory_dirty_bytes`
  - `macos_vmmap.tables.malloc_zones.top_allocated[0]` (`allocated_bytes`, `frag_bytes`, `frag_percent`)
- Observed (empty idle, sample):
  - System vs `mimalloc`: default malloc zone allocated drops ~23.9 MB → ~7.8 MB; `owned unmapped memory` dirty unchanged (~213.6 MB).
  - System vs `jemalloc`: default malloc zone allocated drops ~23.9 MB → ~7.8 MB; `owned unmapped memory` dirty remains the headline (~216.3 MB).
- [ ] Decide whether to keep allocator selection as a dev-only feature (A/B), and whether to surface it in `fretboard dev` presets.
- [ ] Identify top heap offenders via structured `vmmap` summary and pick one bounded optimization.
- [x] Reduce baseline text atlas allocations by lazily allocating the mask atlas pages (avoid preallocating `TEXT_ATLAS_MAX_PAGES`).

## Gates

- [x] Calibrate a macOS footprint gate for `empty-idle` and `text-heavy` (repeat samples captured under `target/fret-diag-mem-*-sample5/`).
- [x] Calibrate a macOS footprint gate for `todo` and record an N=10 repeat baseline (`target/diag/mem-todo-repeat10-20260306/`).
- [x] Calibrate a macOS footprint gate for `ui-gallery-code-editor-torture` and record an N=10 repeat baseline (`target/diag/mem-ui-gallery-editor-repeat10-20260306/`).
- [x] Calibrate a Metal allocated size gate for `empty-idle` and `text-heavy` (requires `--env FRET_DIAG_WGPU_ALLOCATOR_REPORT=1`).
- [x] Add a wgpu hub counts gate (`check.wgpu_hub_counts.json`; requires `--env FRET_DIAG_WGPU_REPORT=1`).
- [x] Add a text-atlas-focused gate (`--max-render-text-atlas-bytes-live-estimate-total`) for more stable attribution vs total Metal bytes.
- [x] Calibrate a post-drop release gate for `image-heavy-memory-steady-after-drop` (avoid peak-based gates; prefer `owned_unmapped_memory` and `wgpu_metal_current_allocated_size_bytes` thresholds).
- [x] Add linear-vs-image-pressure gates for the headline buckets:
  - `--max-macos-owned-unmapped-memory-dirty-bytes-linear-vs-renderer-gpu-images`
  - `--max-wgpu-metal-current-allocated-size-bytes-linear-vs-renderer-gpu-images`
- [x] Add repeat-only distribution gates for memory keys:
  - `fretboard diag repeat ... --check-memory-p90-max <key>:<bytes>` (fails on missing keys or p90 drift)
- [ ] Document acceptable drift policy (e.g. +X MiB allowed with justification).
- [x] Add a memory-only repeat mode (`fretboard diag repeat --no-compare`) so editor-grade workloads do not fail solely because bundle contents differ across runs.
- [x] Add a first-pass `app_snapshot.shell` snapshot for the shared `ui-gallery` shell path (workspace tabs / queries / command registry/page spec metadata counts).
- [x] Add a cold-start nav sweep path (`FRET_UI_GALLERY_NAV_QUERY` + `app_snapshot.shell.nav_visible_*`) and record the first visible-item deltas on the `card` page.
- [x] Fix `fretboard diag repeat --launch` so script `meta.env_defaults` reach the launched app (prevents `ui-gallery` page-targeted memory scripts from silently falling back to `overlay`).
- [x] Backfill `script.result.json.last_bundle_dir` / `last_bundle_artifact` after async `capture_bundle`, and teach `diag repeat` to recover the run dir from `bundle_artifact` when needed, so retained-analysis repeat runs stay self-describing.
- [x] Validate an analysis-only subtree recipe for `ui-gallery` memory attribution:
  - `FRET_DIAG_DEBUG_SNAPSHOT=1`
  - `FRET_DIAG_SEMANTICS_TEST_IDS_ONLY=0`
  - `FRET_DIAG_BUNDLE_DUMP_SEMANTICS_TEST_IDS_ONLY=1`
  This keeps page selection deterministic while preserving test-id roots plus ancestors in `bundle.schema2.json`.
- [ ] Decide whether to promote dedicated analysis scripts/suites for the closure-preserving semantics dump recipe, or keep it as a documented manual override separate from raw steady-memory gates.
- [ ] Extend `app_snapshot.shell` beyond model strings into the remaining widget/runtime-owned shell allocations (shared content shell, overlay shell, command surfaces) so the corrected card-page floor (~90 MiB after `simple_content + nav none`) can be split without relying only on vmmap.
- [x] Add finer-grained content-shell diagnostics for the corrected `card` attribution path (preview sections / section counts / status-bar subtree / content header / page preview) and record the first section-level `card` sweep (`target/diag/mem-ui-gallery-card-sections-empty-r3-20260306/`, `target/diag/mem-ui-gallery-card-sections-navnone-r3-20260306/`).
- [x] Add section-level startup/bisect knobs for `ui-gallery` card preview and land the first targeted scripts/suite (`ui-gallery-card-no-{image,compositions,meeting-notes,heavy-sections}-memory-steady.json`, `tools/diag-scripts/suites/ui-gallery-card-section-memory-bisect/suite.json`).
- [x] Add second-stage card sweep scripts for the remaining preview cluster / shared docs shell (`ui-gallery-card-no-light-sections-memory-steady.json`, `ui-gallery-card-preview-only-memory-steady.json`) and record that `preview only` mainly removes allocator fragmentation (~56 MiB `macos_malloc_zones_total_frag_bytes`) rather than large live allocations.
- [x] Add combined preview-body masks (`no heavy + preview only`, `no light + preview only`) and confirm that, once code tabs are removed, the heavy/light card-preview halves each contribute about **52 MiB** of live allocations while frag stays near the same ~13–14 MiB floor.
- [x] Add `page-shell-only` / `scaffold-only` card runs and confirm that, on this branch, the stripped card doc scaffold still sits around **83.9–84.1 MiB** allocated with only `9` preview nodes, i.e. about **8 MiB** above the current `simple_content + nav none` floor.
- [x] Add card-doc scaffold counters to `app_snapshot.shell` / `memory-summary` and confirm that the attributable visible payload is tiny (full card: ~**31 KiB** code + **681 B** static text; `scaffold-only`: **0 B** text/code), so the residual is not snippet/source strings.
- [x] Add retained runtime/frame counters to diagnostics for `ui-gallery` card attribution (`ui_frame_arena_capacity_estimate_bytes`, `ui_view_cache_roots_*`, `ui_element_runtime_*` state/bounds/scratch-pool counters) so future floor runs can separate retained structure capacity from visible payload.
- [x] Cross-check the remaining `card` delta with macOS `heap -s -H` under `MallocStackLogging=1` and confirm the largest live-heap growth is in framework-side `UiTree::create_node`, `RawVec` growth, `hashbrown` tables, `WindowElementState::take_scratch_element_children_vec`, and `TextSystem::prepare_with_key`, not in raw snippet strings or font blobs.
- [x] Re-run the card floor suite with the new retained-state counters (using the explicit analysis override `FRET_DIAG_DEBUG_SNAPSHOT=1`, since the default steady-memory scripts pin it to `0`) and record which `ui_element_runtime_*` / `ui_view_cache_*` deltas track the remaining scaffold / preview floors best.
- [x] Turn the new retained-state evidence into one bounded optimization candidate: prioritize doc-shell / preview-body retained node+bounds reduction first, and treat the remaining `preview -> full` gap as a separate allocator-growth follow-up rather than a scratch-pool-only issue.
- [x] Promote a dedicated retained-analysis script/suite (`FRET_DIAG_DEBUG_SNAPSHOT=1`, closure-preserving semantics dump, `FRET_UI_GALLERY_NAV_QUERY=__none__`) so future card-floor reruns do not rely on manual overrides (`tools/diag-scripts/suites/ui-gallery-card-retained-analysis-navnone/suite.json`).
- [x] Remove the first redundant wrapper layers from the shared card doc-shell path (`apps/fret-ui-gallery/src/ui/content.rs`, `apps/fret-ui-gallery/src/ui/doc_layout.rs`) so the retained rerun starts from a smaller shell/container floor.
- [x] Re-run the retained-analysis suite after the first doc-shell wrapper reduction and confirm a modest retained-structure win (`simple -> scaffold` fell from **+56 / +112** nodes/bounds to **+51 / +102**), while noting that current-head allocator/frag drift still masks the byte-level effect.
- [x] Reduce card preview-body retained node/bounds fanout and rerun the retained-analysis suite; the shared doc-layout pass removed **27 nodes / 54 bounds** from both `preview_only` and full card, and `scaffold -> preview` now sits at **+454** nodes / **+908** bounds.
- [x] Add preview-only retained bisect scripts/suites for heavy/light and hotspot section analysis (`ui-gallery-card-preview-retained-bisect-navnone`, `ui-gallery-card-preview-retained-hotspots-navnone`) so preview-body reruns can attribute retained nodes/bounds without manual env overrides.
- [x] Split the remaining preview-only retained floor and confirm the main node/bounds hotspot order is `Compositions` (**+119 / +238**) > `Meeting Notes` (**+88 / +176**) > `Image` (**+53 / +106**).
- [x] Land a first low-risk `apps/fret-ui-gallery/src/ui/snippets/card/compositions.rs` cut (merge the two standalone border examples, trim redundant header descriptions) and confirm the isolated section contribution falls from **+119 / +238** to **+107 / +214** nodes/bounds, with preview-only total down **12 / 24**.
- [ ] Keep reducing `apps/fret-ui-gallery/src/ui/snippets/card/compositions.rs`; even after the first pass it is still the single biggest preview-body node/bounds contributor (**+107 / +214**).
- [ ] Reduce `apps/fret-ui-gallery/src/ui/snippets/card/meeting_notes.rs` next; it is the second-largest retained hotspot (**+88 / +176**) and still carries nested list/avatar scaffolding.
- [ ] Keep trimming preview-body shared wrappers / per-section tabs scaffolding; even after the shared doc-layout cut, `scaffold -> preview` remains the largest retained step by a wide margin (**+454** nodes / **+908** bounds).
- [ ] Explain the remaining overlap with allocator-focused evidence in the docs/diag output itself (for example, promote `malloc_zones_total_allocated_bytes` + `malloc_zones_total_frag_bytes` deltas for card bisects instead of relying only on `MALLOC_SMALL`).
- [x] Add a lightweight bisect suite for `ui_gallery` memory (`tools/diag-scripts/suites/ui-gallery-memory-bisect/suite.json`) and record initial p90 deltas.
