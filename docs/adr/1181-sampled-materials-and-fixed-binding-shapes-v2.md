# ADR 1181: Sampled Materials and Fixed Binding Shapes (v2)

Status: Proposed

## Context

ADR 1174 intentionally restricts Tier B materials to “params-only” procedural paints (no
texture/sampler bindings). This is the correct v1 constraint to keep bind group shapes stable and
avoid portability drift.

However, several MagicUI-class and design-system patterns become much easier (and cheaper) if Tier
B materials can optionally sample **one** resource in a controlled way:

- sampling an image-based mask/alpha texture,
- sampling a small LUT/noise texture (prebaked blue-noise, dither matrices),
- sampling a user-provided image for stylized fills (still within a fixed binding shape).

If we don’t define a controlled “sampled material” extension point, ecosystems will route these
effects through Tier A pipelines by default, increasing complexity and integration burden.

Related ADRs:

- Controlled materials v1: `docs/adr/1174-controlled-materials-registry-and-procedural-paints-v1.md`
- Paint primitives: `docs/adr/1172-paint-primitives-brushes-and-gradients-v1.md`
- Renderer capabilities: `docs/adr/0124-renderer-capabilities-and-optional-zero-copy-imports.md`
- Budgets + deterministic degradation: `docs/adr/0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Trust model / tiers: `docs/adr/0125-renderer-extensibility-materials-effects-and-sandboxing-v1.md`

## Decision

### D1 — Materials may opt into one of a small set of fixed binding shapes

V2 introduces explicit material binding shapes, fixed and versioned:

- `BindingShape::ParamsOnly` (v1; `MaterialParams` only)
- `BindingShape::ParamsPlusImage` (v2; `MaterialParams` + one `ImageId` + one sampler)

No other shapes are allowed in v2.

### D2 — Instance payload remains fixed-size and portable

For `ParamsPlusImage`, the instance includes:

- `params: MaterialParams` (64B; ADR 1174),
- `image: ImageId` (portable resource handle),
- `uv: UvRect` (explicit sampling region; no implicit object-fit),
- `opacity: f32` (optional; may be folded into params if needed).

The referenced `ImageId` must obey the existing image contracts (resource handle lifetimes, upload
effects, metadata seam).

### D3 — Capability gating and deterministic fallback are required

Backends may not support sampled materials (or may restrict formats/sampling).

Rules:

- Registration of a `BindingShape::ParamsPlusImage` material MUST be capability-gated (ADR 0124).
- If unsupported, registration fails deterministically and callers must select a fallback.
- If a sampled material instance references an invalid `ImageId`, the renderer MUST fall back
  deterministically to a safe default paint (typically transparent or solid).

### D4 — Budgets and observability

Sampled materials MUST be accounted for in renderer telemetry:

- number of sampled-material draws,
- distinct sampled materials used,
- distinct images sampled by materials (best-effort).

If a backend must degrade (e.g. disallow linear filtering, clamp UVs, or reduce precision), it MUST
do so deterministically and report the degradation path (ADR 0120 / ADR 0036).

## Consequences

- Enables a controlled expansion of the creative surface area without opening “arbitrary shader”
  extensibility.
- Keeps the binding shape stable, which is critical for batching and WebGPU portability.

## Non-goals

- Multiple textures/samplers, bindless-like designs, or arbitrary resource graphs.
- Exposing backend handles or WGSL to component crates.

