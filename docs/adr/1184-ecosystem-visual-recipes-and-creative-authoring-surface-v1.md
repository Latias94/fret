# ADR 1184: Ecosystem Visual Recipes and Creative Authoring Surface (v1)

Status: Proposed

## Context

Fret’s contract philosophy intentionally keeps kernel mechanisms small and portable, while
delegating “policy and looks” to ecosystem crates (ADR 0066). This works well for standard
components, but creative UI ecosystems (MagicUI-class) amplify two authoring pressures:

1) **Developers want a high-level, composable vocabulary**:
   - patterns (dot/stripe/noise),
   - masks (fades/spotlights),
   - postprocessing (blur/pixelate/threshold),
   - blend/compositing (add/multiply/screen),
   - motion (time + easing + reduced-motion).

2) **The kernel should not become a giant component/effect library**:
   - we must avoid “every new look adds a new SceneOp enum variant” drift,
   - we must preserve budgeted, deterministic degradation and diagnostics (ADR 0120 / ADR 0036).

We therefore need a stable developer story for authoring *creative looks* that:

- is ergonomic for component authors,
- is token-driven and theme-friendly,
- is capability-aware and degradable,
- maps cleanly onto Tier A vs Tier B (ADR 0125),
- and stays consistent across ecosystem crates (shadcn, magicui, material3, editor kits).

Related ADRs:

- Effect recipes and tier selection: `docs/adr/0149-effect-recipes-and-tier-selection-v1.md`
- Paint primitives: `docs/adr/1172-paint-primitives-brushes-and-gradients-v1.md`
- Controlled materials: `docs/adr/1174-controlled-materials-registry-and-procedural-paints-v1.md`
- Masks: `docs/adr/1178-mask-layers-and-alpha-masks-v1.md`
- Frame clock and reduced motion: `docs/adr/1179-frame-clock-and-reduced-motion-gates-v1.md`
- Capabilities + budgets: `docs/adr/0124-renderer-capabilities-and-optional-zero-copy-imports.md`,
  `docs/adr/0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`

## Decision

### D1 — The creative authoring surface lives in `ecosystem/fret-ui-kit`

We standardize that the primary developer-facing API for creative visuals is in:

- `ecosystem/fret-ui-kit` (policy + recipes),

not in:

- `crates/fret-ui` (mechanism only),
- `crates/fret-core` (portable scene primitives only).

### D2 — Recipes are first-class: “describe intent, resolve to mechanism”

We define an ecosystem pattern:

- A recipe is a small struct with stable token keys and a `resolve(...)` method that produces a
  portable mechanism configuration (scene ops + params), capability-gated and degradable.

Recipe categories (v1):

- `PaintRecipe` (solid/gradients/materials, per ADR 1172/1174)
- `MaskRecipe` (gradient alpha masks, per ADR 1178)
- `EffectRecipe` (effect chains, per ADR 0149/0119/1175)
- `CompositeRecipe` (blend groups, per ADR 1180)
- `MotionRecipe` (time gating + reduced motion, per ADR 1179)

### D3 — All recipes must have deterministic fallback chains

Each recipe must define a deterministic fallback chain ordered from “best” to “portable minimum”:

- capability unavailable -> fallback,
- budget exhausted -> fallback,
- reduced motion -> pinned time + no continuous frames (for ambient motion).

The resolved form must allow diagnostics to report:

- which fallback branch was taken,
- why (capability/budget/policy),
- and the effective resolved parameters.

### D4 — Material registry access is centralized (avoid per-component registration)

To prevent a “registration zoo” and reduce churn:

- `fret-ui-kit` owns a small **material catalog** abstraction that registers the baseline Tier B
  materials once per renderer instance and hands out `MaterialId` handles to recipes.
- Component crates consume recipes and do not call the renderer registry directly.

This keeps policy consistent and avoids subtle “material mismatch” bugs across crates.

### D5 — Tier A escape hatches stay explicit

Recipes may provide an explicit Tier A escape hatch when a look cannot be expressed portably:

- return `TierA(RenderTargetId)` or a `ViewportSurface` wrapper,
- or require the app to supply an external surface.

This keeps the Tier A vs Tier B decision rule stable and user-visible (ADR 0125 / ADR 0149).

## Consequences

- Component authors can build creative UIs by composing small recipes instead of assembling raw
  renderer details.
- The kernel stays small and mechanism-only, while the ecosystem gains a curated, consistent
  creative vocabulary.
- Diagnostics and perf gates become easier because fallbacks are explicit and resolvable.

## Non-goals

- This ADR does not add a general-purpose shader graph or plugin ABI.
- This ADR does not require all ecosystems to share identical visuals; it requires shared seams and
  consistent degradation rules.

