# Material 3 Refactor: Style API Alignment v1 (Design-System-Agnostic Interfaces)

Status: In progress (core controls aligned; overlay-heavy components pending)

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

If older workstreams or legacy pilot code mention `material3.*`, treat them as deprecated notes and
migrate them to `md.*` or delete them during the refactor.

## Contract: Material 3 component override shape (ADR 1159)

For any slot that varies by widget state, use:

- `Option<WidgetStateProperty<Option<T>>>`
- Prefer using the alias `fret_ui_kit::OverrideSlot<T>` in Rust code for readability.

Where:

- outer `Option`: whether the slot is overridden at all,
- inner `Option<T>`: per-state “no override” to fall back to defaults for that state.

All Material 3 `*Style` structs must provide:

- `fn merged(self, other: Self) -> Self` (shallow, right-biased)

## Current adoption snapshot (main)

This snapshot is intentionally “API-shape only”: it tracks whether a component exposes a public
ADR 1159-style override surface, not whether it is visually aligned with Material.

Important: this snapshot only counts **exported** crate surfaces (reachable from
`ecosystem/fret-ui-material3/src/lib.rs`). Some early experiments may exist in-tree but are not
wired into the crate and therefore do not represent the current public surface.

| Component | File | Has `*Style` | Has `.style(...)` | ADR 1159 shape | Notes |
|---|---|---:|---:|---:|---|
| Select | `ecosystem/fret-ui-material3/src/select.rs` | Yes (`SelectStyle`) | Yes | Yes | Re-introduced on `md.sys.*` / `md.comp.*` tokens (including `md.comp.{outlined,filled}-select.*`) with a listbox overlay. |
| RadioGroup | `ecosystem/fret-ui-material3/src/radio.rs` | Yes (`RadioStyle`) | Yes | Yes | Group is implemented by composing `Radio` items and forwarding `RadioStyle` into each item. |
| Button | `ecosystem/fret-ui-material3/src/button.rs` | Yes (`ButtonStyle`) | Yes | Yes | Style overrides apply to the existing token-derived defaults. |
| IconButton | `ecosystem/fret-ui-material3/src/icon_button.rs` | Yes (`IconButtonStyle`) | Yes | Yes | Supports toggle `selected` via `WidgetStates::SELECTED`. |
| Checkbox | `ecosystem/fret-ui-material3/src/checkbox.rs` | Yes (`CheckboxStyle`) | Yes | Yes | Exposes container/outline/icon/state-layer color overrides; maps `checked` to `WidgetStates::SELECTED`. |
| Switch | `ecosystem/fret-ui-material3/src/switch.rs` | Yes (`SwitchStyle`) | Yes | Yes | Exposes track/handle/outline/state-layer color overrides; maps `selected` to `WidgetStates::SELECTED`. |
| Radio | `ecosystem/fret-ui-material3/src/radio.rs` | Yes (`RadioStyle`) | Yes | Yes | Exposes icon + state-layer color overrides; maps `checked` to `WidgetStates::SELECTED`. |
| Tabs | `ecosystem/fret-ui-material3/src/tabs.rs` | Yes (`TabsStyle`) | Yes | Yes | Overrides: container/label/state-layer/active-indicator colors; maps active tab to `WidgetStates::SELECTED`. |
| TextField | `ecosystem/fret-ui-material3/src/text_field.rs` | Yes (`TextFieldStyle`) | Yes | Yes | Keeps `error` as a bespoke boolean; style overrides apply to the existing token-derived defaults. |
| Menu | `ecosystem/fret-ui-material3/src/menu.rs` | No | N/A | No | Policy-heavy; needs a careful, minimal override surface if we expose one. |
| Dialog | `ecosystem/fret-ui-material3/src/dialog.rs` | No | N/A | No | Same as Menu (overlay surface + motion + focus). |
| Tooltip | `ecosystem/fret-ui-material3/src/tooltip.rs` | No | N/A | No | Often provider-driven; may stay policy-only in v1. |
| Snackbar | `ecosystem/fret-ui-material3/src/snackbar.rs` | No | N/A | No | Typically a higher-level pattern; likely v2. |

## Implementation plan

Status legend: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

### M3SA-000 — Doc alignment and tracking

