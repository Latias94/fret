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

- [x] ThemeConfig v2 supports extra token kinds (number/duration/easing/text-style/corners).
  - Evidence: `crates/fret-ui/src/theme.rs` (`ThemeConfig`, `Theme::{number_by_key,...}`),
    `crates/fret-ui/src/theme_registry.rs` (`ThemeTokenKind`),
    `crates/fret-core/src/geometry.rs` (`Px` serde),
    `crates/fret-core/src/geometry.rs` (`Corners` serde),
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
- [x] Remove non-Material fallbacks from `TextField` (foundation migration TBD).
  - Evidence: `ecosystem/fret-ui-material3/src/text_field.rs`.
- [x] Add a regression test to prevent non-Material token fallbacks.
  - Evidence: `ecosystem/fret-ui-material3/src/lib.rs` (`material3_component_sources_do_not_fallback_to_non_material_tokens`).
- [x] Add scene-level regression tests for radio alignment (dot + ripple origin).
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs`.
- [x] Add a token audit tool (coverage report vs code + material-web sassvars).
  - Evidence: `ecosystem/fret-ui-material3/src/bin/material3_token_audit.rs`.
- [x] Introduce strict token resolver + content defaults (foundation).
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`,
    `ecosystem/fret-ui-material3/src/foundation/content.rs`.
- [x] Add a Material 3 "state matrix" gallery page for manual regression.
  - Evidence: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_STATE_MATRIX`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_state_matrix`).
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
- [x] Align Switch handle tokens with Material Web v30 (`*.handle.width/height`, not `*.handle.size`).
  - Evidence: `ecosystem/fret-ui-material3/src/switch.rs` (`switch_size_tokens`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_switch_scalars`),
    `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-switch.scss`.
- [x] Inject `md.sys.color.*` via dynamic color scheme generation (including Expressive variant).
  - Evidence: `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_sys_colors`,
    `ColorSchemeOptions`, `DynamicVariant`, `theme_config_with_colors`).
- [ ] Implement import pipeline from `repo-ref/material-web/tokens/versions/v30_0` into Fret theme configs.
  - [x] Generate sys motion + state (+ focus-indicator) injectors from Material Web sassvars.
    - Evidence: `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs`,
      `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`,
      `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_tokens`).
  - [x] Generate sys shape injectors from Material Web sassvars (including corner sets).
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_sys_shape`),
      `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_tokens`),
      `crates/fret-ui/src/theme.rs` (`ThemeConfig.corners`).
  - [x] Import typescale tokens (`md.sys.typescale.*`) into `ThemeConfig.text_styles`.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_sys_typescale`),
      `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`TypographyOptions`).
  - [ ] Import the subset of `md.comp.*` tokens used by MVP components (drive by `material3_token_audit`).
    - [x] Import `md.comp.button.*` scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_button_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_button_scalars`).
    - [x] Import `md.comp.checkbox.*` scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_checkbox_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_checkbox_scalars`).
    - [x] Import `md.comp.switch.*` scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_switch_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_switch_scalars`).
    - [x] Import `md.comp.icon-button.*` scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_icon_button_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_icon_button_scalars`).
    - [x] Import `md.comp.primary-navigation-tab.*` scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_primary_navigation_tab_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_primary_navigation_tab_scalars`).
    - [x] Import `md.comp.menu.*` scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_menu_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_menu_scalars`).
    - [x] Import `md.comp.(full-screen-)?dialog.*` scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_dialog_scalars`,
        `inject_comp_full_screen_dialog_scalars`), `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_dialog_scalars`,
        `inject_comp_full_screen_dialog_scalars`).
    - [x] Import `md.comp.(outlined|filled)-text-field.*` scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_outlined_text_field_scalars`,
        `inject_comp_filled_text_field_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_outlined_text_field_scalars`,
        `inject_comp_filled_text_field_scalars`).
    - [ ] Expand scalar import coverage for other MVP components.
    - [x] Represent corner sets via `ThemeConfig.corners` (per-corner radii); other structured tokens TBD.
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

