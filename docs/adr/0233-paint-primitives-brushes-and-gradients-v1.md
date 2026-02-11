# ADR 0233: Paint Primitives (Brushes + Gradients) (v1)

Status: Accepted

## Context

Fret’s scene contract intentionally kept primitives small and renderer-friendly early on: the
baseline `SceneOp::Quad` surface is “solid color fill + solid color border”, and higher-level UI
libraries (shadcn/Radix-aligned, Material3, and future ecosystems like MagicUI-style components)
compose richer visuals in the ecosystem layer.

However, modern UI recipes frequently rely on **paint primitives** that cannot be expressed with
solid colors without severe trade-offs:

- Gradient fills (linear + radial) for backgrounds, borders, and highlight effects.
- “Spotlight” pointer-follow highlights (radial gradients whose center is driven by pointer move).
- Animated beams and moving highlights (gradients whose coordinate frame changes over time).

Without a first-class paint vocabulary:

- Ecosystem components devolve into “many-quads” approximations (high overdraw + high draw op count),
  which undermines the framework’s performance ceilings and makes budgeting/telemetry harder.
- Different component crates implement different approximations, producing visual drift and
  inconsistent quality across platforms.
- Renderer evolution (ADR 0116 / ADR 0123) is blocked because there is no stable contract for
  “brush-like” draw-time evaluation.

We need a minimal, portable, budgetable paint surface that:

- preserves strict `Scene.ops` ordering semantics (ADR 0002 / ADR 0081),
- remains `wgpu`-free at the component level (ADR 0123),
- is implementable on native + wasm/WebGPU,
- and is small enough to keep scene recording/encoding and caching practical.

## Decision

### D1 — Introduce a first-class `Paint` value type in `fret-core`

Define a new core value type:

- `Paint`: a fixed-size, copyable representation of “how to color a fragment”.

V1 supports:

- `Paint::Solid(Color)`
- `Paint::LinearGradient(LinearGradient)`
- `Paint::RadialGradient(RadialGradient)`

Gradients include two additional semantic knobs:

- `TileMode`: how sampling behaves outside the gradient domain (v1 reserves this as
  `Clamp | Repeat | Mirror`; implementations may initially support `Clamp` only, but the enum must
  be part of the stable contract so behavior does not fork across ecosystems).
- `ColorSpace`: the interpolation color space for stop colors (v1 reserves `Srgb | Oklab`; v1
  implementations may initially support `Srgb` only, but the field is part of the stable contract).

Constraints:

- `Paint` is **POD-like** (no heap allocations, no `Vec`).
- All floating-point fields must be finite; non-finite inputs are treated as a safe fallback (see
  D6).
- Gradient stop counts are bounded by a small constant (`MAX_STOPS`) to keep instance payloads
  bounded and portable.

### D2 — Extend `SceneOp::Quad` to use `Paint` for fill and border

Evolve `SceneOp::Quad` from:

- `background: Color`
- `border_color: Color`

to:

- `background: Paint`
- `border_paint: Paint`

`border: Edges` remains unchanged and continues to define the border thickness.

Rationale:

- Gradient borders are a common UI requirement (focus glows, animated beams, rainbow borders).
- Keeping border rendering inside the quad shader preserves the existing ordering + clip behavior
  and avoids introducing a new “stroke op” surface prematurely.

### D3 — (Optional, recommended) Extend `SceneOp::Path` to accept `Paint`

V1 does not require path gradients, but the paint vocabulary should be reusable across draw ops.

If implemented, the v1-compatible extension is:

- `SceneOp::Path { ..., paint: Paint, ... }`

This ADR does not lock tessellation or AA details for gradient paths; it only standardizes the
paint input and its coordinate semantics (D4).

### D4 — Coordinate space semantics: paints are evaluated in the op’s “local scene space”

Paint coordinates are expressed in the same coordinate space as the op’s `rect` / `origin`:

- For `SceneOp::Quad`, the fragment shader evaluates `Paint` using the quad’s `local_pos` (the
  `rect.xy + uv * rect.zw` position before the composed transform is applied).

