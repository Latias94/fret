# Material 3 / Expressive Alignment (TODO)

Status: Work-in-progress

This workstream tracks **visual + interaction outcome alignment** for Material Design 3 (and
Material 3 Expressive) in Fret.

## Refactor Plan (read first)

We are doing a “fearless refactor” to reduce divergence across components by introducing a shared
Material foundation layer (interaction/indication/token resolution) inspired by Compose.

- Plan: `docs/workstreams/material3-refactor-plan.md`
- Style API alignment (cross-ecosystem interfaces): `docs/workstreams/material3-style-api-alignment-v1.md`

## Related Workstreams

Material3 alignment depends on the repository’s shared “state → style” infrastructure workstream.
Prefer reusing these primitives over re-inventing per-component state precedence rules:

- State-driven style resolution v1: `docs/workstreams/state-driven-style-resolution-v1.md`
  - Contract gate: `docs/adr/1158-state-driven-style-resolution-v1.md`
  - Ecosystem override surface: `docs/adr/1159-ecosystem-style-override-surface-v1.md`

## Goals

- Provide a single crate surface: `ecosystem/fret-ui-material3`.
- Align **interaction outcomes** (hover/pressed state layers, focus treatment, ripples, overlay
  dismissal + focus trap/restore, motion curves).
- Align **visual outcomes** via Material tokens (color, shape, elevation, typography, motion).
- Keep `crates/fret-ui` focused on mechanisms; Material policies live in the ecosystem.

## Expressive (component variants)

- [x] List: expressive icon size + per-interaction container shape.
  - Evidence: `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_list_scalars`),
    `ecosystem/fret-ui-material3/src/tokens/list.rs` (`item_container_shape_for_interaction`),
    `ecosystem/fret-ui-material3/src/list.rs` (design variant aware shape selection),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_list`).
- [ ] Expand Expressive coverage incrementally without inventing Fret-only component tokens:
  - Treat `DynamicVariant::Expressive` as the source of truth for **palette/scheme** changes
    (`md.sys.color.*`), independent of per-component `.expressive.*` tokens.
  - As of Material Web v30 sassvars, `.expressive.` component tokens are only present for `List`
    (shape + icon sizes). Do not add placeholder expressive tokens for other components yet.
  - When upstream adds expressive component tokens, implement them by:
    - importing via `material3_token_import` into `tokens/material_web_v30.rs`, and
    - plumbing `MaterialDesignVariant` through the relevant typed token modules (like `tokens/list.rs`),
      keeping component recipes unchanged.

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

## Compose Baseline: Infrastructure vs Components

This section exists to keep us honest about **what should be shared infrastructure** (to avoid
per-component drift) vs **what should stay inside component recipes**.

Compose Material3 provides a useful “how it’s factored” reference even though our runtime is not
Compose. The key point is not copying APIs, but copying **which behaviors are centralized**.

### Compose “infrastructure” (centralized policy)

These files are primarily *shared policy primitives*, not one-off component layouts:

- Theme + tokens: `MaterialTheme.kt`, `ColorScheme.kt`, `TonalPalette.kt`, `Shapes.kt`, `Typography.kt`
- Scoped defaults: `ContentColor.kt` (Compose `LocalContentColor`), `Text.kt` (default text style helpers)
- Ink + interactions: `Ripple.kt` (Indication), `PrecisionPointer.kt` (hover-capable pointer types)
- Motion: `MotionScheme.kt` (spatial vs effects specs)
- Touch target: `InteractiveComponentSize.kt` (minimum touch target policy)
- Surfaces: `Surface.kt` (elevation/tonal overlay/shadow conventions)

### Fret mapping (where each thing should live)

- `crates/fret-ui` (mechanisms)
  - Pointer classification + hover semantics (precision pointers; ignore touch for hover)
  - Paint primitives that are design-system agnostic (state layer / ripple)
  - Layout + rounding guarantees when required (pixel snapping, stable structure guidance)
- `ecosystem/fret-ui-material3` foundation (Material policy)
  - Token namespaces + strict fallback chain (`md.comp.*` → `md.sys.*`)
  - Indication orchestration (pressed/hover/focus state layer + ripple rules)
  - MotionScheme mapping (token numbers → animator configs)
  - Content defaults (Material `contentColor` conventions; disabled alpha)
  - Interactive size policy (min touch target enforcement)
  - Elevation policy (MD3 levels → shadows + tonal overlays)
- `ecosystem/fret-ui-material3` components (recipes)
  - Structure/layout + semantics + focus wiring
  - Measurement-driven visuals (indicator placement, thumb bounds) via shared probes

### Backlog extracted from the comparison

- [ ] Add a `LocalTextStyle`-like scoped default helper in Material foundation (text style + icon size).
- [x] Land `foundation::elevation` (shadow + tonal overlay) and migrate `Surface`-like containers.
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/elevation.rs`, `ecosystem/fret-ui-material3/src/foundation/surface.rs`,
    `ecosystem/fret-ui-material3/src/dialog.rs`, `ecosystem/fret-ui-material3/src/menu.rs`, `ecosystem/fret-ui-material3/src/tooltip.rs`,
    `ecosystem/fret-ui-material3/src/navigation_bar.rs`, `ecosystem/fret-ui-material3/src/navigation_drawer.rs`.
