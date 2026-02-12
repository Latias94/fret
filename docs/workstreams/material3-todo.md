# Material 3 / Expressive Alignment (TODO)

Status: Complete (MVP landed; follow-ups tracked in `docs/workstreams/material3-next-wave-v2.md`)

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
  - Contract gate: `docs/adr/0219-state-driven-style-resolution-v1.md`
  - Ecosystem override surface: `docs/adr/0220-ecosystem-style-override-surface-v1.md`
- Post-MVP next wave: `docs/workstreams/material3-next-wave-v2.md`

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
- [x] Expand Expressive coverage incrementally without inventing Fret-only component tokens:
  - Treat `DynamicVariant::Expressive` as the source of truth for **palette/scheme** changes
    (`md.sys.color.*`), independent of per-component `.expressive.*` tokens.
  - As of Material Web v30 sassvars, `.expressive.` component tokens are only present for `List`
    (shape + icon sizes). Do not add placeholder expressive tokens for other components yet.
  - When upstream adds expressive component tokens, implement them by:
    - importing via `material3_token_import` into `tokens/material_web_v30.rs`, and
    - plumbing `MaterialDesignVariant` through the relevant typed token modules (like `tokens/list.rs`),
      keeping component recipes unchanged.
  - Evidence: `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_sys_colors` sets `md.sys.fret.material.is-expressive` from `DynamicVariant`),
    `ecosystem/fret-ui-material3/src/foundation/context.rs` (`theme_default_design_variant`),
    `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (v30 `.expressive.` keys currently only for `md.comp.list.*`),
    `ecosystem/fret-ui-material3/src/tokens/list.rs` + `ecosystem/fret-ui-material3/src/list.rs` (design variant aware token selection).

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
- Interactivity pseudoclasses contract (hover/pressed as paint-only): `docs/adr/0166-interactivity-pseudoclasses-and-structural-stability.md`

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

- [x] Add a `LocalTextStyle`-like scoped default helper in Material foundation (text style + icon size).
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/context.rs` (`with_material_text_style`, `with_material_icon_size`),
    `ecosystem/fret-ui-material3/src/select.rs` (Select trigger uses `inherited_text_style` for display text).
- [x] Land `foundation::elevation` (shadow + tonal overlay) and migrate `Surface`-like containers.
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/elevation.rs`, `ecosystem/fret-ui-material3/src/foundation/surface.rs`,
    `ecosystem/fret-ui-material3/src/dialog.rs`, `ecosystem/fret-ui-material3/src/menu.rs`, `ecosystem/fret-ui-material3/src/tooltip.rs`,
    `ecosystem/fret-ui-material3/src/navigation_bar.rs`, `ecosystem/fret-ui-material3/src/navigation_drawer.rs`.
- [x] Decide the public surface for hoistable interaction sources (if any), and standardize “pressed origin” latching.
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/context.rs` (decision to defer hoistable sources),
    `ecosystem/fret-ui-material3/src/foundation/indication.rs` (ripple origin derived from last pointer down / keyboard fallback).