This implies:

- transforms (translate/scale/rotate) apply to geometry as today; paints “move with the element”
  because evaluation is tied to the element’s local coordinates,
- pointer-driven paint effects can use element-local pointer positions without extra transforms,
- clipping and opacity stacks behave the same as solid colors.

### D5 — Color space semantics: gradient interpolation is in linear space

To remain consistent with Fret’s color/compositing contracts (ADR 0040):

- gradient stop colors are specified as linear RGBA floats (`f32`),
- interpolation happens in **linear** space (after converting into the selected `ColorSpace`),
- output encoding (srgb or not) is handled by the renderer as it is for solid colors today.

### D6 — Validation and fallback behavior is part of the contract

To keep the scene contract resilient and deterministic:

- If a `Paint` contains non-finite floats, the renderer must treat it as `Paint::Solid(Color::TRANSPARENT)`.
- If gradient stop data is invalid (e.g. stop count is 0, offsets are non-monotonic, or offsets are
  outside `[0,1]`), the renderer must clamp/sanitize to a deterministic fallback:
  - clamp offsets to `[0,1]`,
  - sort by offset (stable sort),
  - if all stops become identical, treat as `Solid` of that color.
- If an implementation does not support a non-`Clamp` `TileMode` or a non-`Srgb` `ColorSpace` in
  v1, it must degrade deterministically:
  - `Repeat/Mirror` behave as `Clamp`,
  - `Oklab` behaves as `Srgb`.

Sanitization must be deterministic across platforms to keep `fretboard diag` parity and scene
fingerprints meaningful.

## Non-goals (v1)

- Arbitrary user-provided WGSL or plugin-authored paints (see ADR 0123 trust model).
- General CSS-like masking and blend modes.
- Unbounded stop counts, conic gradients, or gradient mesh primitives.
  - If a “conic sweep highlight” is required for shimmers, prefer a controlled Tier B material kind
    (e.g. `ConicSweep`) rather than expanding the `Paint` v1 gradient set.
- A full material graph system. V1 provides a minimal paint vocabulary; richer procedural looks
  should layer through a future `MaterialId` registry (ADR 0123) or Tier A external pipelines.

## Consequences

- Scene and renderer instance payload sizes will increase.
- Renderers must implement paint evaluation in a portable, budgetable way (native + wasm/WebGPU).
- Ecosystem component libraries gain a stable mechanism primitive, reducing pressure to “fake”
  gradients with many quads.
- This creates a natural path to a controlled `MaterialId` registry (ADR 0123) without exposing
  raw GPU handles at the component layer.

## Alternatives considered

1) **Keep solid colors only; approximate gradients with many quads**
   - Rejected: large overdraw/op count, inconsistent quality, hard to budget and diagnose.

2) **Force all gradient-heavy components to use Tier A (`RenderTargetId`)**
   - Rejected: too heavy for common UI chrome; increases app integration burden for basic visuals.

3) **Adopt a fully general material graph now**
   - Deferred: too large a surface; v1 needs a small, stable “paint vocabulary” first.

## Validation / Acceptance criteria

Implementation is considered conformant when:

- A linear gradient and a radial gradient can be rendered via `SceneOp::Quad` with correct clip stack
  interaction (rect + rrect) and correct opacity stack behavior.
- A gradient border can be rendered with the existing quad border semantics (`Edges` thickness).
- A minimal conformance test exists that renders a known gradient scene and verifies sampled pixels
  (similar to existing blur/clip conformance tests).
- Diagnostics can report paint usage counts (at least “number of gradient quads”) to make the cost
  visible during ecosystem adoption.

## Follow-ups (expected)

- Effect-level “threshold / color matrix” steps for SVG-filter-class text effects (Tier B) or a
  recommended Tier A pattern for those effects.
- Controlled `MaterialId` registry for procedural patterns (noise, beams, sparkles) with budgets
  and deterministic degradation (ADR 0123 + ADR 0118).
- Masking and blend modes (if/when required by core UI recipes).
