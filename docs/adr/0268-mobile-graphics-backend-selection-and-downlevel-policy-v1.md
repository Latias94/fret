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

### D1 — Default policy: prefer platform-native backends (capability-first)

For mobile “first-class” targets:

- iOS: prefer Metal (via `wgpu`).
- Android: prefer Vulkan (via `wgpu`).

Rationale:

- iOS is effectively “Metal-native” for modern rendering, and `wgpu`’s Metal backend is the
  expected first-class path.
- On Android, Vulkan is typically the most capable and predictable backend on real hardware, and
  is the best match for editor-grade UI workloads.

Important: the long-lived contract is not “always Vulkan” vs “always Metal”, but rather:

- the backend selection policy is deterministic and overrideable,
- the policy is observable in logs/diagnostics,
- and the selected adapter satisfies Fret’s required renderer capability gate (D6).

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

- it is explicitly enabled by a developer configuration knob (v1: env, debug-only; future: structured runner config),
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

### D6 — Minimum renderer capability gate (downlevel flags)

Fret’s default wgpu renderer relies on storage buffers in vertex shaders (e.g. per-quad instance
data). Therefore, the selected adapter MUST satisfy:

- `wgpu::DownlevelFlags::VERTEX_STORAGE`

If the adapter does not meet this minimum, initialization MUST fail fast with a clear error (even
if the backend “creates a device”), because later validation panics would otherwise occur during
pipeline creation.

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

## Implementation status (current)

As of 2026-02-12:

Implemented (evidence anchors):

- Default backend policy:
  - Android defaults to Vulkan-only.
  - iOS defaults to Metal-only.
  - Other targets default to `wgpu::Backends::PRIMARY`.
  - Code: `crates/fret-render-wgpu/src/lib.rs` (`default_wgpu_backends_for_target`)
- Explicit override knob:
  - `FRET_WGPU_BACKEND` (parsed in `fret-render-wgpu`).
  - Invalid overrides fail fast.
  - Code: `crates/fret-render-wgpu/src/lib.rs` (`backend_override_from_env`)
- Opt-in fallback (dev only):
  - `FRET_WGPU_ALLOW_FALLBACK=1` enables additional backend attempts in debug builds.
  - Release builds remain fail-fast.
  - Code: `crates/fret-render-wgpu/src/lib.rs` (`allow_fallback_from_env`)
- Downlevel capability gate:
  - Enforces `DownlevelFlags::VERTEX_STORAGE` at init time.
  - Code: `crates/fret-render-wgpu/src/lib.rs` (`validate_adapter`)
- Diagnostics capture:
  - Adapter selection snapshot includes the selected backend + adapter info and init attempt
    history.
  - Code: `crates/fret-render-wgpu/src/lib.rs` (`WgpuAdapterSelectionSnapshot`)
  - Bundle: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
