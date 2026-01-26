# Material 3 Refactor: Style API Alignment v1 (Design-System-Agnostic Interfaces)

Status: Draft / proposed

This workstream exists to ensure `ecosystem/fret-ui-material3` becomes a *reference consumer* of
Fret’s ecosystem style authoring contracts, instead of inventing a parallel styling API.

It intentionally focuses on **interfaces that matter outside Material 3**, so future component
libraries (Fluent, Material variants, custom design systems) can follow one predictable pattern.

## Context

Fret already has a repository-wide contract for “state → style” authoring:

- ADR 1158: `WidgetStates` + `WidgetStateProperty<T>` (state resolution primitive)
- ADR 1159: ecosystem component `*Style` override surface (partial per-state overrides)

At the same time, Material 3 alignment work introduced a Compose-inspired internal “foundation”
layer (`token_resolver`, `indication`, `motion_scheme`, `content`, `interactive_size`, …).

Both directions are valuable, but **two parallel authoring patterns** are a long-term cost:

- downstream users cannot predict how to override visuals across component libraries,
- new ecosystems will copy the “wrong” patterns and reintroduce drift,
- maintaining multiple override mechanisms forces late breaking refactors.

This plan aligns Material 3 components with ADR 1159 so Material 3 becomes:

- a design-system policy library (tokens + defaults),
- a proving ground for a consistent public `*Style` API shape.

## Design constraints

- `crates/fret-ui` stays mechanism-only (no design-system policy).
- `ecosystem/fret-ui-kit` owns design-system-agnostic authoring primitives:
  `WidgetStates`, `WidgetStateProperty<T>`, and shared resolution helpers.
- `ecosystem/fret-ui-material3` owns Material policy:
  token namespaces (`md.sys.*`, `md.comp.*`), alias mapping, motion, ink rules, etc.
- Any public component override surface in ecosystem crates should follow ADR 1159:
  `Option<WidgetStateProperty<Option<T>>>` + shallow right-biased `merged()`.

## Goal

Make the following authoring experience consistent across ecosystems:

- Per-component `*Style` structs exist for interactive controls.
- Users can override **only one state** of one slot without copying full defaults.
- Merge semantics are predictable (no deep merge).
- Token namespace rules are explicit (Material never falls back to shadcn keys).

## Scope (v1)

In scope:

- Define + implement `*Style` override surfaces for Material 3 interactive controls.
- Introduce shared resolution helpers so the override plumbing stays small.
- Add minimal gallery coverage for “default vs override” validation.
- Document the token namespace decision for Material (`md.sys.*` / `md.comp.*`).

Out of scope (v1):

- A cross-design-system runtime polymorphism layer (no `DesignSystem` trait yet).
- Full parity with Compose / Flutter APIs.
- A universal “theme → style” resolver shared by all ecosystems (keep policy local).

## Decision: Token namespaces

Material 3 uses:

- `md.sys.*` for system tokens (colors, typescale, motion, shape, state opacities),
- `md.comp.*` for component tokens (scalars and component-specific roles).

Do **not** introduce a second Material namespace (e.g. `material3.*`) for the same outcomes.

If older workstreams mention `material3.*`, treat them as deprecated notes and reconcile the docs.

## Contract: Material 3 component override shape (ADR 1159)

For any slot that varies by widget state, use:

- `Option<WidgetStateProperty<Option<T>>>`

Where:

- outer `Option`: whether the slot is overridden at all,
- inner `Option<T>`: per-state “no override” to fall back to defaults for that state.

All Material 3 `*Style` structs must provide:

- `fn merged(self, other: Self) -> Self` (shallow, right-biased)

## Current adoption snapshot (main)

This snapshot is intentionally “API-shape only”: it tracks whether a component exposes a public
ADR 1159-style override surface, not whether it is visually aligned with Material.

