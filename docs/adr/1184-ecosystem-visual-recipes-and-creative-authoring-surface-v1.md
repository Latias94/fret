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
- Sampled materials (v2): `docs/adr/1181-sampled-materials-and-fixed-binding-shapes-v2.md`
- Masks: `docs/adr/1178-mask-layers-and-alpha-masks-v1.md`
- Compositing groups: `docs/adr/1190-compositing-groups-and-blend-modes-v1.md`
- Frame clock and reduced motion: `docs/adr/1179-frame-clock-and-reduced-motion-gates-v1.md`
- Pointer motion snapshots: `docs/adr/1182-pointer-motion-snapshots-and-move-coalescing-v1.md`
- Procedural determinism: `docs/adr/1183-procedural-material-determinism-seeds-and-time-inputs-v1.md`
- Capabilities + budgets: `docs/adr/0124-renderer-capabilities-and-optional-zero-copy-imports.md`,
  `docs/adr/0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`

Existing in-repo authoring surfaces (to build on, not replace):

- Mechanism wrapper: `EffectLayerProps` in `crates/fret-ui/src/element.rs` (emits `SceneOp::PushEffect/PopEffect`).
- User guide: `docs/effects-authoring.md` (recommended `EffectLayer` usage + pitfalls).
- Early “limited recipes” already exist in `fret-ui-kit`:
  - `ecosystem/fret-ui-kit/src/recipes/glass.rs` + `ecosystem/fret-ui-kit/src/declarative/glass.rs`
  - `ecosystem/fret-ui-kit/src/recipes/pixelate.rs` + `ecosystem/fret-ui-kit/src/declarative/pixelate.rs`

Design references (non-normative):

- Flutter/Skia: “effects and blends live in a paint-like object at group boundaries” (`saveLayer`
  + `Paint.blendMode` / `ImageFilter`). Fret mirrors the *boundary-centric* approach, but keeps it
  portable and budgeted (ADR 0120) and routes authoring through recipes instead of exposing a raw
  shader surface.

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

## Proposed API Shape (non-normative)

This section sketches an ergonomic, token-driven surface for component authors. The goal is to
ensure the ADR is actionable and does not merely restate intent.

### Naming and stability goals (normative intent)

- Kernel mechanisms (`SceneOp` and `fret-ui` wrappers like `EffectLayerProps`) should evolve
  **additively** whenever possible.
- Ecosystem-facing authoring APIs should prioritize **wrapper stability**:
  - keep existing entry points (e.g. `glass_panel`, `pixelate_panel`) stable,
  - migrate internals to shared traits/catalogs,
  - avoid making every component depend on renderer details directly.

### A) Core recipe trait pattern

```rust
pub struct ResolveCtx<'a> {
    pub theme: &'a fret_ui::Theme,
    pub caps: &'a fret_runtime::PlatformCapabilities,
    pub prefers_reduced_motion: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradationReason {
    UnsupportedCapability,
    BudgetExceeded,
    ReducedMotion,
    InvalidInput,
}

pub struct ResolvedWithFallback<T> {
    pub value: T,
    pub degraded: bool,
    pub reason: Option<DegradationReason>,
    /// Best-effort stable identifier for diagnostics (e.g. `"glass.effect.blur"`).
    pub label: Option<&'static str>,
}

pub trait EffectRecipe {
    fn resolve(&self, cx: &ResolveCtx<'_>) -> ResolvedWithFallback<fret_core::EffectChain>;
}
```

Notes:

- `ResolveCtx` must be easy to obtain inside `ElementContext`/widget contexts.
- The resolved form carries a small, structured degradation report that diagnostics can surface.
- V1 keeps this intentionally lightweight (one level). A future extension could allow nested
  reports for complex recipes, but v1 should not require an allocation-heavy tree of reasons.

### B) Declarative wrapper integration

Recipes should provide wrappers that compile down to mechanism:

- `EffectLayerProps` for postprocessing (existing).
- `PushMask/PopMask` for alpha masks (ADR 1178).
- `PushCompositeGroup/PopCompositeGroup` for blend groups (ADR 1180).
- `Paint` / `MaterialId` for fills and strokes (ADR 1172 / ADR 1174).

### C) Catalog pattern (centralize registry + stable defaults)

To avoid per-component ad-hoc registration and to reduce churn:

```rust
pub struct VisualCatalog {
    pub materials: MaterialCatalog,
    pub effects: EffectCatalog,
}

pub struct MaterialCatalog {
    // Registers baseline Tier B materials once per renderer.
    // Hands out stable `MaterialId`s to recipes.
}

pub struct EffectCatalog {
    // Owns canonical recipe presets and token key namespaces.
}
```

The catalog should be created at the app/runtime integration layer (where renderer access exists)
and passed into component ecosystems via an app-owned seam (model/service) rather than by letting
each component call renderer APIs directly.

### C) Migration rule (normative intent)

Existing “limited recipe” helpers in `fret-ui-kit` (glass/pixelate) should be treated as the
bootstrap implementation of this pattern. Future creative recipes (MagicUI, shadcn extensions)
must follow the same resolve/fallback/report shape so we do not grow inconsistent one-off APIs.

Migration guidance:

- Keep `glass_panel` / `pixelate_panel` public signatures stable.
- Move shared token resolution + clamping helpers into a common `recipes::resolve` module.
- Gradually introduce `DegradationReason` reporting and feed it into diagnostics bundles.

### D) Diagnostics hook (best-effort, v1)

When a recipe degrades, ecosystems should be able to report it without wiring bespoke logging:

- `fret-ui-kit` SHOULD provide a tiny sink API (or re-use an existing diagnostics sink) so recipes
  can emit `RecipeDegraded { label, reason }` events that show up in `fretboard diag` bundles
  (ADR 0174).

This does not need to be perfect in v1, but the seam must exist before widespread ecosystem
adoption, otherwise “silent fallback” becomes impossible to debug.

### E) Determinism guidelines (recommended)

Creative recipes should avoid hidden non-determinism that makes diagnostics and perf baselines
flaky:

- Avoid calling `rand::random()` (or equivalent) directly inside view building / paint without an
  explicit seed.
- Prefer explicit seeds in params for procedural looks (ADR 1183).
- When randomness is desired for “sparkles”-class ambient effects, provide a recipe parameter or
  token key that can pin a seed in diagnostics runs.
- Treat wall-clock time as forbidden input for visuals; use the frame clock snapshot (ADR 1179)
  and request frames explicitly (ADR 0034).

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
