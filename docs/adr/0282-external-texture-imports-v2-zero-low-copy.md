---
title: External Texture Imports v2 (Zero/Low-Copy, Capability-Gated)
status: Draft
date: 2026-02-16
---

# ADR 0282: External Texture Imports v2 (Zero/Low-Copy, Capability-Gated)

## Context

ADR 0234 established a staged approach for “external texture imports”:

1. A **contract path** that runs end-to-end in-repo using `RenderTargetId` +
   `SceneOp::ViewportSurface`, with per-frame `TextureView` refresh via runner-applied deltas.
2. A **true external import** path (optional, capability-gated) that avoids leaking backend handles
   into UI/component code.

The v1 copy-based paths are sufficient for static or low-frequency images. However, for high
frequency + high resolution sources (video/camera/remote desktop/large canvases), “copy every
frame into a renderer-owned texture” becomes a primary frame-budget risk:

- bandwidth and GPU time (especially at 1080p/4K @ 60/120Hz),
- latency and jitter (extra sync points / queuing),
- memory and power (mobile constraints),
- and semantic correctness (colorspace/orientation/alpha metadata needs a single authoritative
  place to live).

At the same time, WebGPU and mobile GPUs have strict portability constraints. Many “zero-copy”
mechanisms are only available behind capabilities and require deterministic fallback.

## Decision

Define a v2 policy for external texture imports that raises the performance ceiling while keeping
the public UI contract stable and portable:

1. **UI-facing contract remains unchanged**:
   - UI/component code continues to use `RenderTargetId` + `SceneOp::ViewportSurface`.
   - No backend handles (wgpu/WebGPU/Vulkan/Metal) are exposed to `fret-ui` or ecosystem code.

2. **Import strategy is capability-gated and bounded**:
   - The runner/renderer selects an import strategy from a small, ordered set.
   - Unsupported strategies must fall back deterministically to a copy-based path.

3. **Metadata is first-class and authoritative**:
   - colorspace / transfer / matrix hints,
   - orientation / transform hints,
   - alpha semantics (premul vs straight),
   - and timestamp hints (diagnostics only),
   must travel with the imported target so zero/low-copy and copy paths converge to the same
   observable output.

4. **Perf is gated**:
   - existing copy-path perf baselines remain non-regression anchors,
   - v2 introduces steady-state perf baselines for any landed zero/low-copy strategy, especially
     on wasm/mobile.

This ADR is tracked as a workstream:

- `docs/workstreams/external-texture-imports-v2-zero-low-copy.md`

## Non-goals (v2)

- Exposing backend handles to UI/component code.
- A fully general color management pipeline (ICC profiles, arbitrary transfer functions).
- Shipping “true zero-copy” on every backend immediately (capability-gated + staged).

## Import strategy set (bounded)

This ADR defines a bounded set of strategies. Concrete backend implementations may add internal
details, but must map into one of these categories:

1. **Zero-copy (external texture sampling)**
   - Example: WebGPU `ExternalTexture` sampling for WebCodecs `VideoFrame`.
   - Requires explicit capability gating and a deterministic fallback.

2. **Low-copy (GPU-only copy / blit)**
   - Example: `Queue::copy_external_image_to_texture` (web) or GPU-to-GPU copies on native.
   - This is the default portable “fast path” when true zero-copy is unavailable.

3. **CPU upload**
   - Example: decoded bytes uploaded via `Queue::write_texture`.
   - Always available as the final deterministic fallback.

The renderer must never silently choose an “unknown best effort” path. The selected strategy must
be observable in diagnostics/perf snapshots.

## Deterministic fallback order

For a given target/backend, the effective strategy is selected by a deterministic ordered chain:

1. Prefer **zero-copy** if the backend reports support and the source provides an eligible frame.
2. Else prefer **low-copy GPU copy**.
3. Else fall back to **CPU upload**.

This chain must be stable across machines for the same capability snapshot, and must not depend
on timing or opportunistic resource availability.

## Semantics: metadata and correctness

Imported render targets must behave identically (as observed by UI sampling) regardless of the
ingestion strategy:

- alpha semantics must be applied consistently (straight vs premul),
- orientation/transform hints must map to stable sampling transforms,
- and colorspace hints must not be “lost” when using copy-based fallbacks.

If a backend cannot preserve a metadata property for a given strategy (e.g. colorspace metadata is
not representable), it must degrade deterministically and record a diagnostic counter.

### Metadata field rules (v2 scope, executable)

This ADR intentionally keeps the portable metadata surface **small**. The “executable” part of v2
is that *every* field we do carry has a clear preserve/degrade rule, and any non-preservable field
becomes observable via a counter/hint (workstream TODOs).

**RenderTargetDescriptor**

- `color_space: RenderTargetColorSpace` (`srgb|linear`)
  - Preserve across all strategies.
  - If a backend cannot represent the requested value, degrade deterministically to `srgb` and
    record a counter/hint (see workstream `EXTV2-diag-040`).

**RenderTargetMetadata**

