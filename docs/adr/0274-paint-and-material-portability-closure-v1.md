---
title: Paint and Material Portability Closure v1
status: Draft
date: 2026-02-14
---

# ADR 0274: Paint and Material Portability Closure v1

## Context

Fret already defines:

- a portable `Paint` vocabulary (solids + gradients + `Paint::Material`) (ADR 0233),
- a controlled Tier B material registry (`MaterialId`, `MaterialKind`, fixed binding shapes) (ADR 0235 / ADR 0242),
- determinism rules for procedural materials (explicit seeds/time; no hidden clocks) (ADR 0244),
- budgets + deterministic degradation for multi-pass features (ADR 0118),
- and capability gating guidance (ADR 0122).

As the renderer is refactored internally (RenderPlan compilation, caching, pass scheduling), the
largest portability risk is not “missing features” but **undefined drift**:

- “works on desktop” but silently differs on wasm/mobile,
- capability failures becoming runtime surprises (or worse: non-deterministic),
- ecosystem crates depending on backend quirks.

This ADR closes the loop by making paint/material portability obligations explicit and testable.

## Decision

### D1 — Paint inputs are sanitized deterministically at the contract boundary

Paint sources must be sanitized deterministically (as value semantics), including:

- non-finite floats (treated as transparent),
- unsupported `TileMode` / `ColorSpace` (deterministically degraded),
- and degenerate gradient stops (deterministically normalized).

This is already required by ADR 0233; this ADR makes it an explicit portability gate for wasm/mobile.

### D2 — Tier B materials are capability-gated at registration time, with deterministic fallbacks

Rules:

- Registering an unsupported material (kind or binding shape) must fail deterministically.
- Missing/unknown `MaterialId` at draw time must fall back deterministically to a safe paint.
- Fallbacks must be observable via telemetry/debug snapshots.

V1 baseline (normative):

- Unknown `MaterialId` is treated as `Paint::Solid(Color::TRANSPARENT)` for the affected draw.
- Unsupported registration returns `MaterialRegistrationError::Unsupported`.

Notes:

- This mirrors the intent of ADR 0235 / ADR 0242 and makes it mandatory for portability.

### D3 — Binding shapes remain fixed and versioned

Binding shapes are versioned and fixed (ADR 0242). Backends that cannot support a binding shape
MUST fail registration deterministically and callers MUST select an alternative.

This ADR explicitly rejects “best-effort silently ignore the sampled texture” as a fallback because
it creates hard-to-debug drift across targets.

### D4 — Determinism is required (no hidden time)

Tier B materials MUST remain pure functions of:

- geometry / local coordinates,
- and explicit `MaterialParams`.

They MUST NOT read implicit “global time”.

This is already required by ADR 0244; this ADR promotes it to a portability gate.

### D5 — Capability reporting is required and must be stable for a renderer session

The renderer must expose a stable capability snapshot that includes, at minimum:

- supported material binding shapes,
- which material kinds are supported,
- and any relevant constraints (e.g. sampler restrictions).

This is aligned with ADR 0122, but this ADR requires that the snapshot be available for:

- diagnostics bundles/perf snapshots,
- and conformance harnesses.

### D6 — Conformance gates are required for any contract expansion

If the public scene contract expands in a way that affects paint/material behavior (examples:
`Path` accepting `Paint`, new mask sources using image sampling, new material binding shapes),
the change MUST ship with:

- at least one renderer conformance test (GPU readback when feasible),
- and a deterministic fallback story under missing capabilities and under budget pressure.

## Consequences

- wasm/mobile portability becomes a first-class constraint rather than an “after the fact” port.
- Ecosystem crates can rely on stable material behavior (or explicit registration failures), which
  prevents silent drift.
- Renderer internal refactors can proceed fearlessly because conformance + telemetry gates are
  contractually required for expansions.

## Acceptance criteria (recommended, not exhaustive)

This ADR is considered “aligned” when:

- The renderer can deterministically handle:
  - unsupported material registration,
  - unknown/unregistered `MaterialId`,
  - and non-finite `MaterialParams` (sanitized),
- Telemetry/debug snapshots can report:
  - material draws (count),
  - distinct materials used,
  - and capability-driven fallbacks,
- Conformance tests exist for:
  - `Paint` gradients sanitization behavior,
  - `Paint::Material` baseline rendering,
  - sampled material binding shapes (catalog textures) where supported.

Suggested evidence anchors (existing tests):

- `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs`
- `crates/fret-render-wgpu/tests/materials_conformance.rs`
- `crates/fret-render-wgpu/tests/materials_sampled_conformance.rs`

## Related

- Paint primitives: `docs/adr/0233-paint-primitives-brushes-and-gradients-v1.md`
- Tier B materials: `docs/adr/0235-controlled-materials-registry-and-procedural-paints-v1.md`
- Sampled materials: `docs/adr/0242-sampled-materials-and-fixed-binding-shapes-v2.md`
- Material determinism: `docs/adr/0244-procedural-material-determinism-seeds-and-time-inputs-v1.md`
- Budgets + degradation: `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Capabilities: `docs/adr/0122-renderer-capabilities-and-optional-zero-copy-imports.md`

