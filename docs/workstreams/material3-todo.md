# Material 3 / Expressive Alignment (TODO)

Status: Draft / work-in-progress

This workstream tracks **visual + interaction outcome alignment** for Material Design 3 (and
Material 3 Expressive) in Fret.

## Refactor Plan (read first)

We are doing a “fearless refactor” to reduce divergence across components by introducing a shared
Material foundation layer (interaction/indication/token resolution) inspired by Compose.

- Plan: `docs/workstreams/material3-refactor-plan.md`

## Goals

- Provide a single crate surface: `ecosystem/fret-ui-material3`.
- Align **interaction outcomes** (hover/pressed state layers, focus treatment, ripples, overlay
  dismissal + focus trap/restore, motion curves).
- Align **visual outcomes** via Material tokens (color, shape, elevation, typography, motion).
- Keep `crates/fret-ui` focused on mechanisms; Material policies live in the ecosystem.

## Non-goals (initially)

- Perfect parity with `@material/web` implementation details (DOM/Lit behavior).
- Full accessibility parity on day one (we will converge via a11y contracts + tests).

## References (pinned in-repo)

- Material Web Components: `repo-ref/material-web`
  - Tokens (including Expressive in `versions/v30_0`): `repo-ref/material-web/tokens`
- Bevy MD3 replication (outcomes-first, non-DOM): `repo-ref/bevy_material_ui`
- Compose Multiplatform Material3 (interaction + indication + token patterns):
  `repo-ref/compose-multiplatform-core/compose/material3`
- Fret overlay policy boundary: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Fret theme tokens: `docs/adr/0032-style-tokens-and-theme-resolution.md`,
  `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`
- Interactivity pseudoclasses contract (hover/pressed as paint-only): `docs/adr/0181-interactivity-pseudoclasses-and-structural-stability.md`

## Tracking Checklist

## Progress (completed)

- [x] ThemeConfig v2 supports extra token kinds (number/duration/easing/text-style).
  - Evidence: `crates/fret-ui/src/theme.rs` (`ThemeConfig`, `Theme::{number_by_key,...}`),
    `crates/fret-ui/src/theme_registry.rs` (`ThemeTokenKind`),
    `crates/fret-core/src/geometry.rs` (`Px` serde),
    `crates/fret-core/src/text.rs` (`TextStyle` serde),
    tests in `crates/fret-ui/src/theme.rs` (`theme_config_v2_parses_additional_token_kinds`).
- [x] Mechanism-level state layer painting primitive.
  - Evidence: `crates/fret-ui/src/paint.rs` (`paint_state_layer`),
    tests in `crates/fret-ui/src/paint.rs` (`paint_state_layer_emits_single_quad_with_expected_alpha`).

### Refactor Execution (Material foundation)

