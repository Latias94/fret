Status: Draft (workstream tracker)

This workstream defines and lands the **v2 ceiling** for external texture imports: a bounded,
capability-gated **zero/low-copy** ingestion path for high-frequency sources (video/camera/remote
desktop/large canvases) that must remain portable to wasm/WebGPU and mobile GPUs.

This is the “execution companion” to:

- ADR 0282: `docs/adr/0282-external-texture-imports-v2-zero-low-copy.md`

It builds on v1’s contract-path closure:

- ADR 0234: `docs/adr/0234-imported-render-targets-and-external-texture-imports-v1.md`
- Workstream (v1): `docs/workstreams/external-texture-imports-v1.md`

## Core constraints (non-negotiable)

1. **UI contract stays stable**
   - UI/component code consumes `RenderTargetId` + `SceneOp::ViewportSurface` only.
   - No backend handles (wgpu/WebGPU/Vulkan/Metal) leak into `fret-ui` or ecosystem code.

2. **Bounded strategy set + deterministic fallback**
   - The effective ingest strategy is selected from a small, ordered set.
   - Unsupported strategies fall back deterministically (no “best effort”).

3. **Correctness is metadata-driven**
   - Alpha semantics + orientation/transform hints must be applied consistently across strategies.
   - Any metadata that cannot be preserved for a strategy must degrade deterministically and be
     observable via counters/hints.

4. **Perf is gated**
   - v1 copy-path baselines remain non-regression anchors.
   - v2 adds steady-state baselines for each landed zero/low-copy strategy, especially on
     wasm/mobile.

## Tracking

- TODOs: `docs/workstreams/external-texture-imports-v2-zero-low-copy-todo.md`
- Milestones: `docs/workstreams/external-texture-imports-v2-zero-low-copy-milestones.md`

## Recommended execution order

1. Lock the bounded strategy set + metadata semantics (ADR 0282 exit criteria).
2. Land metadata additions (if required) in `fret-render-core`, with deterministic degradation.
3. Land native/mobile low-copy improvements behind capabilities + counters.
4. Keep web zero-copy explicitly blocked until the backend supports it; keep copy-path perf baselines green.

## Web DevTools WS notes (practical)

- Scripted diagnostics over DevTools WS still require the app to be **actively rendering** so inbound
  WS messages are processed deterministically.
- Browsers may throttle timers and `requestAnimationFrame` when the tab is backgrounded; keep the
  demo tab visible during `diag perf` runs.
- `--perf-baseline` expects a **JSON file path**, not a directory. If you keep baselines under
  `docs/workstreams/perf-baselines/`, pass the full file name.
- If multiple sessions exist, pass `--devtools-session-id <id>` (list via
  `cargo run -p fret-diag-export -- --list-sessions --token <token>`).