- [x] Audit which parts of minimum touch target policy should become a core `fret-ui` mechanism vs remain Material-only.
  - Decision: keep as Material foundation policy for now (tree-local + token-driven), not core.
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/interactive_size.rs`,
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`md.sys.layout.minimum-touch-target.size`),
    tests in `ecosystem/fret-ui-material3/src/lib.rs` (`material3_components_apply_minimum_touch_target_policy`).
- [x] Decide whether we need a core pixel-snapping policy hook for non-1.0 scale factors (radio/checkbox drift class).
  - Decision: add an explicit, opt-in snapping hook at the container paint boundary, and use it
    in Material 3 controls that are sensitive to fractional pixel drift.
  - Evidence: `crates/fret-ui/src/pixel_snap.rs` (snapping helpers),
    `crates/fret-ui/src/element.rs` (`ContainerProps.snap_to_device_pixels`),
    `crates/fret-ui/src/declarative/host_widget/paint.rs` (applies snapping when enabled),
    `ecosystem/fret-ui-material3/src/checkbox.rs` + `ecosystem/fret-ui-material3/src/radio.rs`.
- [x] Add MVP `Card` + `AssistChip` surfaces (token-driven + foundation indication) and cover them in headless suites.
  - Notes: Material Web v30 sassvars do not currently include chip spacing tokens (e.g. `leading-space`/`trailing-space`),
    so those remain component recipe constants for now (see `ecosystem/fret-ui-material3/src/tokens/chip.rs`).
  - Evidence: `ecosystem/fret-ui-material3/src/{card,chip}.rs`,
    `ecosystem/fret-ui-material3/src/tokens/{card,chip}.rs`,
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (injectors),
    `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs` (prefix set),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` + `goldens/material3-headless/v1/material3-controls.*.json`,
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_chip`, `preview_material3_card`).
- [x] Add MVP `FilterChip` / `InputChip` / `SuggestionChip` surfaces and cover them in headless suites + gallery.
  - Notes:
    - Material Web v30 filter chip tokens do not currently include distinct leading/trailing icon size tokens,
      so we temporarily use `md.comp.filter-chip.with-icon.icon.size` (deprecated upstream) instead of inventing new keys.
    - InputChip trailing icon supports a dedicated nested pressable via `InputChip::on_trailing_icon_activate`.
      (Includes expanded touch target + arrow-key focus handoff between primary/trailing actions.)
    - FilterChip trailing icon supports a dedicated nested pressable via `FilterChip::on_trailing_icon_activate`,
      following the same multi-action (primary + trailing) model as Material Web.
  - Evidence: `ecosystem/fret-ui-material3/src/{filter_chip,input_chip,suggestion_chip}.rs`,
    `ecosystem/fret-ui-material3/src/tokens/{filter_chip,input_chip,suggestion_chip}.rs`,
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (injectors),
    `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs` (prefix + emit),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` + `goldens/material3-headless/v1/material3-controls.*.json`,
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_chip`).
- [x] Add MVP `ChipSet` (roving focus container) and make roving focus work when focus is inside a multi-action chip.
  - Notes:
    - Material Web `md-chip-set` uses `:focus-within` to treat a focused trailing action as the active chip.
      Fret's `RovingFlex` now mirrors that behavior by resolving the active item index from descendant focus.
  - Evidence: `ecosystem/fret-ui-material3/src/chip_set.rs`,
    `crates/fret-ui/src/declarative/host_widget/event/roving_flex.rs`,
    tests in `crates/fret-ui/src/declarative/tests/interactions.rs` (`roving_flex_treats_descendant_focus_as_active_item`),
    tests in `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`chip_set_roving_treats_trailing_action_focus_as_active_chip`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_chip`).

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
  - Notes: This audit treats Material Web v30 sassvars as the source of truth for key existence (unknown keys are flagged),
    helping us avoid inventing Fret-only `md.*` keys during refactors.
  - Repro: `cargo run -p fret-ui-material3 --bin material3_token_audit -- --check --debug` should exit 0 and report
    `missing injected keys: 0` and no `Unknown keys` section.
- [x] Introduce strict token resolver + content defaults (foundation).
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`,
    `ecosystem/fret-ui-material3/src/foundation/content.rs`.
- [x] Add a Material 3 "state matrix" gallery page for manual regression.
  - Evidence: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_STATE_MATRIX`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_state_matrix`).
- [x] Align core Material 3 components with the ecosystem `*Style` override surface (ADR 0220).
  - Evidence: `docs/workstreams/material3-style-api-alignment-v1.md`,
    `ecosystem/fret-ui-material3/src/{button,checkbox,dialog,dropdown_menu,icon_button,menu,radio,switch,tabs,text_field}.rs`,
    `apps/fret-ui-gallery/src/ui.rs` (default vs override blocks for core controls and overlays).
