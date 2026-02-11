# ADR 0247: Compositing Groups and Blend Modes (v1)

Status: Proposed

## Context

Modern UI ecosystems frequently rely on lightweight compositing semantics beyond premultiplied
alpha-over:

- additive glows / beams,
- multiply/screen overlays (noise/grain, highlights),
- “mix-blend-mode”-style stylization.

If compositing is not a first-class, portable contract, component ecosystems tend to diverge:

- some components emulate blend via “many quads” hacks,
- some components fall back to heavy Tier A pipelines (ADR 0123),
- some components silently depend on backend-specific blend state defaults.

Fret’s current scene contract assumes a single compositing mode (premultiplied alpha-over; ADR 0040) and intentionally defines `PushLayer/PopLayer` as marker-only (ADR 0079). We therefore need a
dedicated, explicit mechanism to express isolated compositing with a small, controlled blend
vocabulary.

Related ADRs:

- Display list ordering and batching: `docs/adr/0002-display-list.md`, `docs/adr/0009-renderer-ordering-and-batching.md`
- Premultiplied alpha contract: `docs/adr/0040-color-management-and-compositing-contracts.md`
- Layers are marker-only: `docs/adr/0079-scene-layers-marker-only-v1.md`
- Renderer budgets + deterministic degradation: `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Renderer extensibility tiers: `docs/adr/0123-renderer-extensibility-materials-effects-and-sandboxing-v1.md`

Design references (non-normative):

- Flutter/Skia expresses isolated compositing via `saveLayer(bounds, paint)` where `paint` carries
  `blendMode` (and optionally filters). Fret’s design mirrors this *shape*, but keeps the public
  contract portable and budgeted (ADR 0118) rather than exposing backend paint objects.

## Decision

### D1 — Add an explicit compositing group stack

Extend the scene op vocabulary with a compositing group stack:

- `SceneOp::PushCompositeGroup { bounds: Rect, mode: BlendMode, quality: EffectQuality }`
- `SceneOp::PopCompositeGroup`

`bounds` is a **computation bound**, not an implicit clip (align with ADR 0117).

Semantics:

- Children inside the group are rendered into an offscreen intermediate (an isolated “saveLayer”).
- When the group is popped, the intermediate is composited back onto the parent target using the
  specified `BlendMode`.
- Within the group, children use the normal premultiplied alpha-over behavior.

### D2 — Blend vocabulary is small and portable (v1)

Define a minimal `BlendMode` set for v1:

- `Over` (default; premultiplied alpha-over)
- `Add` (additive; used for glow/beam)
- `Multiply` (multiply; used for grain/darken overlays)
- `Screen` (screen; used for light overlays)

V1 explicitly defers:

- full CSS `mix-blend-mode` parity,
- per-op blend state on every draw op (group-only in v1),
- advanced Porter–Duff compositing operators.

### D3 — Deterministic degradation under budgets is required

Compositing groups allocate intermediates and MUST participate in renderer budgets (ADR 0118).

If the intermediate cannot be allocated within budgets:

1) The renderer SHOULD degrade intermediate resolution according to `quality` (same tiering as
   effect intermediates; ADR 0118).
2) If no intermediate tier fits, the renderer MUST degrade deterministically by behaving as if:
   - the group was not isolated, and
   - `mode` was `Over`.

All degradations MUST be observable via diagnostics/telemetry (ADR 0036 / ADR 0118).

### D4 — Interaction with clip stack and transforms

Compositing groups respect the effective transform and clip stacks:

- The group’s intermediate is scoped to `bounds` intersected with the effective clip/scissor.
- Rounded clipping must remain correct (ADR 0063 / ADR 0138) for content rendered within the group.

## Consequences

- Ecosystem components can express common MagicUI-class looks (beams, glows, grain overlays) as
  Tier B composition without escalating to Tier A pipelines.
- The renderer can budget and degrade these effects deterministically, preserving performance
  ceilings and debuggability.

## Non-goals

- This ADR does not define a general-purpose shader surface for arbitrary blend math.
- This ADR does not define layer-as-isolation (ADR 0079 remains intact).

## Implementation Notes (non-normative)

To reduce future churn as the group surface grows (e.g. adding an optional color filter), prefer a
descriptor-struct shape rather than repeatedly changing the `SceneOp` enum variant fields:

- `SceneOp::PushCompositeGroup { desc: CompositeGroupDesc }`
- `#[non_exhaustive] pub struct CompositeGroupDesc { bounds, mode, quality, ... }`

This is directly inspired by mature “paint object” designs (Flutter/Skia), while keeping Fret’s
mechanism/policy split intact: component authors primarily consume ecosystem recipes, not raw group
descriptors.
