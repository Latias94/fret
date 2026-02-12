# ADR 0239: Mask Layers and Alpha Masks (v1)

Status: Proposed

## Context

Rounded-rect clipping (`PushClipRRect`) is a baseline composition primitive in Fret (ADR 0063), but
it is not the same as *masking*:

- Clipping is primarily a geometry constraint ("overflow hidden") and must affect hit-testing.
- Masking is primarily a visual shaping tool ("fade edges", "spotlight", "mask-image") and often
  **should not** change hit-testing.

MagicUI-style recipes rely heavily on mask semantics:

- `mask-image: linear-gradient(...)` for fades,
- `radial-gradient(...)` for spotlights / reveal animations,
- image-based masks for patterns and cutouts.

If Fret lacks a first-class mask contract, ecosystem components will either:

- re-encode masks as bespoke shader/effect pipelines (Tier A, ADR 0123), or
- approximate masks by stacking many quads (poor parity and hard to keep consistent).

We want a portable, budgetable v1 mask surface that:

- stays mechanism-level (no CSS mask-composite feature set),
- composes with transforms, clips, and effect groups,
- has deterministic degradation under renderer budgets (ADR 0118),
- does not leak backend handles into `fret-ui` (ADR 0066 / ADR 0123).

Related ADRs:

- Rounded clipping: `docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`
- Effects and bounds semantics: `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
- Effect clip masks (renderer-internal): `docs/adr/0138-renderer-effect-clip-masks-and-soft-clipping-v1.md`
- Renderer budgets + deterministic degradation: `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Paint primitives (gradients): `docs/adr/0233-paint-primitives-brushes-and-gradients-v1.md`
- Image `object-fit`: `docs/adr/0237-image-object-fit-for-sceneop-image-v1.md`

## Decision

### 1) Add an explicit mask stack to the scene contract (v1)

Extend the portable scene op vocabulary with a mask stack:

- `SceneOp::PushMask { bounds: Rect, mask: Mask }`
- `SceneOp::PopMask`

`bounds` is in window-local logical pixels (same coordinate space as other scene rects) and is a
**computation bound**, not an implicit clip (align with ADR 0117).

### 2) Mask semantics are coverage multiplication (alpha mask)

Masks define a **coverage function** `m(x, y) in [0, 1]` in scene space.

When a mask is active:

- All pixels produced by subsequent ops are multiplied by the effective mask coverage before being
  composited to the parent target (premultiplied alpha; ADR 0040).
- Multiple nested masks multiply coverage: `m_effective = m1 * m2 * ...`.
- Masks compose with the existing clip stack: clip determines *visibility* (hard/soft clip), and
  mask further shapes *opacity*.

### 3) Masks are paint-only by default (hit-testing unchanged)

`PushMask` affects **paint only**:

- Hit-testing MUST continue to be governed by layout bounds + clip stack semantics (ADR 0063 /
  ADR 0082).
- Masking MUST NOT change which elements receive pointer events.

Rationale:

- This matches the common “mask-image is visual only” mental model.
- It avoids surprising interaction loss when a mask animates (spotlight/reveal effects).

If a component wants masked hit-testing, it must use clipping or explicit hit-test hooks in the
component layer (future: a separate, explicit contract).

### 4) V1 mask sources (minimal, portable subset)

V1 `Mask` supports a small set of sources that are broadly portable:

- `Mask::LinearGradient { start, end, stops, tile_mode, color_space }`
- `Mask::RadialGradient { center, radius, stops, tile_mode, color_space }`

These reuse the gradient semantics and stop representation from ADR 0233, but only the **alpha**
channel of the evaluated color is used as coverage.

V1 explicitly defers:

- arbitrary vector paths (`PushClipPath`),
- CSS-like `mask-composite` operators,
- multi-channel masks and luminance masks.

### 5) Capability gating and deterministic degradation

Mask evaluation is renderer-owned and capability-gated (ADR 0122 / ADR 0123):

- If the renderer supports analytic gradient masks, it SHOULD implement them without allocating an
  intermediate texture (best-effort).
- If masks are unsupported or disabled by budgets, the renderer MUST degrade deterministically:
  - Prefer degrading mask quality (e.g. fewer stops or reduced precision) before disabling.
  - If the mask is disabled entirely, the subtree is rendered unmasked (paint-only change).

All degradations MUST be:

- deterministic (input-only, no timing randomness),
- layout-invariant,
- observable via renderer telemetry/diagnostics (ADR 0118 / ADR 0036).

### 6) Interaction with effect groups

Masking is orthogonal to effects:

- A mask outside an effect group gates the *composited output* of that group.
- A mask inside an effect group gates the content within the group before effect evaluation.

Renderers may implement this by applying the effective mask coverage at composite boundaries, and
MUST preserve ordering semantics.

## Consequences

- Ecosystem crates can express common mask-image recipes (fades, spotlights) without Tier A
  pipelines.
- The renderer gains a new portable coverage mechanism that composes with effects and clips, while
  keeping hit-testing stable.

## Non-goals

- Full parity with CSS masks is not a goal for v1.
- Path masking is deferred; v1 focuses on gradients because they cover a large portion of common
  UI recipes with a compact, portable surface.