- [x] Finish migrating remaining components to the Material foundation surface, and delete duplicated per-component helpers.
  - [x] Migrate `List` to the foundation indication path and remove non-Material fallbacks.
    - Evidence: `ecosystem/fret-ui-material3/src/list.rs` (uses `material_ink_layer_for_pressable`, `material_pressable_indication_config`).
  - [x] Migrate `Select` (trigger + option rows) to the foundation indication path and remove non-Material fallbacks.
    - Evidence: `ecosystem/fret-ui-material3/src/select.rs` (uses `material_ink_layer_for_pressable`, `material_pressable_indication_config`).
  - [x] Migrate navigation items (`NavigationBar`/`NavigationDrawer`/`NavigationRail`) to the foundation indication path.
    - Evidence: `ecosystem/fret-ui-material3/src/navigation_bar.rs`,
      `ecosystem/fret-ui-material3/src/navigation_drawer.rs`,
      `ecosystem/fret-ui-material3/src/navigation_rail.rs`.
  - [x] Keep `Slider`/`RangeSlider` on a bespoke paint path (Canvas-driven), but reuse the shared state-layer animation config from the foundation to reduce drift.
    - Evidence: `ecosystem/fret-ui-material3/src/slider.rs` (custom track/tick/handle painting; state-layer opacity driven by `StateLayerAnimator` configured via `material_pressable_indication_config`).
  - [x] Reduce per-component helper duplication (e.g. `interaction_state` precedence, small color math helpers) if the cost/benefit is favorable.
    - Evidence: `ecosystem/fret-ui-material3/src/foundation/interaction.rs` (centralized pressable interaction precedence),
      `ecosystem/fret-ui-material3/src/{button,icon_button,checkbox,list,navigation_bar,navigation_drawer,navigation_rail,radio,switch}.rs` (uses it).

## Audit Anchors (Fret)

- Theme system (v1): `crates/fret-ui/src/theme.rs`, `crates/fret-ui/src/theme_registry.rs`
- Focus-visible + focus rings: `crates/fret-ui/src/focus_visible.rs`, `crates/fret-ui/src/paint.rs`
- Shadows/elevation primitive: `crates/fret-ui/src/paint.rs`
- Overlay mechanisms: `crates/fret-ui/src/tree/*` + `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Interactivity pseudoclasses contract: `docs/adr/0166-interactivity-pseudoclasses-and-structural-stability.md`

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
- [x] Implement import pipeline from `repo-ref/material-web/tokens/versions/v30_0` into Fret theme configs.
  - [x] Auto-discover `repo-ref/material-web` in git worktrees (fallback to `MATERIAL_WEB_DIR`).
    - Evidence: `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs` (`default_material_web_dir`),
      `ecosystem/fret-ui-material3/src/bin/material3_token_audit.rs` (`resolve_material_web_dir`).
  - [x] Add a `--check` mode for `material3_token_import` to make the output reproducible (CI-friendly).
    - Evidence: `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs` (`--check`, `generate_output`, `rustfmt_file`).
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
  - [x] Import the subset of `md.comp.*` tokens used by MVP components (drive by `material3_token_audit`).
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
    - [x] Expand scalar import coverage for other MVP components.
      - Evidence: `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs` (prefix allowlist includes all MVP components),
        `tools/check_material3_tokens.py` (local reproducible check entrypoint).
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
- [x] Extend MotionScheme mapping for Expressive tokens (fallback to Compose baseline until the token
  source of truth provides expressive system motion tokens).
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/motion_scheme.rs`,
    `ecosystem/fret-ui-material3/src/foundation/context.rs` (`theme_default_motion_scheme`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`md.sys.fret.material.motion.spring.*`).
  - Reference: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ExpressiveMotionTokens.kt`.
- [x] Decide how to represent spring configs long-term (ecosystem-only vs core mechanism).
  - Decision (v1): keep springs as pairs of number tokens (`*.damping` + `*.stiffness`) and
    construct `SpringSpec` in the ecosystem (Material foundation / motion helpers). Do not add a
    new core `ThemeTokenKind` yet.
  - Rationale: current spring usage is still Material-specific (MotionScheme mapping and a small
    set of component corner/overlay springs). Elevating to a core token kind is hard to roll back,
    and we can revisit once another ecosystem (or core animation infra) needs first-class springs.
  - Evidence: `ecosystem/fret-ui-material3/src/motion.rs` (`SpringSpec`),
    `ecosystem/fret-ui-material3/src/foundation/motion_scheme.rs` (`sys_spring_in_scope`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (Expressive spring fallback tokens under
    `md.sys.fret.material.motion.spring.*`).
- [x] Introduce typed token modules per component to reduce raw string key usage and centralize
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
    `ecosystem/fret-ui-material3/src/snackbar.rs` (toast-layer motion tokens).
- [x] Overlay open/close motion uses MotionScheme spring specs (menu/tooltip/select).
  - Evidence: `ecosystem/fret-ui-material3/src/foundation/overlay_motion.rs` (`drive_overlay_open_close_motion`),
    `ecosystem/fret-ui-material3/src/dropdown_menu.rs`,
    `ecosystem/fret-ui-material3/src/select.rs`,
    `ecosystem/fret-ui-material3/src/tooltip.rs`.
  - Reference: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Menu.kt`
    (`ExpandedScaleTarget = 1f`, `ClosedScaleTarget = 0.8f`) and `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tooltip.kt`.