### Material Foundation Backlog (Compose baseline)

- [x] Add a tree-local Material context provider (theme-ish overrides) in `fret-ui-material3`:
  - content defaults (Compose `LocalContentColor` analogue),
  - ripple configuration escape hatch (Compose `LocalRippleConfiguration` analogue),
  - motion scheme override (Compose `LocalMotionScheme` analogue) (scheme selection; expressive tokens TBD).
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/context.rs`
- [x] Add a minimal layout probe helper for measurement-driven visuals (1-frame latency).
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/layout_probe.rs`,
    `ecosystem/fret-ui-material3/src/tabs.rs`
- [ ] Decide whether we need a hoistable interaction source surface (Compose
  `MutableInteractionSource` analogue) or whether `PressableState` + foundation runtime state is
  sufficient for our current authoring model.
- [x] Implement a MotionScheme mapping for the 6 canonical specs (standard) and expose it via the
  tree-local Material context override.
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/motion_scheme.rs`,
    `ecosystem/fret-ui-material3/src/foundation/context.rs`
- [ ] Extend MotionScheme mapping for Expressive tokens (when available in the token source of truth).
- [ ] Decide how to represent spring configs long-term (ecosystem-only vs core mechanism).
- [ ] Introduce typed token modules per component to reduce raw string key usage and centralize
  derived token math (disabled alpha, state-layer alpha selection).

### Interaction Outcomes

- [x] State layer policy (hover/pressed/focus) with v30 sys opacities.
  - Evidence: `ecosystem/fret-ui-material3/src/button.rs` (`state_layer_target_opacity`),
    `ecosystem/fret-ui-material3/src/interaction/state_layer.rs` (`StateLayerAnimator`),
    `crates/fret-ui/src/paint.rs` (`paint_state_layer`).
- [x] Ripple policy (pointer-origin + fallback-to-center) wired to mechanism primitive.
  - Evidence: `ecosystem/fret-ui-material3/src/interaction/ripple.rs` (`RippleAnimator`),
    `ecosystem/fret-ui-material3/src/button.rs` (`PointerRegion` → `PointerRegionState.last_down`),
    `ecosystem/fret-ui-material3/src/foundation/geometry.rs` (`down_origin_local`),
    `crates/fret-ui/src/paint.rs` (`paint_ripple`).
- [ ] Ripple parity improvements (unbounded clip, token radius, spec fade rules).
  - Evidence (partial): `ecosystem/fret-ui-material3/src/foundation/indication.rs` (`RippleClip`),
    `ecosystem/fret-ui-material3/src/checkbox.rs` (unbounded ripple),
    `ecosystem/fret-ui-material3/src/radio.rs` (unbounded ripple),
    `ecosystem/fret-ui-material3/src/switch.rs` (unbounded ripple).
- [x] Focus ring style and focus-visible heuristics aligned with Material expectations.
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/focus_ring.rs` (`material_focus_ring_for_component`),
    `crates/fret-ui/src/declarative/host_widget/paint.rs` (`paint_focus_ring` gated by `focus_visible::is_focus_visible`).
- [x] Transition timelines support theme cubic-bezier easing (overlay motion parity).
  - Evidence: `ecosystem/fret-ui-headless/src/transition.rs` (`update_with_cubic_bezier`),
    `ecosystem/fret-ui-kit/src/declarative/transition.rs` (`drive_transition_with_durations_and_cubic_bezier`),
    `ecosystem/fret-ui-kit/src/overlay_controller.rs` (`transition_with_durations_and_cubic_bezier`),
    `ecosystem/fret-ui-material3/src/dialog.rs` (scrim fade + panel scale/translate transition),
    `ecosystem/fret-ui-material3/src/dropdown_menu.rs` (menu transition),
    `ecosystem/fret-ui-material3/src/tooltip.rs` (tooltip transition),
    `ecosystem/fret-ui-material3/src/snackbar.rs` (toast-layer motion tokens).