| Component | File | Has `*Style` | Has `.style(...)` | ADR 1159 shape | Notes |
|---|---|---:|---:|---:|---|
| Select | `ecosystem/fret-ui-material3/src/select.rs` | Yes (`SelectStyle`) | Yes | Yes | Trigger + option slots use `Option<WidgetStateProperty<Option<ColorRef>>>`. |
| RadioGroup | `ecosystem/fret-ui-material3/src/radio_group.rs` | Yes (`RadioGroupStyle`) | Yes | Yes | Items + icon/label/indicator slots use ADR 1159 shape. |
| Button | `ecosystem/fret-ui-material3/src/button.rs` | No | N/A | No | Uses foundation + tokens; needs a public `ButtonStyle` surface if we want consistent overrides. |
| IconButton | `ecosystem/fret-ui-material3/src/icon_button.rs` | No | N/A | No | Same as Button. |
| Checkbox | `ecosystem/fret-ui-material3/src/checkbox.rs` | No | N/A | No | Same; currently outcome-oriented implementation. |
| Switch | `ecosystem/fret-ui-material3/src/switch.rs` | No | N/A | No | Same; currently outcome-oriented implementation. |
| Radio | `ecosystem/fret-ui-material3/src/radio.rs` | No | N/A | No | Uses foundation + tokens; no public override surface yet. |
| Tabs | `ecosystem/fret-ui-material3/src/tabs.rs` | No | N/A | No | Uses foundation + tokens; no public override surface yet. |
| TextField | `ecosystem/fret-ui-material3/src/text_field.rs` | No | N/A | No | Uses `md.*` tokens but does not expose ADR 1159-style overrides (also has `error` as a bespoke boolean). |
| Menu | `ecosystem/fret-ui-material3/src/menu.rs` | No | N/A | No | Policy-heavy; needs a careful, minimal override surface if we expose one. |
| Dialog | `ecosystem/fret-ui-material3/src/dialog.rs` | No | N/A | No | Same as Menu (overlay surface + motion + focus). |
| Tooltip | `ecosystem/fret-ui-material3/src/tooltip.rs` | No | N/A | No | Often provider-driven; may stay policy-only in v1. |
| Snackbar | `ecosystem/fret-ui-material3/src/snackbar.rs` | No | N/A | No | Typically a higher-level pattern; likely v2. |

## Implementation plan

Status legend: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

### M3SA-000 — Doc alignment and tracking

- [ ] Decide whether `docs/workstreams/state-driven-style-resolution-v1.md` should:
  - (A) track Material 3 progress (as a consumer of ADR 1159), or
  - (B) stop tracking Material 3 and only track cross-ecosystem primitives.
- [ ] If (B), move any Material 3 token lists from the SDSR workstream into `material3-todo.md`.
- [ ] Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` evidence for ADR 1158/1159 to include Material 3
  adoption once v1 `*Style` surfaces land across M3 controls.

### M3SA-010 — Shared resolution helpers (kit-level)

Add small, design-system-agnostic helpers (prefer `fret-ui-kit`) to reduce per-component boilerplate:

- [ ] `resolve_slot(overrides, defaults, states) -> T` for the ADR 1159 “nullable per-state” shape.
- [ ] `merge_slot(self.field, other.field)` helper (optional) to standardize right-biased merge.

### M3SA-100 — Define minimal `*Style` slot vocab for Material 3 controls

Keep v1 small and focused on the slots that are commonly overridden by downstream apps.

Suggested initial slot sets:

- Button: container/content/outline + optional icon color.
- Checkbox: outline/indicator/label + optional state-layer color.
- Switch: track/thumb/outline/label.
- Radio: outline/indicator/label.
- Tabs: container + indicator + label.
- TextField: container/outline/focus-ring + text/placeholder/supporting.
- Menu / MenuItem: container + item background/foreground + selection state.

- [ ] For each component, document which slots are public and which remain policy-only.
- [ ] Confirm how `WidgetState::Open` / `Selected` are used (e.g. menus/selects/tabs).

### M3SA-200 — Implement `*Style` surfaces per component (incremental)

Per component:

- [ ] Export `*Style` in the module and in `ecosystem/fret-ui-material3/src/lib.rs` as needed.
- [ ] Add `.style(style)` builder to the component.
- [ ] Resolve theme-derived defaults first, then apply overrides at resolve-time.
- [ ] Keep foundation-owned behavior (indication, motion scheme, token fallback chain) unchanged.

Recommended order:

- [ ] TextField (high value; currently bespoke)
- [ ] Button
- [ ] Checkbox / Switch / Radio
- [ ] Tabs
- [ ] Menu / Dialog surfaces (only if the override surface remains small)

### M3SA-300 — Gallery validation pages

For each component that gains a `*Style` surface:

- [ ] Add a “Default vs Override” comparison block in `apps/fret-ui-gallery`.
- [ ] Include at least one partial override example (hover-only or focus-ring-only).

### M3SA-400 — Decide how to model “invalid/error” (TextField)

We need a cross-ecosystem answer for “error” styling without exploding the state vocabulary.

Options:

- Option A (v1): keep `error` as a component-specific boolean/variant and do not add a new widget state.
- Option B (v2): introduce `WidgetState::Invalid` (or a small extension mechanism) in kit-level primitives.

- [ ] Pick v1 approach and document it.

## Evidence anchors

- ADR 1158: `docs/adr/1158-state-driven-style-resolution-v1.md`
- ADR 1159: `docs/adr/1159-ecosystem-style-override-surface-v1.md`
- Kit primitive: `ecosystem/fret-ui-kit/src/style/state.rs`
- Shared patterns: `docs/shadcn-style-override-patterns.md`
- Material foundation: `ecosystem/fret-ui-material3/src/foundation/*`
- Material components: `ecosystem/fret-ui-material3/src/*.rs`
- Gallery: `apps/fret-ui-gallery/src/{spec.rs,ui.rs,docs.rs}`
