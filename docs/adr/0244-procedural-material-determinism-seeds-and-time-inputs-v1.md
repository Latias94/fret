# ADR 0244: Procedural Material Determinism (Seeds and Time Inputs) (v1)

Status: Accepted

## Context

Procedural paints (dot grids, stripes, noise, beams, sparkles) are a key target for Tier B
materials (ADR 0235). To be viable for an ecosystem:

- they must be deterministic across runs for diagnostics and tests,
- they must not “secretly animate” using hidden time sources,
- they must provide a consistent, portable way to opt into animation while respecting reduced
  motion (ADR 0232 / ADR 0240).

If determinism and time inputs are not locked, component ecosystems will diverge:

- different crates pick different RNG seeds and time bases,
- reduced-motion handling becomes inconsistent,
- screenshot/perf gates become flaky.

Related ADRs:

- Controlled materials registry: `docs/adr/0235-controlled-materials-registry-and-procedural-paints-v1.md`
- Frame clock + reduced motion: `docs/adr/0240-frame-clock-and-reduced-motion-gates-v1.md`
- Environment queries: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`

## Decision

### D1 — No hidden time dependency in Tier B materials

Tier B materials MUST be pure functions of:

- per-draw inputs (geometry, UVs, etc.),
- and the explicit `MaterialParams` payload.

Materials MUST NOT read an implicit “global time” value.

If a material is intended to animate, the caller must pass an explicit time/phase parameter in
`MaterialParams`.

### D2 — Seeds are explicit and stable by default

All procedural materials that use randomness MUST take an explicit seed.

V1 rule:

- The seed is provided by the caller via `MaterialParams`.
- If the caller does not care, ecosystem helpers SHOULD derive a stable default seed from stable
  element identity (e.g. a keyed element id or a recipe-provided seed), not from frame counters.

### D3 — Reduced motion gating is a policy seam, but inputs must support it

Reduced motion remains policy-owned (ADR 0232), but Tier B materials must allow policy to disable
animation deterministically.

Recommended pattern:

- Recipes derive `t` from the frame clock (ADR 0240) when motion is allowed.
- Under reduced motion, recipes pin `t = 0` (or another stable constant) and avoid requesting
  continuous frames.

### D4 — Baseline parameter slots (recommended, not required)

To reduce ecosystem drift for common materials, v1 recommends reserving parameter meanings for a
baseline “procedural recipe” convention:

- `params[0]`: primary scale/size controls (logical px)
- `params[1]`: secondary controls (thickness, softness, intensity)
- `params[2]`: `seed` and other randomness controls
- `params[3]`: animation controls (`t` / phase), explicitly provided by the caller

Exact field meanings remain material-kind-specific, but recipes should follow this convention so
shared tooling (diag overlays, inspectors) can present reasonable labels.

## Consequences

- Diagnostics and scripted tests remain stable: procedural looks can be reproduced by replaying
  the same params and seed.
- Reduced motion handling becomes consistent: time is explicit and can be pinned.

## Non-goals

- This ADR does not standardize a full procedural language or node graph.
- This ADR does not guarantee cross-GPU bit-identical noise; it guarantees input determinism and
  policy seams for stable behavior.
