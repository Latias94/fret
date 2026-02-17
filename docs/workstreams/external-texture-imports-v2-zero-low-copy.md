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

## Current status (practical)

- M0 complete (ADR 0282 locked to executable detail).
- M1 complete (portable metadata closure: bounded color encoding hints + deterministic degradation counters).
- M2A complete (Windows MF real source wired end-to-end as `CpuUpload`, with steady perf + correctness gates).
- M2B feasibility spike concluded:
  - “Wrap/import a foreign platform texture handle into `wgpu::Texture`” is currently blocked by upstream APIs
    (wgpu 28). The workstream records this explicitly and treats it as capability-gated + revisit-later.
  - Native “no-copy” uplift remains viable via **shared allocation**: the runner/renderer allocates a
    `wgpu::Texture` and hands its native handle to the producer/decoder to write into. This typically classifies as
    `Owned` in the bounded strategy set.
- Shared allocation proof path landed (DX12-only):
  - A synthetic native writer clears a renderer-owned `wgpu::Texture` via the DX12 queue with deterministic state
    transitions, and a minimal diag correctness gate exists.
- Next up (native uplift, practical):
  - Factor the deterministic fallback chain into a single helper in `fret-launch` (so demos/callers don’t drift).
  - Pick the first real native producer that can write into a shared allocation (per-platform, capability-gated).

## Recommended execution order

1. Lock the bounded strategy set + metadata semantics (ADR 0282 exit criteria).
2. Land metadata additions (if required) in `fret-render-core`, with deterministic degradation.
3. Land native/mobile low-copy improvements behind capabilities + counters.
   - Recommended staging:
     - M2A: wire a real frame source end-to-end (can start as `CpuUpload`/`GpuCopy`).
     - M2B: add a true zero/low-copy fast path behind explicit capabilities (e.g. Windows D3D12),
       or satisfy “no-copy” via shared allocation when external-handle import is blocked.
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
  - Note: web sessions may change across reloads; re-run `--list-sessions` if tooling reports an
    unknown session id.
