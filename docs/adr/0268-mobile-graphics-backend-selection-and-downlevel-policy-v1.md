# ADR 0268: Mobile Graphics Backend Selection and Downlevel Policy (v1)

Status: Proposed

## Context

Fret is a modular, GPU-first UI framework targeting desktop-first + wasm today, with long-term
mobile support. Mobile bring-up is rewrite-prone unless we lock the “hard-to-change” runner/graphics
assumptions early (ADR 0262).

In practice, Android/iOS environments vary significantly:

- Android devices: typically Vulkan on real hardware (Adreno/Mali), but driver quality varies.
- Android emulators: often run “Vulkan” through GFXStream/SwiftShader, and may be unstable for
  certain `wgpu`/Vulkan paths (including crashes during early renderer initialization).
- GLES/OpenGL backends can exist, but may be downlevel and can fail device creation due to limits.

If the framework does not define a stable policy for:

- which backend we target by default on mobile,
- how (and when) we allow fallback/downlevel,
- and how we make backend choice observable/diagnosable,

then higher layers will accumulate ad-hoc workarounds that are hard to unwind later.

This ADR defines the v1 policy for mobile graphics backend selection and downlevel behavior.

## Goals

1. Define a deterministic backend selection policy for Android/iOS runners.
2. Define explicit override knobs for developers and CI.
3. Define when downlevel/fallback is allowed vs when we fail fast.
4. Make backend selection and GPU capabilities observable in logs/diagnostics.
5. Keep `crates/fret-ui` mechanism-only (ADR 0066): no backend-specific UI behavior.

## Non-goals (v1)

- Guaranteeing the Android emulator is a stable rendering target across host GPUs/SDK versions.
- Supporting every `wgpu` backend equally on mobile on day one.
- Specifying a full “GPU compatibility matrix” for all Android OEM drivers.

## Decision

### D1 — Default policy: mobile is Vulkan/Metal-first

For mobile “first-class” targets:

- iOS: Metal (via `wgpu`).
- Android: Vulkan (via `wgpu`).

Rationale: editor-grade UI needs predictable performance characteristics; Vulkan/Metal are the
primary long-term backends for modern mobile GPUs.

### D2 — Emulators are best-effort, not an acceptance gate (v1)

Android emulator rendering is **best-effort**:

- We MAY support an emulator-friendly backend when it is reliable.
- We MUST NOT treat the emulator as the primary acceptance gate for GPU correctness/perf.

Instead:

- “Runs on a real device” is the MVP acceptance gate for mobile bring-up (workstream evidence).

### D3 — Override knobs are required and must be explicit

Runners MUST allow developers/CI to override backend choice explicitly:

- Environment variable override (e.g. `FRET_WGPU_BACKEND` parsed by the renderer layer).
- Runner config override (future: a structured `WgpuInitPolicy` / `RendererBackendPolicy`).

Override rules:

- Explicit user override wins over automatic selection.
- If the override fails to create a device/surface, the runner MUST log a clear error and fail
  fast (unless the user opted into fallback explicitly).

### D4 — Downlevel/fallback is opt-in and scoped to developer convenience

Downlevel/fallback (e.g. trying GLES/GL after Vulkan fails) is allowed only if:

- it is explicitly enabled by a developer configuration knob (future),
- and the runner can provide a clear diagnostic trail showing:
  - which backends were attempted,
  - why each attempt failed,
  - and which limits/features are in effect.

In CI and release builds, the default posture SHOULD be “fail fast” on backend init failure
instead of silently falling back.

### D5 — Diagnostics: backend and adapter info must be recorded

The runner MUST log, at minimum:

- selected backend,
- adapter name/vendor/device,
- driver/driver_info,
- and whether the choice was explicit override vs automatic.

When diagnostics are enabled (diag bundles), the selected backend + adapter info SHOULD be captured
as part of the bundle metadata so failures are reviewable without reproducing locally.

## Consequences

- “Mobile support” can advance without being blocked by the Android emulator’s GPU stack quirks.
- Developers have explicit controls for backend selection, enabling systematic triage.
- Higher-level UI code remains backend-agnostic.

## Implementation notes / evidence anchors

Bring-up workstream:

- `docs/workstreams/mobile-bringup-v1.md`
- `docs/workstreams/mobile-bringup-v1-device-packaging.md`

Expected implementation loci (not exhaustive):

- runner init + lifecycle gating: `crates/fret-launch`
- backend parsing and context creation: `crates/fret-render-wgpu`