- [ ] Overlay outcomes (menu, dialog, tooltip):
  - [x] Escape dismissal (menu dropdown)
    - Evidence: `ecosystem/fret-ui-material3/src/dropdown_menu.rs` (OverlayRequest::dismissible_menu)
  - [x] outside press dismissal (menu dropdown)
    - Evidence: `ecosystem/fret-ui-material3/src/dropdown_menu.rs` (OverlayRequest::dismissible_menu)
  - [x] focus trap/restore (modal) (currently validated via modal navigation drawer)
    - Evidence: `ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs` (focus trap),
      `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (focus restore),
      `ecosystem/fret-ui-primitives/src/focus_scope.rs` (`FocusScopeProps { trap_focus: true }`).
  - [ ] click-through semantics (non-modal)

### Visual Outcomes

- [x] Elevation mapping (MD3 levels → shadow parameters).
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/elevation.rs` (`shadow_for_elevation_with_color`),
    `ecosystem/fret-ui-material3/src/menu.rs` (`ContainerProps.shadow`),
    `crates/fret-ui/src/paint.rs` (`paint_shadow`).
- [ ] Shape mapping (corner tokens, per-state expressive shape where applicable).
  - [x] Corner set tokens (`md.sys.shape.corner.*.(top|start|end)`) and component shapes that depend on them.
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
- [x] Navigation bar (MVP: roving focus + state layer + bounded ripple + active indicator)
  - Evidence: `ecosystem/fret-ui-material3/src/navigation_bar.rs` (`NavigationBar`, `NavigationBarItem`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_navigation_bar_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_navigation_bar`).
- [x] Navigation rail (MVP: roving focus + state layer + bounded ripple + active indicator)
  - Evidence: `ecosystem/fret-ui-material3/src/navigation_rail.rs` (`NavigationRail`, `NavigationRailItem`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_navigation_rail_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_navigation_rail`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_NAVIGATION_RAIL`).
- [x] Navigation drawer (MVP: roving focus + state layer + bounded ripple + selected pill background)
  - Evidence: `ecosystem/fret-ui-material3/src/navigation_drawer.rs` (`NavigationDrawer`, `NavigationDrawerItem`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_navigation_drawer_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_navigation_drawer`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_NAVIGATION_DRAWER`).
- [x] Modal navigation drawer (MVP: modal overlay + scrim + slide-in motion + focus trap/restore)
  - Evidence: `ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs` (`ModalNavigationDrawer`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_modal_navigation_drawer`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_MODAL_NAVIGATION_DRAWER`).
- [x] Menu (MVP: in-place list + dropdown overlay, roving focus + prefix typeahead, state layer + bounded ripple)
  - Evidence: `ecosystem/fret-ui-material3/src/menu.rs` (`Menu`, `MenuItem`, `roving_typeahead_prefix_arc_str_always_wrap`),
    `ecosystem/fret-ui-material3/src/dropdown_menu.rs` (`DropdownMenu`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_menu_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_menu`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_MENU`).
- [ ] List (standalone primitive; icons/selection density; shared with menu)
- [x] Dialog (MVP: modal overlay + scrim + focus trap/restore + dialog actions)
  - Evidence: `ecosystem/fret-ui-material3/src/dialog.rs` (`Dialog`, `DialogAction`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_dialog`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_DIALOG`).
- [x] Tooltip (MVP: plain tooltip, delay group + hover intent + safe-hover corridor, token-driven styling)
  - Evidence: `ecosystem/fret-ui-material3/src/tooltip.rs` (`PlainTooltip`, `TooltipProvider`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_plain_tooltip_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_tooltip`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_TOOLTIP`).
- [x] Snackbar (MVP: toast-layer skin using `md.comp.snackbar.*` tokens, action + dismiss icon)
  - Evidence: `ecosystem/fret-ui-material3/src/snackbar.rs` (`SnackbarHost`, `SnackbarController`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_snackbar_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_snackbar`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_SNACKBAR`).

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