- [x] Overlay outcomes (menu, dialog, tooltip):
  - [x] Escape dismissal (menu dropdown)
    - Evidence: `ecosystem/fret-ui-material3/src/dropdown_menu.rs` (OverlayRequest::dismissible_menu),
      `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`dropdown_menu_dismisses_and_restores_focus_across_schemes`).
  - [x] outside press dismissal (menu dropdown)
    - Evidence: `ecosystem/fret-ui-material3/src/dropdown_menu.rs` (OverlayRequest::dismissible_menu),
      `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`dropdown_menu_dismisses_and_restores_focus_across_schemes`).
  - [x] outside press closes without activating the underlay (menu-like popovers)
    - Evidence: `ecosystem/fret-ui-kit/src/overlay_controller.rs` (`OverlayRequest::dismissible_menu`),
      `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`dropdown_menu_dismisses_and_restores_focus_across_schemes`,
      `select_keyboard_open_sets_initial_focus_and_outside_dismiss_restores_focus_across_schemes`).
  - [x] focus trap/restore (modal) (currently validated via modal navigation drawer)
    - Evidence: `ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs` (focus trap),
      `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (focus restore),
      `ecosystem/fret-ui-kit/src/primitives/focus_scope.rs` (`FocusScopeProps { trap_focus: true }`).
  - [x] scrim press dismisses without activating the underlay (modal)
    - Evidence: `ecosystem/fret-ui-material3/src/dialog.rs` (scrim pressable + dismiss handler),
      `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`dialog_scrim_dismisses_without_activating_underlay`).
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
- [x] Shape mapping (corner tokens, per-state expressive shape where applicable).
  - [x] Corner set tokens (`md.sys.shape.corner.*.(top|start|end)`) and component shapes that depend on them.
  - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_sys_shape`),
    `ecosystem/fret-ui-material3/src/tokens/{menu,select,switch}.rs` (typed shape access),
    `ecosystem/fret-ui-material3/src/{checkbox,menu,navigation_bar,radio,select,switch}.rs`
    (components consume token-driven `Corners`, including state-layer/focus indicator clipping).
- [x] Typography mapping (typescale roles).
  - Evidence: `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs` (`inject_sys_typescale`),
    `ecosystem/fret-ui-material3/src/{button,dialog,list,menu,navigation_bar,navigation_drawer,navigation_rail,select,snackbar,tabs,tooltip}.rs`
    (components use `md.sys.typescale.*` or stable per-component aliases such as `md.comp.button.label-text`).

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
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_navigation_bar`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_navigation_suite_goldens_v1`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`navigation_bar_roving_skips_disabled_and_updates_model`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`navigation_bar_roving_wraps_and_skips_disabled_on_reverse`),
    `goldens/material3-headless/v1/material3-navigation.scale1_0.dark.tonal_spot.json` (representative; full matrix is generated).
