# ADR 0060: Shadows and Elevation (No-Blur Baseline)

- Status: Accepted
- Date: 2025-12-25

## Context

shadcn/ui recipes commonly rely on `shadow-*` for elevation (cards, popovers, menus, dialogs) and
expect the same "looks correct everywhere" behavior as CSS box-shadow.

Fret currently records ordered `SceneOp`s and renders them via quads/text/images, without a
general-purpose blur/filter pipeline. If we wait for blur to be "done right", component work will
either:

- hand-roll ad-hoc shadow hacks per widget (drift), or
- avoid elevation entirely (UX regression vs shadcn).

We want a **stable contract** that:

- is usable by component-layer recipes today,
- is renderer-agnostic and portable (no implicit GPU filters),
- can later be upgraded to true blur without breaking authoring APIs.

Since ADR 0286 later introduced a bounded blur-based effect step (`DropShadowV1`), this ADR needs
to stay explicit about the portable baseline it governs instead of silently overlapping with the
effect pipeline.

## Decision

1) Introduce a low-level, declarative drop shadow primitive:

- `fret-ui::element::ShadowStyle`
- `fret-ui::element::ContainerProps { shadow: Option<ShadowStyle>, .. }`

2) Define the baseline rendering strategy as "no-blur elevation":

- A shadow is painted **before** the container background/border.
- The runtime approximates softness by drawing multiple expanded quads with alpha falloff
  (`ShadowStyle.softness` controls the number of layers).
- The contract is expressed in *layout space* (Px offsets/spread), not in device pixels.

3) Keep shadow semantics **low-opinionated** and component-owned:

- `fret-ui` exposes the primitive but does not prescribe `shadow-sm/md/lg` or interaction policies.
- `fret-ui-kit` maps shadcn-like elevation levels to `ShadowStyle` using theme extension
  tokens under `component.shadow.*`.

4) Adopt an explicit coexistence posture with ADR 0286:

- `ShadowStyle` remains the portable baseline for box/container chrome and theme-token-driven
  component presets.
- `DropShadowV1` remains an explicit effect-step surface for content-derived blur where the caller
  already owns effect bounds and `EffectMode::FilterContent`.
- v1 MUST NOT silently translate generic `ContainerProps.shadow` / `ShadowPreset` usage into
  `DropShadowV1`.

## Theme / Tokens

Component recipes should resolve shadow parameters via extension keys:

- `component.shadow.xs.offset_x`, `.offset_y`, `.spread`, `.softness`
- `component.shadow.sm.offset_x`, `.offset_y`, `.spread`, `.softness`
- `component.shadow.md.*`
- `component.shadow.lg.*`
- `component.shadow.xl.*`

When a preset family is multi-layer, ecosystem themes may also seed secondary lanes such as
`component.shadow.sm2.*`, `component.shadow.md2.*`, `component.shadow.lg2.*`, and
`component.shadow.xl2.*`.

Shadow color uses the semantic key `shadow` (best-effort fallback is opaque black; component code
typically overrides alpha per level).

## Non-Goals (for now)

- A general blur/filter pipeline for arbitrary primitives.
- Inner shadows, inset shadows, or drop-shadows on arbitrary paths.
- Physically correct shadows (this is a UI elevation affordance, not lighting).
- The blur-based effect-backed shadow contract itself; that lives in ADR 0286.

## Consequences

- Component-layer shadcn surfaces can ship a consistent elevation vocabulary immediately.
- The rendering backend can later replace the "layered quads" approximation with true blur, while
  keeping `ShadowStyle` stable (only the implementation changes).
- Themes can tune elevation per platform/DPI by adjusting `component.shadow.*` extension tokens.
- Portable component chrome keeps a deterministic baseline on every backend, while higher-fidelity
  blur remains an explicit opt-in via `DropShadowV1` rather than an implicit renderer substitution.