- [x] Introduce Material foundation modules for shared ink + geometry.
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/mod.rs`,
    `ecosystem/fret-ui-material3/src/foundation/indication.rs`,
    `ecosystem/fret-ui-material3/src/foundation/geometry.rs`.
- [x] Migrate `Button` and `Tabs` to the foundation indication path and remove non-Material fallbacks.
  - Evidence: `ecosystem/fret-ui-material3/src/button.rs`,
    `ecosystem/fret-ui-material3/src/tabs.rs`.
- [x] Migrate `Checkbox` and `IconButton` to the foundation indication path and remove non-Material fallbacks.
  - Evidence: `ecosystem/fret-ui-material3/src/checkbox.rs`,
    `ecosystem/fret-ui-material3/src/icon_button.rs`.
- [x] Migrate `Switch` to the foundation indication path and remove non-Material fallbacks.
  - Evidence: `ecosystem/fret-ui-material3/src/switch.rs`,
    `ecosystem/fret-ui-material3/src/foundation/indication.rs` (ripple-bounds support).
- [x] Migrate `Menu` to the foundation indication path and remove non-Material fallbacks.
  - Evidence: `ecosystem/fret-ui-material3/src/menu.rs`.
- [x] Migrate `Radio` to the foundation indication path and remove non-Material fallbacks.
  - Evidence: `ecosystem/fret-ui-material3/src/radio.rs`.
- [ ] Migrate the remaining components and delete duplicated per-component helpers.

## Audit Anchors (Fret)

- Theme system (v1): `crates/fret-ui/src/theme.rs`, `crates/fret-ui/src/theme_registry.rs`
- Focus-visible + focus rings: `crates/fret-ui/src/focus_visible.rs`, `crates/fret-ui/src/paint.rs`
- Shadows/elevation primitive: `crates/fret-ui/src/paint.rs`
- Overlay mechanisms: `crates/fret-ui/src/tree/*` + `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Interactivity pseudoclasses contract: `docs/adr/0181-interactivity-pseudoclasses-and-structural-stability.md`

### Token System

- [ ] Decide canonical token key namespace for Material (proposal: `md.sys.*`, `md.comp.*`).
- [x] Provide a baseline, hand-authored v30 token preset injection (state/motion/shape/typescale subset).
  - Evidence: `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_tokens`, `theme_config`).
- [x] Inject `md.sys.color.*` via dynamic color scheme generation (including Expressive variant).
  - Evidence: `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_sys_colors`,
    `ColorSchemeOptions`, `DynamicVariant`, `theme_config_with_colors`).
- [ ] Implement import pipeline from `repo-ref/material-web/tokens/versions/v30_0` into Fret theme configs.
- [x] Add support for non-color/non-px token kinds needed by Material:
  - [x] scalar numbers (e.g. state-layer opacity)
  - [x] durations (ms)
  - [x] easing curves (cubic-bezier)
  - [x] typescale/text styles (family/size/line-height/weight)

### Runtime Surface (core changes)

- [x] Extend `ThemeConfig`/`Theme` to query typed values beyond `Color` and `Px`.
- [x] Provide a mechanism-level primitive for state layers (paint-only overlay).
- [x] Provide a mechanism-level primitive for ripple painting (policy in ecosystem).
  - Evidence: `crates/fret-ui/src/paint.rs` (`paint_ripple`) + unit tests.

### Interaction Outcomes

- [x] State layer policy (hover/pressed/focus) with v30 sys opacities.
  - Evidence: `ecosystem/fret-ui-material3/src/button.rs` (`state_layer_target_opacity`),
    `ecosystem/fret-ui-material3/src/interaction/state_layer.rs` (`StateLayerAnimator`),
    `crates/fret-ui/src/paint.rs` (`paint_state_layer`).
- [x] Ripple policy (bounded, pointer-origin) wired to mechanism primitive.
  - Evidence: `ecosystem/fret-ui-material3/src/interaction/ripple.rs` (`RippleAnimator`),
    `ecosystem/fret-ui-material3/src/button.rs` (`PointerRegion` → `PointerRegionState.last_down`),
    `crates/fret-ui/src/paint.rs` (`paint_ripple`).
- [ ] Ripple (unbounded, keyboard click synthesis, spec fade rules).
- [ ] Focus ring style and focus-visible heuristics aligned with Material expectations.
- [ ] Overlay outcomes (menu, dialog, tooltip):
  - [x] Escape dismissal (menu dropdown)
    - Evidence: `ecosystem/fret-ui-material3/src/dropdown_menu.rs` (OverlayRequest::dismissible_menu)
  - [x] outside press dismissal (menu dropdown)
    - Evidence: `ecosystem/fret-ui-material3/src/dropdown_menu.rs` (OverlayRequest::dismissible_menu)
  - [ ] focus trap/restore (modal)
  - [ ] click-through semantics (non-modal)

### Visual Outcomes

- [ ] Elevation mapping (MD3 levels → shadow parameters).
- [ ] Shape mapping (corner tokens, per-state expressive shape where applicable).
- [ ] Typography mapping (typescale roles).

### Component Surface (MVP set)

- [x] Button (MVP: filled/tonal/elevated/outlined/text, state layer + bounded ripple)
  - Evidence: `ecosystem/fret-ui-material3/src/button.rs` (`Button`, `ButtonVariant`).
- [x] Icon button (MVP: standard/filled/tonal/outlined, state layer + bounded ripple, optional toggle colors)
  - Evidence: `ecosystem/fret-ui-material3/src/icon_button.rs` (`IconButton`, `IconButtonVariant`).
- [x] Checkbox (MVP: bool-only, state layer + bounded ripple)
  - Evidence: `ecosystem/fret-ui-material3/src/checkbox.rs` (`Checkbox`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_checkbox_*`).
- [x] Switch (MVP: bool-only, thumb state layer + bounded ripple)
  - Evidence: `ecosystem/fret-ui-material3/src/switch.rs` (`Switch`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_switch_*`).
- [x] Radio (MVP: bool/group-value binding, state layer + bounded ripple, dot grow animation)
  - Evidence: `ecosystem/fret-ui-material3/src/radio.rs` (`Radio`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_radio_button_*`).
- [x] Radio group (MVP: `RadioGroup` semantics + roving focus + APG arrow/Home/End navigation + prefix typeahead)
  - Evidence: `ecosystem/fret-ui-material3/src/radio.rs` (`RadioGroup`, `RadioGroupItem`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_radio`).
- [x] Text field (MVP: outlined + filled chrome + label motion; outlined notch patch; filled hover state layer; hover/focus/error/disabled styling)
  - Evidence: `ecosystem/fret-ui-material3/src/text_field.rs` (`TextField`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_(outlined|filled)_text_field_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_text_field`).
- [x] Tabs (MVP: roving focus + state layer + bounded ripple + active indicator)
  - Evidence: `ecosystem/fret-ui-material3/src/tabs.rs` (`Tabs`, `TabItem`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_primary_navigation_tab_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_tabs`).
- [x] Menu (MVP: in-place list + dropdown overlay, roving focus + prefix typeahead, state layer + bounded ripple)
  - Evidence: `ecosystem/fret-ui-material3/src/menu.rs` (`Menu`, `MenuItem`, `roving_typeahead_prefix_arc_str_always_wrap`),
    `ecosystem/fret-ui-material3/src/dropdown_menu.rs` (`DropdownMenu`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_menu_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_menu`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_MENU`).
- [ ] List (standalone primitive; icons/selection density; shared with menu)
- [ ] Dialog / Snackbar / Tooltip (pick order based on demos)

### Conformance / Regression

- [x] Add a Material 3 gallery page for manual interaction verification.
  - Evidence: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_BUTTON`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_button`),
    `apps/fret-ui-gallery/src/driver.rs` (v30 token injection via `extend_tokens_from_config`).
- [x] Add a Material 3 IconButton gallery page for manual interaction verification.
  - Evidence: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_ICON_BUTTON`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_icon_button`).
- [x] Add a Material 3 Checkbox gallery page for manual interaction verification.
  - Evidence: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_CHECKBOX`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_checkbox`).
- [x] Add a Material 3 Radio gallery page for manual interaction verification.
  - Evidence: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_RADIO`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_radio`).
- [x] Add a Material 3 Text field gallery page for manual interaction verification.
  - Evidence: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_TEXT_FIELD`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_text_field`).
- [ ] Add a small, scripted interaction test harness for Material states (hover/press/ripple).
- [ ] Add golden-style visual snapshots per component state (light/dark, density variants).

## Proposed ADRs (drafts)

- `docs/adr/1158-theme-value-kinds-and-themeconfig-v2.md`
- `docs/adr/1159-material3-state-layer-and-ripple-primitives.md`
