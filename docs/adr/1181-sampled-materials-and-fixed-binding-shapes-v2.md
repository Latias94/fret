# ADR 1181: Sampled Materials and Fixed Binding Shapes (v2)

Status: Proposed

## Context

ADR 1174 intentionally restricts Tier B materials to “params-only” procedural paints (no
texture/sampler bindings). This is the correct v1 constraint to keep bind group shapes stable and
avoid portability drift.

However, several MagicUI-class and design-system patterns become much easier (and cheaper) if Tier
 B materials can optionally sample **one** resource in a controlled way.

There are two distinct pressures:

1) **Renderer-owned “catalog textures”** (recommended v2 first step):
   - sampling a small, baked texture shipped with the renderer (blue-noise, Bayer dither matrices,
     tiny LUTs) for higher quality patterns and portable stylization, without exposing arbitrary
     resources to components.

2) **App-provided images** (deferred to a later revision):
   - sampling a user-provided image as a stylized fill or mask, still within a fixed binding shape,
     without forcing Tier A pipelines by default.

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
- `BindingShape::ParamsPlusCatalogTexture` (v2; `MaterialParams` + one renderer-owned catalog texture + one sampler)

No other shapes are allowed in v2.

### D1.1 — Catalog textures are renderer-owned and fixed (v2)

`BindingShape::ParamsPlusCatalogTexture` binds exactly one texture selected from a small,
framework-controlled catalog. The catalog is renderer-owned:

- components and ecosystem crates never provide raw handles, shader code, or arbitrary texture
  bytes,
- the catalog list is small, portable, and capability-gated,
- the binding shape is stable (WebGPU-friendly) and budgetable.

The catalog selection is part of the **material descriptor** (registration-time), not the per-draw
instance payload. This avoids expanding `Paint::Material { id, params }` for v2 and keeps scene
recording portable and minimal.

Example (non-normative) catalog kinds:

- `BlueNoise64x64R8`
- `Bayer8x8R8`
- `DitherLutR8`

### D1.2 — App-provided images are deferred (v3+)

Sampling arbitrary app-provided resources (e.g. `ImageId`) is explicitly deferred until a later
revision, because it typically forces a global “image table / texture atlas” binding model in the
quad/material pass and is high risk for portability and complexity.

A future extension may add:

- `BindingShape::ParamsPlusImage` (v3+; `MaterialParams` + one `ImageId` + one sampler)

but that is not part of v2 acceptance criteria.

### D2 — Instance payload remains fixed-size and portable

For v2 (`ParamsPlusCatalogTexture`), the instance remains:

- `params: MaterialParams` (64B; ADR 1174).

No additional per-instance resource handles are introduced in v2.

### D3 — Capability gating and deterministic fallback are required

Backends may not support catalog textures (or may restrict formats/sampling).

Rules:

- Registration of a `BindingShape::ParamsPlusCatalogTexture` material MUST be capability-gated (ADR 0124).
- If unsupported, registration fails deterministically and callers must select a fallback.

### D4 — Budgets and observability

Sampled materials MUST be accounted for in renderer telemetry:

- number of sampled-material draws,
- distinct sampled materials used,
- catalog-texture usage counts (best-effort).

If a backend must degrade (e.g. disallow linear filtering, clamp UVs, or reduce precision), it MUST
do so deterministically and report the degradation path (ADR 0120 / ADR 0036).

## Consequences

- Enables a controlled expansion of the creative surface area without opening “arbitrary shader”
  extensibility.
- Keeps the binding shape stable, which is critical for batching and WebGPU portability.

## Non-goals

- Multiple textures/samplers, bindless-like designs, or arbitrary resource graphs.
- Exposing backend handles or WGSL to component crates.

## Validation / Acceptance criteria (v2)

This ADR is considered conformant when:

- a renderer can register a `ParamsPlusCatalogTexture` material on capable backends,
- at least one baseline material kind can use the catalog texture in its shader evaluation,
- deterministic fallbacks exist when unsupported,
- a small conformance test renders a sampled material and verifies sampled pixels,
- perf snapshots can report sampled-material usage at least at the “draw op counts” level.