- [ ] Decide the public surface for hoistable interaction sources (if any), and standardize “pressed origin” latching.
- [ ] Audit which parts of minimum touch target policy should become a core `fret-ui` mechanism vs remain Material-only.
- [ ] Decide whether we need a core pixel-snapping policy hook for non-1.0 scale factors (radio/checkbox drift class).

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
- [x] Centralize pressable indication timing defaults (durations + easing) to avoid per-component drift.
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/indication.rs` (`material_pressable_indication_config`).
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
- [x] Migrate `TextField` hover state layer to the foundation indication path.
  - Evidence: `ecosystem/fret-ui-material3/src/text_field.rs`,
    `ecosystem/fret-ui-material3/src/foundation/indication.rs`,
    `ecosystem/fret-ui-material3/tests/text_field_hover.rs` (filled/error hover overlay, focus indicator thickness, disabled/outlined invariants, overlay survives focus).
- [x] Add a regression test to prevent non-Material token fallbacks.
  - Evidence: `ecosystem/fret-ui-material3/src/lib.rs` (`material3_component_sources_do_not_fallback_to_non_material_tokens`).
- [x] Add scene-level regression tests for radio alignment (dot + ripple origin).
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs`.
- [x] Add a token audit tool (coverage report vs code + material-web sassvars).
  - Evidence: `ecosystem/fret-ui-material3/src/bin/material3_token_audit.rs` (filters allowlisted Fret-only keys,
    expands format-string templates via `expand_key_templates`, excludes `src/bin` from runtime scans, filters typography-only sassvar keys).