- [x] Navigation rail (MVP: roving focus + state layer + bounded ripple + active indicator)
  - Evidence: `ecosystem/fret-ui-material3/src/navigation_rail.rs` (`NavigationRail`, `NavigationRailItem`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_navigation_rail_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_navigation_rail`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_NAVIGATION_RAIL`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_navigation_suite_goldens_v1`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`navigation_rail_roving_skips_disabled_and_updates_model`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`navigation_rail_roving_wraps_and_skips_disabled_on_reverse`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`navigation_rail_roving_does_not_wrap_when_loop_navigation_false`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`navigation_rail_roving_single_enabled_item_does_not_move_under_no_loop`),
    `goldens/material3-headless/v1/material3-navigation.scale1_0.dark.tonal_spot.json` (representative; full matrix is generated).
- [x] Navigation drawer (MVP: roving focus + state layer + bounded ripple + selected pill background)
  - Evidence: `ecosystem/fret-ui-material3/src/navigation_drawer.rs` (`NavigationDrawer`, `NavigationDrawerItem`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_navigation_drawer_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_navigation_drawer`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_NAVIGATION_DRAWER`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_navigation_suite_goldens_v1`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`navigation_drawer_roving_skips_disabled_and_updates_model`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`navigation_drawer_roving_wraps_and_skips_disabled_on_reverse`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`navigation_drawer_roving_does_not_wrap_when_loop_navigation_false`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`navigation_drawer_roving_single_enabled_item_does_not_move_under_no_loop`),
    `goldens/material3-headless/v1/material3-navigation.scale1_0.dark.tonal_spot.json` (representative; full matrix is generated).
- [x] Modal navigation drawer (MVP: modal overlay + scrim + slide-in motion + focus trap/restore)
  - Evidence: `ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs` (`ModalNavigationDrawer`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_modal_navigation_drawer`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_MODAL_NAVIGATION_DRAWER`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_navigation_suite_goldens_v1`),
    `goldens/material3-headless/v1/material3-navigation.scale1_0.dark.tonal_spot.json` (representative; full matrix is generated).
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
- [x] Divider (MVP: token-driven thickness + color)
  - Evidence: `ecosystem/fret-ui-material3/src/divider.rs` (`Divider`),
    `ecosystem/fret-ui-material3/src/tokens/divider.rs` (`md.comp.divider.*` token mapping),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_divider_suite_goldens_v1`).
- [x] Progress indicator (MVP: determinate + indeterminate linear/circular incl four-color, token-driven colors/sizes/thickness)
  - Evidence: `ecosystem/fret-ui-material3/src/progress_indicator.rs` (`LinearProgressIndicator`, `CircularProgressIndicator`),
    `ecosystem/fret-ui-material3/src/tokens/progress_indicator.rs` (`md.comp.progress-indicator.*` token mapping),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_progress_indicator_suite_goldens_v1`).
- [x] Slider (MVP: token-driven track/handle + state-layer (hover/pressed/focus-visible) + value indicator + tick marks (+ tick count override) + pointer drag + keyboard step)
  - Evidence: `ecosystem/fret-ui-material3/src/slider.rs` (`Slider`),
    `ecosystem/fret-ui-material3/src/tokens/slider.rs` (`md.comp.slider.*` token mapping),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_slider_suite_goldens_v1`, cases: `idle`/`hover`/`pressed`/`dragging`/`focus_visible`/`keyboard_page`/`rtl_idle`/`rtl_keyboard_arrows`/`with_tick_marks`/`tick_count`/`range_dragging`/`range_focus_thumb_switch`/`range_keyboard_page`/`rtl_range_keyboard_arrows`).
