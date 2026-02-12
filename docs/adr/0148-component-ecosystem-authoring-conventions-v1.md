# ADR 0148: Component Ecosystem Authoring Conventions (v1)

Status: Proposed

## Context

Fret is growing beyond kernel mechanisms (`crates/*`) into an ecosystem of policy-heavy UI kits,
design-system component libraries, and domain ecosystems (node graphs, charts, gizmos, markdown).

We already have many relevant ADRs (tokens, DPI/units, commands/keymap, overlays, authoring model),
but third-party authors (and even our own ecosystem crates) still face friction:

- inconsistent initialization patterns (where commands/tokens/settings are registered),
- ad-hoc shortcut wiring inside components instead of `CommandId` + keymap,
- token key naming drift and hard-coded styling leaks,
- unclear unit semantics for “px” at boundaries (UI logical vs render-target physical),
- unclear “what must be stable vs can evolve quickly” across crates.

We want a small set of **cross-cutting conventions** that make ecosystem crates:

- easy to integrate into apps (and into each other),
- portable (native + wasm),
- compatible with hotpatch/hot reload safety boundaries,
- future-proof for multiple design systems and component libraries.

## Goals

1. Define a shared checklist for ecosystem crates (UI or domain) to be composable and predictable.
2. Standardize “integration surfaces” (commands, tokens, settings, assets) without forcing a monolith.
3. Keep kernel boundaries intact (ADR 0092), and align with the golden-path story (ADR 0107/0110/0111).
4. Make “simple apps” stay simple: advanced behaviors must remain opt-in and feature-gated.

## Non-goals

- Designing a single “everything” crate as the recommended entry point.
- Forcing a specific async runtime or background execution model.
- Solving editor-grade project asset pipelines (ADR 0026 remains app-owned).

## Decision

### 1) Keep the three-layer model as the baseline

We keep the stable layering:

- **Kernel** (`crates/*`): mechanisms + contracts only.
- **Ecosystem** (`ecosystem/*`): policy + component surfaces + integration helpers.
- **Tooling** (`apps/*`, `fretboard`): dev workflows and templates.

Hard rule: `crates/*` must not depend on `ecosystem/*` (ADR 0092).

### 2) Ecosystem crates must provide a small, explicit integration entry point

Each ecosystem crate that expects app-level integration (commands/tokens/settings/assets) should
provide at least one explicit entry point:

- `pub fn install(app: &mut fret_app::App, services: &mut dyn fret_core::UiServices)` *(preferred)*, or
- `pub fn install_into(driver: &mut fret_bootstrap::UiAppBuilder)` *(if it is bootstrap-focused)*.

Normative rules:

- The entry point must be **idempotent** (safe to call once per app lifetime; multiple calls are no-ops).
- The entry point must be **purely app-thread** (no background mutations of `App`).
- The entry point must not rely on hidden global singletons beyond `App` globals/models.

Rationale:

- Idempotent “install” keeps composition predictable.
- Keeping it as a function preserves hotpatch-friendly behavior (ADR 0105 / ADR 0110).

Implementation guidance:

- Do not force `fret-app` as a dependency for crates that only provide portable UI components. Prefer to keep
  `install(...)` behind an optional feature (e.g. `app-integration`) so downstream users can choose whether
  they want app-level conveniences (commands/default keybindings/settings wiring).

### 3) Commands and shortcuts are standardized: no component-local hard-coded key handling

Commands:

- Public behaviors that can be invoked via menus/palette/shortcuts must be expressed as a `CommandId`
  (ADR 0023 / ADR 0020).
- Command IDs must be namespaced: `domain.scope.action` (ADR 0111).

Shortcuts:

- Default shortcuts must be expressed as `CommandMeta.default_keybindings` and installed into the app keymap
  (ADR 0021 / ADR 0022 / ADR 0023).
- Component code should not hard-code “Ctrl+K”-style behavior via raw key events, except for true low-level
  text input engines and platform-specific IME affordances.