- [x] Choose (B): keep `docs/workstreams/state-driven-style-resolution-v1.md` focused on cross-ecosystem primitives; track Material 3 consumer alignment here.
- [x] Remove deprecated v0 Material 3 token lists from the SDSR workstream (they used `material3.*` keys and are no longer authoritative).
- [x] Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` evidence for ADR 1159 to include current Material 3 adoption.

### M3SA-005 — Remove or migrate legacy pilot modules

There were early, ADR-1159-shaped experiments in-tree (`select.rs`, `radio_group.rs`) that were
not exported from the crate and did not follow the current `md.*` token namespace decision.

- [x] Delete these modules entirely (they were not exported and used deprecated `material3.*` keys).
- [x] Re-introduce `Select` on top of the current foundation + `md.*` tokens.
  - Evidence: `ecosystem/fret-ui-material3/src/select.rs`, `ecosystem/fret-ui-material3/src/tokens/select.rs`.

### M3SA-010 — Shared resolution helpers (kit-level)

Add small, design-system-agnostic helpers (prefer `fret-ui-kit`) to reduce per-component boilerplate:

- [x] `resolve_override_slot*` helpers in `fret-ui-kit` (including computed-default variants `resolve_override_slot_with` / `resolve_override_slot_opt_with`).
- [x] `merge_override_slot(self.field, other.field)` helper to standardize right-biased merge.

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
- [x] Confirm how `WidgetState::Open` / `Selected` are used (e.g. menus/selects/tabs).

Widget state conventions (v1):

- `WidgetStates::OPEN` is used for "overlay is open / expanded" presentation state of a *trigger*
  (e.g. Select trigger, Menu trigger). It is not a selection state.
- `WidgetStates::SELECTED` is used for "this option/tab is the current value" (e.g. Select option,
  Tabs active tab) and for boolean "checked/on" controls (Checkbox/Switch/Radio/IconButton toggles).
- Overlays (Menu/Select listbox) should usually treat `SELECTED` as belonging to the *option row*,
  not to the overlay container.

### M3SA-200 — Implement `*Style` surfaces per component (incremental)

Per component:

- [x] Export `*Style` in the module and in `ecosystem/fret-ui-material3/src/lib.rs` as needed (done: Button/IconButton/Checkbox/Switch/Radio/Tabs/TextField).
- [x] Add `.style(style)` builder to the component (done: Button/IconButton/Checkbox/Switch/Radio/Tabs/TextField).
- [x] Resolve theme-derived defaults first, then apply overrides at resolve-time (done: Button/IconButton/Checkbox/Switch/Radio/Tabs/TextField).
- [x] Keep foundation-owned behavior (indication, motion scheme, token fallback chain) unchanged.

Recommended layering order (per slot):

1. Token-derived defaults (`md.comp.*` → `md.sys.*`)
2. Material tree-local context overrides (content color / motion scheme / ripple config) when applicable
3. Component `*Style` overrides (ADR 1159 shape)

Recommended order:

- [x] TextField (high value; currently bespoke)
- [x] Button
- [x] Checkbox / Switch / Radio
- [x] Tabs
- [ ] Menu / Dialog surfaces (only if the override surface remains small)

### M3SA-300 — Gallery validation pages

For each component that gains a `*Style` surface:

- [x] Add a “Default vs Override” comparison block in `apps/fret-ui-gallery` (done: Button/IconButton/Checkbox/Switch/Radio/Tabs/TextField).
- [x] Include at least one partial override example (hover-only or focus-ring-only) (done: Button/IconButton/Checkbox/Switch/Radio/Tabs/TextField).

### M3SA-400 — Decide how to model “invalid/error” (TextField)

We need a cross-ecosystem answer for “error” styling without exploding the state vocabulary.

Options:

- Option A (v1): keep `error` as a component-specific boolean/variant and do not add a new widget state.
- Option B (v2): introduce `WidgetState::Invalid` (or a small extension mechanism) in kit-level primitives.

- [x] Pick Option A (v1): keep `error` as a component-specific boolean and do not add a new widget state; revisit `WidgetState::Invalid` in v2 if multiple ecosystems need it.

## Evidence anchors

- ADR 1158: `docs/adr/1158-state-driven-style-resolution-v1.md`
- ADR 1159: `docs/adr/1159-ecosystem-style-override-surface-v1.md`
- Kit primitive: `ecosystem/fret-ui-kit/src/style/state.rs`
- Shared patterns: `docs/shadcn-style-override-patterns.md`
- Material foundation: `ecosystem/fret-ui-material3/src/foundation/*`
- Material components: `ecosystem/fret-ui-material3/src/*.rs`
- Gallery: `apps/fret-ui-gallery/src/{spec.rs,ui.rs,docs.rs}`