- [x] Introduce strict token resolver + content defaults (foundation).
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`,
    `ecosystem/fret-ui-material3/src/foundation/content.rs`.
- [x] Add a Material 3 "state matrix" gallery page for manual regression.
  - Evidence: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_STATE_MATRIX`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_state_matrix`).
- [x] Align core Material 3 components with the ecosystem `*Style` override surface (ADR 1159).
  - Evidence: `docs/workstreams/material3-style-api-alignment-v1.md`,
    `ecosystem/fret-ui-material3/src/{button,checkbox,icon_button,radio,switch,tabs,text_field}.rs`.
- [ ] Migrate the remaining components and delete duplicated per-component helpers.

## Audit Anchors (Fret)

- Theme system (v1): `crates/fret-ui/src/theme.rs`, `crates/fret-ui/src/theme_registry.rs`
- Focus-visible + focus rings: `crates/fret-ui/src/focus_visible.rs`, `crates/fret-ui/src/paint.rs`
- Shadows/elevation primitive: `crates/fret-ui/src/paint.rs`
- Overlay mechanisms: `crates/fret-ui/src/tree/*` + `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Interactivity pseudoclasses contract: `docs/adr/0181-interactivity-pseudoclasses-and-structural-stability.md`

### Token System

- [x] Decide canonical token key namespace for Material (proposal: `md.sys.*`, `md.comp.*`).
  - Evidence: literal key usage throughout `ecosystem/fret-ui-material3/src/*` and the strict
    resolver `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`.
- [x] Provide a baseline, hand-authored v30 token preset injection (state/motion/shape/typescale subset).
  - Evidence: `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_tokens`, `theme_config`).
- [x] Expand token audit coverage to validate format-string templates (variant/state keys) against v30 injection.
  - Evidence: `ecosystem/fret-ui-material3/src/bin/material3_token_audit.rs` (`expand_key_templates`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (icon-button hovered/focused/pressed icon tokens).
- [x] Align Switch handle tokens with Material Web v30 (`*.handle.width/height`, not `*.handle.size`).
  - Evidence: `ecosystem/fret-ui-material3/src/switch.rs` (`switch_size_tokens`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_switch_scalars`),
    `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-switch.scss`.
- [x] Inject `md.sys.color.*` via dynamic color scheme generation (including Expressive variant).
  - Evidence: `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_sys_colors`,
    `ColorSchemeOptions`, `DynamicVariant`, `theme_config_with_colors`; includes fixed roles like `md.sys.color.primary-fixed*`).
- [ ] Implement import pipeline from `repo-ref/material-web/tokens/versions/v30_0` into Fret theme configs.
  - [x] Auto-discover `repo-ref/material-web` in git worktrees (fallback to `MATERIAL_WEB_DIR`).
    - Evidence: `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs` (`default_material_web_dir`),
      `ecosystem/fret-ui-material3/src/bin/material3_token_audit.rs` (`resolve_material_web_dir`).
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
    - [x] Import `md.comp.radio-button.*` scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_radio_button_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_radio_button_scalars`).
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
    - [x] Import navigation surface scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_navigation_{bar,drawer,rail}_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_navigation_{bar,drawer,rail}_scalars`).
    - [x] Import `md.comp.menu.*` scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_menu_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_menu_scalars`).
    - [x] Import `md.comp.list.*` scalar tokens (non-color) from Material Web (includes Expressive shapes).
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_list_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_list_scalars`).
    - [x] Import tooltip/snackbar scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_{plain,rich}_tooltip_scalars`, `inject_comp_snackbar_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_{plain,rich}_tooltip_scalars`, `inject_comp_snackbar_scalars`).
    - [x] Import `md.comp.(full-screen-)?dialog.*` scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_dialog_scalars`,
        `inject_comp_full_screen_dialog_scalars`), `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_dialog_scalars`,
        `inject_comp_full_screen_dialog_scalars`).
    - [x] Import `md.comp.(outlined|filled)-text-field.*` scalar tokens (non-color) from Material Web.
      - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_outlined_text_field_scalars`,
        `inject_comp_filled_text_field_scalars`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_outlined_text_field_scalars`,
        `inject_comp_filled_text_field_scalars`).
    - [x] Generate `md.comp.*` color alias injectors from Material Web and apply them on top of
      dynamic `md.sys.color.*` generation (copy from `md.sys.color.*` / `md.ref.palette.*`).
      - Evidence: `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs`,
        `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_comp_*_colors_from_sys`),
        `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_*_colors_from_sys`).
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
- [x] Decide whether we need a hoistable interaction source surface (Compose
  `MutableInteractionSource` analogue) or whether `PressableState` + foundation runtime state is
  sufficient for our current authoring model.
  - Decision: keep `PressableState` + Material foundation state as the default for now; defer a
    hoistable interaction source until a concrete preview/authoring need appears.
- [x] Implement a MotionScheme mapping for the 6 canonical specs (standard) and expose it via the
  tree-local Material context override.
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/motion_scheme.rs`,
    `ecosystem/fret-ui-material3/src/foundation/context.rs`
- [x] Add a design variant selection mechanism (Standard vs Expressive) for component `.expressive.*`
  token variants (global default + subtree override).
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/context.rs` (`MaterialDesignVariant`,
    `theme_default_design_variant`, `with_material_design_variant`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`md.sys.fret.material.is-expressive`).
- [ ] Extend MotionScheme mapping for Expressive tokens (when available in the token source of truth).
- [ ] Decide how to represent spring configs long-term (ecosystem-only vs core mechanism).
- [ ] Introduce typed token modules per component to reduce raw string key usage and centralize
  derived token math (disabled alpha, state-layer alpha selection).
  - [x] IconButton token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/icon_button.rs`,
      `ecosystem/fret-ui-material3/src/icon_button.rs`.
  - [x] Checkbox token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/checkbox.rs`,
      `ecosystem/fret-ui-material3/src/checkbox.rs`.
  - [x] Button token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/button.rs`,
      `ecosystem/fret-ui-material3/src/button.rs`.
  - [x] Switch token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/switch.rs`,
      `ecosystem/fret-ui-material3/src/switch.rs`.
  - [x] Radio token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/radio.rs`,
      `ecosystem/fret-ui-material3/src/radio.rs`.
  - [x] Select token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/select.rs`,
      `ecosystem/fret-ui-material3/src/select.rs`.
  - [x] Tabs token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/tabs.rs`,
      `ecosystem/fret-ui-material3/src/tabs.rs`.
  - [x] Menu token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/menu.rs`,
      `ecosystem/fret-ui-material3/src/menu.rs`.
  - [x] TextField token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/text_field.rs`,
      `ecosystem/fret-ui-material3/src/text_field.rs`.
  - [x] List token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/list.rs`,
      `ecosystem/fret-ui-material3/src/list.rs`.
  - [x] NavigationDrawer token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/navigation_drawer.rs`,
      `ecosystem/fret-ui-material3/src/navigation_drawer.rs`.
  - [x] NavigationBar token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/navigation_bar.rs`,
      `ecosystem/fret-ui-material3/src/navigation_bar.rs`.
  - [x] NavigationRail token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/navigation_rail.rs`,
      `ecosystem/fret-ui-material3/src/navigation_rail.rs`.
  - [x] DropdownMenu token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/dropdown_menu.rs`,
      `ecosystem/fret-ui-material3/src/dropdown_menu.rs`.
  - [x] Dialog token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/dialog.rs`,
      `ecosystem/fret-ui-material3/src/dialog.rs`.
  - [x] Tooltip token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/tooltip.rs`,
      `ecosystem/fret-ui-material3/src/tooltip.rs`.
  - [x] Snackbar token keys + fallbacks centralized.
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/snackbar.rs`,
      `ecosystem/fret-ui-material3/src/snackbar.rs`.

### Interaction Outcomes

- [x] State layer policy (hover/pressed/focus) with v30 sys opacities.
  - Evidence: `ecosystem/fret-ui-material3/src/button.rs` (`state_layer_target_opacity`),
    `ecosystem/fret-ui-material3/src/interaction/state_layer.rs` (`StateLayerAnimator`),
    `crates/fret-ui/src/paint.rs` (`paint_state_layer`).
- [x] Hover semantics are precision-pointer only (ignore touch moves).
  - Evidence: `crates/fret-ui/src/tree/dispatch.rs` (gate hovered element updates by `PointerType`),
    tests in `crates/fret-ui/src/declarative/tests/interactions.rs` (`pressable_on_hover_change_hook_ignores_touch_pointer_move`),
    `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/PrecisionPointer.kt`.
- [x] Minimum interactive touch target (48dp) enforced for core pressables; visual chrome centered.
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/interactive_size.rs`,
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`md.sys.layout.minimum-touch-target.size`),
    `ecosystem/fret-ui-material3/src/checkbox.rs`,
    `ecosystem/fret-ui-material3/src/radio.rs`,
    `ecosystem/fret-ui-material3/src/switch.rs`,
    `ecosystem/fret-ui-material3/src/icon_button.rs`,
    `ecosystem/fret-ui-material3/src/tabs.rs`,
    `ecosystem/fret-ui-material3/src/menu.rs`,
    `ecosystem/fret-ui-material3/src/navigation_bar.rs`,
    `ecosystem/fret-ui-material3/src/navigation_rail.rs`,
    `ecosystem/fret-ui-material3/src/navigation_drawer.rs`,
    tests in `ecosystem/fret-ui-material3/src/lib.rs` (`material3_components_apply_minimum_touch_target_policy`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_touch_targets`),
    Compose reference: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/InteractiveComponentSize.kt`.
- [x] Ripple policy (pointer-origin + fallback-to-center) wired to mechanism primitive.
  - Evidence: `ecosystem/fret-ui-material3/src/interaction/ripple.rs` (`RippleAnimator`),
    `ecosystem/fret-ui-material3/src/button.rs` (`PointerRegion` → `PointerRegionState.last_down`),
    `ecosystem/fret-ui-material3/src/foundation/geometry.rs` (`rect_center`, `ripple_max_radius`),
    `crates/fret-ui/src/paint.rs` (`paint_ripple`).
- [x] Ripple parity improvements (keyboard origin, minimum press duration, bounds/clip, fade rules).
  - [x] Fade starts on release (no fade while held).
    - Evidence: `ecosystem/fret-ui-material3/src/interaction/ripple.rs` (`RippleAnimator::release`),
      tests in `ecosystem/fret-ui-material3/src/interaction/ripple.rs` (`ripple_does_not_fade_until_release`).
  - [x] Ripple color is latched at press start (avoids hover/focus color drift during fade).
    - Evidence: `ecosystem/fret-ui-material3/src/foundation/indication.rs` (passes color into `RippleAnimator::start`),
      `ecosystem/fret-ui-material3/src/interaction/ripple.rs` (`RipplePaintFrame.color`).
  - [x] Keyboard activation ripple origin ignores stale pointer down (falls back to state-layer center).
    - Evidence: `ecosystem/fret-ui-material3/src/foundation/indication.rs` (`RippleOrigin::Local`, tick-gated `last_down`),
      tests in `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`switch_keyboard_ripple_origin_ignores_stale_pointer_down`).
  - [x] Minimum press duration keeps pressed ripple visible for short taps/clicks.
    - Evidence: `ecosystem/fret-ui-material3/src/foundation/indication.rs` (`IndicationConfig.ripple_min_press_ms`),
      tests in `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`switch_ripple_holds_for_minimum_press_duration_before_fade`).
  - [x] Clip + bounds parity for state-layer ripples (Material Web-style bounded-by-state-layer).
    - Evidence: `ecosystem/fret-ui-material3/src/checkbox.rs`, `ecosystem/fret-ui-material3/src/radio.rs`,
      `ecosystem/fret-ui-material3/src/switch.rs` (use `RippleClip::Bounded` with circular state-layer bounds),
      `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (asserts `SceneOp::PushClipRRect` is emitted).
  - [x] Ripple bounds/origin coordinate space is consistent under nested layout offsets.
    - Evidence: `ecosystem/fret-ui-material3/src/foundation/geometry.rs` (`rect_center`, `ripple_max_radius`),
      `ecosystem/fret-ui-material3/src/foundation/indication.rs` (`advance_indication_for_pressable_with_ripple_bounds`),
      tests in `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`radio_ripple_origin_tracks_pointer_down_position`, `switch_ripple_origin_tracks_pointer_down_position`).
  - Evidence (partial): `ecosystem/fret-ui-material3/src/foundation/indication.rs` (`RippleClip`,
    `IndicationConfig.ripple_radius`, `IndicationConfig.ripple_min_press_ms`).
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
    - Evidence: `ecosystem/fret-ui-material3/src/dropdown_menu.rs` (OverlayRequest::dismissible_menu),
      `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`dropdown_menu_dismisses_and_restores_focus_across_schemes`).
  - [x] outside press dismissal (menu dropdown)
    - Evidence: `ecosystem/fret-ui-material3/src/dropdown_menu.rs` (OverlayRequest::dismissible_menu),
      `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`dropdown_menu_dismisses_and_restores_focus_across_schemes`).
  - [x] focus trap/restore (modal) (currently validated via modal navigation drawer)
    - Evidence: `ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs` (focus trap),
      `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (focus restore),
      `ecosystem/fret-ui-primitives/src/focus_scope.rs` (`FocusScopeProps { trap_focus: true }`).
  - [x] scrim defaults (modal navigation drawer)
    - Evidence: `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`md.comp.navigation-drawer.scrim.*` defaults),
      `ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs` (token lookup + fade),
      `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-navigation-drawer.scss` (token note).
  - [x] click-through semantics (non-modal)
    - Evidence: `ecosystem/fret-ui-material3/src/tooltip.rs` (`tooltip_prim::tooltip_request`),
      `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`tooltip_is_click_through_and_does_not_block_underlay_activation_across_schemes`).

### Visual Outcomes

- [x] Elevation mapping (shadow + tonal surface tint overlay).
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/elevation.rs` (`shadow_for_elevation_with_color`, `apply_surface_tint`, `apply_surface_tint_if_surface`),
    `ecosystem/fret-ui-material3/src/dialog.rs` (container surface tint + shadow),
    `ecosystem/fret-ui-material3/src/tooltip.rs` (container surface tint + shadow),
    `ecosystem/fret-ui-material3/src/navigation_bar.rs` (container surface tint + shadow),
    `ecosystem/fret-ui-material3/src/navigation_drawer.rs` (modal container elevation + shadow),
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
- [x] List (MVP: roving focus + selection follows focus + state layer + bounded ripple)
  - Evidence: `ecosystem/fret-ui-material3/src/list.rs` (`List`, `ListItem`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_list_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_list`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_LIST`).
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
- [x] Add a compact Material 3 Gallery page for scanning outcome drift.
  - Evidence: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_GALLERY`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_gallery`).
- [x] Add a gallery-level variant toggle (Standard vs Expressive) to exercise subtree overrides.
  - Evidence: `apps/fret-ui-gallery/src/driver.rs` (`UiGalleryWindowState.material3_expressive`),
    `apps/fret-ui-gallery/src/ui.rs` (`material3_scoped_page`).
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
- [x] Add a Material 3 Select gallery page for manual interaction verification.
  - Evidence: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_SELECT`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_select`).
- [x] Add a small, scripted interaction test harness for Material states (hover/press/ripple).
  - Evidence: `ecosystem/fret-ui-material3/tests/interaction_harness.rs` (`scene_signature`),
    tests in `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`tabs_pressed_scene_structure_is_stable`, `icon_button_pressed_scene_structure_is_stable`, `switch_pressed_scene_structure_is_stable`, `radio_pressed_scene_structure_is_stable`).
- [x] Extend the interaction harness with quad-level signatures to catch geometry jitter after animations settle.
  - Evidence: `ecosystem/fret-ui-material3/tests/interaction_harness.rs` (`scene_quad_geometry_signature`),
    tests in `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`*_pressed_scene_structure_is_stable`).
- [x] Verify Tabs pressed-scene stability across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`tabs_pressed_scene_structure_is_stable`).
- [x] Verify Radio pressed-scene stability across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`radio_pressed_scene_structure_is_stable`).
- [x] Verify IconButton pressed-scene stability across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`icon_button_pressed_scene_structure_is_stable`).
- [x] Verify Switch pressed-scene stability across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`switch_pressed_scene_structure_is_stable`).
- [x] Verify Checkbox pressed-scene stability across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`checkbox_pressed_scene_structure_is_stable`).
- [x] Verify Menu pressed-scene stability across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`menu_pressed_scene_structure_is_stable`).
- [x] Verify TextField hover/focus invariants across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/text_field_hover.rs` (multi-scheme regression matrix).
- [x] Verify modal overlay focus trap/restore across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`dialog_focus_is_contained_and_restored_across_schemes`,
    `modal_navigation_drawer_focus_is_contained_and_restored_across_schemes`).
- [x] Verify tooltip hover open/close and dropdown menu dismissal across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`tooltip_opens_and_closes_on_hover_across_schemes`,
    `dropdown_menu_dismisses_and_restores_focus_across_schemes`).
- [x] Verify Select dismissal and focus restore across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`select_dismisses_and_restores_focus_across_schemes`,
    `select_keyboard_open_sets_initial_focus_and_outside_dismiss_restores_focus_across_schemes`).
- [x] Add golden-style visual snapshots per component state (light/dark, density variants).
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_controls_suite_goldens_v1`, `material3_headless_overlays_suite_goldens_v1`, `material3_headless_text_field_suite_goldens_v1`),
    `goldens/material3-headless/v1/material3-*.json` (controls suite, overlay suite, text-field suite; includes `scale1_0`/`scale1_25`/`scale2_0` variants and overlay cases such as `both_open` + `select_open`).

## Proposed ADRs (drafts)

- `docs/adr/1158-theme-value-kinds-and-themeconfig-v2.md`
- `docs/adr/1159-material3-state-layer-and-ripple-primitives.md`