### 4) Theme tokens are the primary styling surface; hard-coded values are discouraged

All component styling should be token/key driven (ADR 0032 / ADR 0101 / ADR 0050).

Normative rules:

- Ecosystem crates must document their token keys (namespace + meaning + fallback).
- Component code should not bake an invariant palette as a dependency.
- Typed keys (enums) are allowed for baseline surfaces, but extensible string keys remain supported.

### 5) Units are explicit at boundaries; “px” must be qualified in API names and docs

The baseline UI coordinate space is **logical pixels** (ADR 0017).

At boundaries where both spaces matter (viewport surfaces, engines, gizmos), APIs must be explicit about:

- UI-space logical px,
- render-target physical px,
- the conversion factor (`pixels_per_point` / scale factor),

consistent with the explicit-units viewport input contract (ADR 0132).

### 6) Settings and config layering is opt-in, but standardized when present

If an ecosystem crate reads settings, it must:

- define a stable, namespaced settings key space,
- provide defaults and schema guidance,
- support layered loading as an app decision (ADR 0014).

### 7) Assets: only UI render assets are in scope for ecosystem integration helpers

Ecosystem crates may integrate with UI render-asset caches (ADR 0004, ADR 0107/0111) but must not
assume an editor-grade project asset pipeline (ADR 0026).

### 8) Stability tiers must be clear

Ecosystem crates should declare a stability intent for their public surface:

- “Stable contract surface” (expected to change slowly),
- “Recipe layer / fast iteration” (may change faster),
- “Internal” (not for downstream use).

This is especially important for future third-party component ecosystems.

### 9) Feature-gating conventions (recommended)

To keep small apps small and enable incremental adoption, ecosystem crates should:

- keep heavyweight integrations behind features (icons/UI assets/command palette/dev hotpatch),
- use predictable feature names across crates (e.g. `icons`, `ui-assets`, `app-integration`, `dev-tools`) when feasible,
- avoid enabling large optional deps in `default` features unless the crate is explicitly a distribution crate.

## Implementation Plan (non-binding)

1. Publish a component-author facing guide that translates these rules into practical steps.
2. Update existing ecosystem crates to expose `install(...)` entry points where they currently have ad-hoc wiring.
3. Consolidate the golden-path builder (`fret-bootstrap`) to compose multiple “installers” via feature gates.
4. Add lightweight conformance tests for:
   - command registration + default keybinding install,
   - token key resolution + fallback behavior,
   - unit correctness at viewport/tooling boundaries (where applicable).

## Alternatives Considered

### A) Introduce a new “kit” meta crate as the recommended user entry point

Rejected as the default recommendation, because it conflicts with ADR 0109’s explicit goal to avoid a mixed
“everything crate” story. A meta crate may still exist as an optional distribution later, but must not be the
primary recommended path unless we supersede ADR 0109 with a new decision.

### B) No conventions; let each ecosystem crate choose its own wiring patterns

Rejected: it increases user friction and breaks composability as the ecosystem grows.

## References

- Kernel/backends/apps layering: `docs/adr/0092-crate-structure-core-backends-apps.md`
- Golden-path driver/pipelines: `docs/adr/0110-golden-path-ui-app-driver-and-pipelines.md`
- User-facing crate surfaces: `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- Ecosystem integration guidance: `docs/adr/0111-ecosystem-integration-contracts.md`
- Commands/palette: `docs/adr/0023-command-metadata-menus-and-palette.md`
- Keymap + when: `docs/adr/0021-keymap-file-format.md`, `docs/adr/0022-when-expressions.md`
- DPI + units: `docs/adr/0017-multi-window-display-and-dpi.md`, `docs/adr/0132-viewport-input-forwarding-explicit-units.md`
- Tokens/theme: `docs/adr/0032-style-tokens-and-theme-resolution.md`, `docs/adr/0101-semantic-theme-keys-and-extensible-token-registry.md`
- Component authoring model: `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