- `alpha_mode: RenderTargetAlphaMode` (`premultiplied|straight`)
  - Preserve across all strategies.
  - If a producer cannot provide the declared alpha mode for a given frame, it must not “lie”:
    it should switch to the correct declared mode for that frame (or fall back to a copy path that
    can normalize alpha), and record a counter/hint if a requested mode could not be honored.
- `orientation: RenderTargetOrientation` (`rotation` + `mirror_x`)
  - Preserve across all strategies.
  - If a backend cannot apply the requested orientation, degrade deterministically to the identity
    orientation (`r0`, `mirror_x=false`) and record a counter/hint.
- `color_encoding: RenderTargetColorEncoding` (bounded colorimetry hints)
  - Best-effort hints for real media sources (video/camera/remote desktop):
    - primaries, transfer function, matrix coefficients, and range.
  - Preserve across all strategies when representable.
  - If a backend cannot preserve the effective values for a strategy, degrade deterministically to
    `unknown` values and record a counter/hint (see workstream `EXTV2-diag-040`).
- `requested_ingest_strategy` vs `ingest_strategy`
  - Always populate both (or keep `unknown`) so capability-gated fallbacks are observable in perf
    snapshots/bundles.
- `frame_timestamp_ns`
  - Diagnostics-only. If not available, set `None`. No correctness semantics depend on it.

**Explicit deferral (non-goal):** full color management (ICC profiles, HDR tone mapping, arbitrary
transfer functions) remains out of scope for v2.

## Capability matrix (expected reality)

This ADR assumes the following *typical* capability picture. Implementations must treat this as
capability-gated (query, do not assume).

- Web (wasm + wgpu WebGPU backend):
  - `ExternalZeroCopy`: **blocked** until backend support exists for WebGPU `ExternalTexture`.
  - `GpuCopy`: available today (`copyExternalImageToTexture` path) and is the portable “fast” path.
  - `CpuUpload`: available as the final fallback (but avoid for high-frequency sources).
- Native desktop (wgpu, platform GPU):
  - `GpuCopy`: available and should be the default non-regression anchor.
  - `ExternalZeroCopy`: may be possible only for specific producer integrations; treat as optional.
- Mobile (iOS/Android):
  - Strategy availability depends heavily on the backend + OS primitives; assume “copy-first” until
    proven and gated by real device perf baselines.

## Perf gates (must be explicit)

For each landed v2 strategy, add:

1. A deterministic steady-state `fretboard diag perf` script (or reuse an existing one).
2. A committed baseline JSON under `docs/workstreams/perf-baselines/`.
3. A rule for whether ingest fallbacks are expected on the target (and how they are asserted).

Existing v1 anchors (copy paths):

- Web copy path:
  - `tools/diag-scripts/external-texture-imports-web-copy-perf-steady.json`
  - `docs/workstreams/perf-baselines/external-texture-imports-web-copy.web-local.v1.json`
- Native contract-path:
  - `tools/diag-scripts/external-texture-imports-contract-path-perf-steady.json`
  - `docs/workstreams/perf-baselines/external-texture-imports-contract-path.windows-local.v1.json`

## Draft exit criteria (before marking Accepted)

This ADR can move out of Draft once the following are true:

1. The strategy selection is **fully deterministic** and bounded:
   - for each target class (web/wasm, native desktop, mobile), the ordered fallback chain is
     explicit and does not depend on timing or opportunistic resource availability.
2. The metadata semantics are **complete enough for correctness reasoning**:
   - alpha mode + orientation must be explicitly preserved across all strategies,
   - and any non-preservable metadata must have a deterministic “drop/approximate” rule plus an
     explicit counter/hint.
3. The perf gates are **actionable**:
   - at least one steady-state baseline exists per landed strategy,
   - and the “requested vs effective” ingest attribution is validated in the baseline notes.
4. The web zero-copy status is **truthful and capability-gated**:
   - it remains explicitly blocked until backend support exists, with the copy-path baselines kept
     green as anchors.

## Consequences

- The framework gains a clear, portable, performance-oriented story for video/camera/streaming UI.
- Implementation complexity is isolated to runner/renderer layers; `fret-ui` remains a mechanism
  contract surface.
- Web zero-copy remains explicitly capability-gated and may remain blocked until upstream backend
  support exists; the copy paths remain first-class and gated by perf baselines.

## Evidence / implementation anchors

- v1 contract + staging: `docs/adr/0234-imported-render-targets-and-external-texture-imports-v1.md`
- Workstream tracking:
  - `docs/workstreams/external-texture-imports-v1.md`
  - `docs/workstreams/external-texture-imports-v1-todo.md`
  - `docs/workstreams/external-texture-imports-v1-milestones.md`
- Capability surface (today): `crates/fret-render-wgpu/src/capabilities.rs`
- Native adapter seam (today): `crates/fret-launch/src/runner/native_external_import.rs`
- v2 workstream tracking:
  - `docs/workstreams/external-texture-imports-v2-zero-low-copy.md`
  - `docs/workstreams/external-texture-imports-v2-zero-low-copy-todo.md`
  - `docs/workstreams/external-texture-imports-v2-zero-low-copy-milestones.md`