- [x] Range slider (MVP: two-thumb range selection, token-driven styling, pointer drag + keyboard step)
  - Evidence: `ecosystem/fret-ui-material3/src/slider.rs` (`RangeSlider`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_slider_suite_goldens_v1`, cases: `range_dragging`/`range_focus_thumb_switch`; per-thumb focus semantics via test ids `range-slider-30-70.start` + `range-slider-30-70.end`).
- [x] Tooltip (MVP: plain tooltip, delay group + hover intent + safe-hover corridor, token-driven styling)
  - Evidence: `ecosystem/fret-ui-material3/src/tooltip.rs` (`PlainTooltip`, `TooltipProvider`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_plain_tooltip_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_tooltip`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_TOOLTIP`).
- [x] Snackbar (MVP: toast-layer skin using `md.comp.snackbar.*` tokens, action + dismiss icon)
  - Evidence: `ecosystem/fret-ui-material3/src/snackbar.rs` (`SnackbarHost`, `SnackbarController`),
    `ecosystem/fret-ui-material3/src/tokens/v30.rs` (`inject_comp_snackbar_*`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_snackbar`),
    `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MATERIAL3_SNACKBAR`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_snackbar_suite_goldens_v1`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`snackbar_action_emits_command_and_dismisses`),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`snackbar_dismiss_button_dismisses_without_emitting_command`),
    `goldens/material3-headless/v1/material3-snackbar.scale1_0.dark.tonal_spot.json` (representative; full matrix is generated).

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
  - Evidence: `ecosystem/fret-ui-material3/src/text_field.rs` (floating label progress driven by `MotionSchemeKey::FastSpatial`;
    placeholder opacity uses `FastEffects`/`SlowEffects`; outline/indicator thickness uses `FastSpatial`).
  - Reference: `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/internal/TextFieldImpl.kt`
    (`labelTransitionSpec = MotionSchemeKeyTokens.FastSpatial`).
- [x] Verify modal overlay focus trap/restore across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`dialog_focus_is_contained_and_restored_across_schemes`,
    `modal_navigation_drawer_focus_is_contained_and_restored_across_schemes`).
- [x] Verify tooltip hover open/close and dropdown menu dismissal across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`tooltip_opens_and_closes_on_hover_across_schemes`,
    `dropdown_menu_dismisses_and_restores_focus_across_schemes`).
- [x] Verify Select dismissal and focus restore across light/dark + TonalSpot/Expressive schemes.
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`select_dismisses_and_restores_focus_across_schemes`,
    `select_keyboard_open_sets_initial_focus_and_outside_dismiss_restores_focus_across_schemes` (ArrowUp/ArrowDown/Enter/Space open keys),
    `select_roving_scrolls_focused_option_into_view`, `select_open_scrolls_selected_option_into_view`,
    `select_listbox_typeahead_moves_focus_skipping_disabled_options`,
    `select_menu_matches_anchor_width_and_clamps_height_to_available_space`).
  - Evidence: `ecosystem/fret-ui-material3/src/select.rs` (Select trigger: `.leading_icon(...)` / `.label(...)` / `.supporting_text(...)` / `.error(...)` + animated trailing icon; `SelectStyle` override surface includes label/supporting/leading icon slots;
    supporting text inset aligns with the input text start when a leading icon is present; supporting text color follows hover/focus token branches; open state is treated as focused (Material Web parity); listbox padding; `SelectItem::{leading_icon,trailing_icon}`),
    `ecosystem/fret-ui-material3/src/tokens/select.rs` (`text-field.error.*` mapping + `menu_list_item_{leading,trailing}_icon_*` tokens),
    `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (headless suites include select trigger states + `material3_headless_controls_suite_goldens_v1` includes `idle_select_supporting_text_insets` + `material3_headless_overlays_suite_goldens_v1` includes `select_open`, `select_open_trigger` and `select_open_hover_selected`),
  - Evidence (reference): `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Menu.kt`
    (`MenuVerticalMargin = 48.dp`),
    `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ExposedDropdownMenu.kt`
    (`exposedDropdownSize(matchAnchorWidth)`, `calculateMaxHeight`).
  - Notes: Select menu item typography defaults to `md.sys.typescale.label-large` (Material Web v30 uses `label-large` for the menu list-item label).
- [x] Add golden-style visual snapshots per component state (light/dark, density variants).
  - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_controls_suite_goldens_v1`, `material3_headless_overlays_suite_goldens_v1`, `material3_headless_text_field_suite_goldens_v1`, `material3_headless_divider_suite_goldens_v1`, `material3_headless_progress_indicator_suite_goldens_v1`, `material3_headless_slider_suite_goldens_v1`),
    `goldens/material3-headless/v1/material3-*.json` (controls suite, overlay suite, text-field suite, divider suite, progress indicator suite, slider suite; includes `scale1_0`/`scale1_25`/`scale2_0` variants and overlay cases such as `both_open` + `select_open`).

## Proposed ADRs (drafts)

- `docs/adr/0228-theme-value-kinds-and-themeconfig-v2.md`
- `docs/adr/0226-material3-state-layer-and-ripple-primitives.md`
