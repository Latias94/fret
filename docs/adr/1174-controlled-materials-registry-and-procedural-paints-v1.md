# ADR 1174: Controlled Materials Registry and Procedural Paints (v1)

Status: Proposed

## Context

Fret’s long-term rendering roadmap explicitly calls out “future-facing UI looks” and controlled
extensibility without leaking backend handles or arbitrary shaders into component code (ADR 0125).
In practice, modern UI ecosystems (shadcn/Radix-inspired, and MagicUI-class “visual components”)
rely on effects that are **not** well-expressed as:

- solid colors,
- or even gradients alone (ADR 1172).

Common examples:

- procedural patterns: dot/grid/stripe/checkerboard,
- animated beams/highlights (border beam, moving shine),
- noise/grain overlays and subtle texture,
- “sparkle” and lightweight stylization primitives.

If these remain ecosystem-only and are approximated by “many quads”:

- draw op counts and overdraw increase,
- quality and performance drift across crates,
- and it becomes difficult to budget, degrade deterministically, and diagnose cost (ADR 0120 / ADR 0036).

If we expose arbitrary user-provided WGSL or `wgpu` handles:

- portability drift and security risks increase (ADR 0125 trust model),
- determinism and ordering invariants become fragile (ADR 0002 / ADR 0009).

We need a **controlled, renderer-owned** “material” surface for **lightweight stylization** (Tier B),
with explicit budgets and deterministic fallbacks, that can be referenced from the scene without
introducing a new per-component shader zoo.

## Decision

### D1 — Introduce `MaterialId` as a renderer-owned registry handle (core ID)

Add a new stable ID type to `fret-core`:

- `MaterialId`: an opaque, renderer-owned identifier for a material pipeline.

Allocation and storage remain renderer-owned (similar to `RenderTargetId` ownership patterns):

- apps/components never receive raw backend handles,
- and `MaterialId` values are stable for a session unless explicitly unregistered.

### D2 — Materials are framework-controlled (v1), not user-provided WGSL

V1 materials are restricted to a framework-controlled set:

- no app/plugin-provided WGSL,
- no dynamic shader compilation hooks exposed to component code.

Apps/components may select materials only via `MaterialId` values obtained from the renderer
registration API (see D4).

This keeps v1 aligned with ADR 0125’s trust boundary.

### D3 — Define a fixed-size `MaterialParams` payload

To keep the scene portable and renderer-friendly:

- each material instance receives a fixed-size parameter block: `MaterialParams`.

V1 constraints:

- no heap allocations,
- all floats must be finite (non-finite values are sanitized to deterministic defaults),
- parameter interpretation is material-defined, but the payload size is fixed.

V1 payload size (normative):

- `MaterialParams` is **64 bytes**: `16 * f32`.
- The canonical shader view is `params: vec4<f32>[4]`.

Sanitization (normative):

- Renderers MUST sanitize non-finite values (`NaN`, `±Inf`) to `0.0` before use.
- Renderers SHOULD clamp “obviously unbounded” values (e.g. negative sizes) to a safe default on a
  per-material basis, but MUST do so deterministically.

Rationale:

- fixed-size instance payloads preserve batching and avoid per-op dynamic allocation,
- they work on native + wasm/WebGPU.

### D3.1 — Fixed binding shape (v1)

All Tier B materials share a fixed binding shape:

- exactly one uniform buffer binding for `MaterialParams`,
- no texture/sampler bindings in v1.

Non-goal (v1):

- texture-sampling materials (including sampling `ImageId` or external surfaces) are deferred until
  a dedicated ADR locks bind group shapes + budgets for sampled resources.

### D4 — Registration API lives in `fret-render` (not `fret-core`), and is capability-aware

The renderer exposes a registry API (shape is locked here; exact Rust placement is non-normative):

- `register_material(desc) -> MaterialId`
- `unregister_material(id) -> bool`

`desc` must be a backend-agnostic descriptor:

- it selects from a predefined material set (e.g. “dot grid”, “noise”, “border beam”),
- and does not contain raw backend handles.

### D4.1 — V1 baseline material kinds (portable minimum)

The registry MUST support (at minimum) the following kinds, with stable parameter meanings:

- `DotGrid`: repeating dots on a grid.
- `Stripe`: repeating stripes with angle control.
- `Noise`: stable, deterministic noise/grain (no hidden time dependency).
- `Beam`: a directional highlight band intended for “moving shine” when animated by the caller.
- `Sparkle`: a lightweight sparkle field intended for subtle ambient motion when animated by the caller.

This list is intentionally small; it exists to prevent ecosystem crates from inventing divergent
pattern semantics while still keeping the framework surface controlled.

Capabilities (ADR 0124) may restrict which materials are available on a given backend:

- registration of unsupported materials must fail deterministically,
- callers must fall back to a portable look (see D6).

### D5 — Scene integration: materials are referenced via `Paint` (follow-up to ADR 1172)

Materials are intended to be used as “paint sources” for primitives like quads and paths.

We lock the intended integration point:

- a future revision of `Paint` (ADR 1172 follow-up) will gain:
  - `Paint::Material { id: MaterialId, params: MaterialParams }`.

This ADR does not require landing that `Paint` variant immediately, but it defines the registry and
parameter semantics that the `Paint` integration must use when implemented.

### D6 — Deterministic fallback rules are required

To preserve portability and avoid ecosystem drift:

- if a referenced `MaterialId` is unknown (unregistered), renderers must fall back deterministically
  to `Paint::Solid(Color::TRANSPARENT)` (or a framework-chosen “safe default” if the integration
  point is not `Paint` yet),
- if a material is unavailable on a backend (capability-gated), apps/components must select a
  portable alternative (e.g. gradient-only or solid color), and the renderer must report the
  fallback path in diagnostics.

### D7 — Observability requirements

Renderer perf snapshots must be able to report at least:

- number of draw ops using materials,
- number of distinct materials referenced in the last frame,
- and any capability-driven degradations (ADR 0124 / ADR 0120).

This is required so “ecosystem makes things pretty” cannot silently destroy performance ceilings.

## Non-goals (v1)

- Untrusted/user-provided shader code, plugin ABIs, or general shader graphs (ADR 0125 future work).
- General blend modes and compositing operators beyond premul over (ADR 0040).
- Texture-sampling materials with arbitrary resource binding shapes (defer until budgets and bind
  group shapes are fully locked).

## Alternatives considered

1) Add dozens of hard-coded pattern enums in `fret-core`
   - Rejected: grows the kernel contract surface and increases churn.

2) Allow arbitrary WGSL in ecosystem crates
   - Rejected: violates ADR 0125 trust model and portability goals.

3) Force all stylized components to use Tier A (`RenderTargetId`)
   - Rejected: too heavy for common UI chrome; increases app integration burden.

## Validation / Acceptance criteria

This ADR is considered conformant when:

- a minimal set of framework materials can be registered and referenced from scene paint,
- materials participate in budgets/telemetry and produce deterministic fallbacks,
- a small conformance test exists that renders a procedural pattern and verifies sampled pixels.

## References

- Extensibility tiers + trust model: `docs/adr/0125-renderer-extensibility-materials-effects-and-sandboxing-v1.md`
- Budgets + degradation: `docs/adr/0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Capabilities: `docs/adr/0124-renderer-capabilities-and-optional-zero-copy-imports.md`
- Paint vocabulary (gradients): `docs/adr/1172-paint-primitives-brushes-and-gradients-v1.md`
