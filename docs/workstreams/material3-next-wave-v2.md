## Material 3 / Expressive — Next Wave (v2)

Status: Work-in-progress

This workstream starts **after** the MVP set in `docs/workstreams/material3-todo.md` is landed.
It tracks the next set of components and any small mechanism gaps that become unavoidable.

Non-goals:

- Perfect DOM/Lit parity with Material Web.
- Premature “component library completeness”. We continue to use demos + regression harnesses to
  drive requirements.

### Guiding principles (carry-over)

- Prefer token-driven defaults (`md.comp.*` → `md.sys.*`) and typed token modules per component.
- Keep `crates/fret-ui` mechanism-only; only propose core changes when unavoidable for correct
  outcomes.
- Favor suite-style headless goldens + small scripted interaction tests over pixel snapshots.

## P0 (recommended)

- [x] FAB (Floating Action Button) MVP surface in `ecosystem/fret-ui-material3`.
  - Subtasks:
    - Import `md.comp.fab.*` / `md.comp.extended-fab.*` scalars/colors via `material3_token_import` (no Fret-only keys).
    - Implement variants (surface/primary/secondary/tertiary) and sizes (small/medium/large; extended ignores sizes).
    - Interaction: bounded ripple + state layer + focus-visible ring.
    - Headless suites: add `material3-fab.*.json` into `goldens/material3-headless/v1/`.
  - Evidence:
    - `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_fab_suite_goldens_v1`).
    - `goldens/material3-headless/v1/material3-fab.scale1_0.dark.tonal_spot.json` (representative; full matrix is generated).
    - `apps/fret-ui-gallery/src/ui.rs` (Material 3 gallery + state matrix include FAB examples).
  - References: `repo-ref/material-web/components/fab`, `repo-ref/compose-multiplatform-core/compose/material3/.../Fab.kt`.

- [x] Segmented buttons MVP surface (single + multi-select groups).
  - Subtasks:
    - Import `md.comp.outlined-segmented-button.*` scalars/colors via `material3_token_import`.
    - Implement single-select and multi-select groups with a value model + item list API.
    - Roving focus + Home/End + skip disabled, consistent with other groups.
    - Selection semantics: expose selected state on items and treat the set as a group.
    - Interaction: bounded ripple + state layer + focus-visible ring.
    - Headless suites: add `material3-segmented-button.*.json` into `goldens/material3-headless/v1/`.
    - Gallery page: `apps/fret-ui-gallery` page `material3_segmented_button`.
  - Evidence:
    - `ecosystem/fret-ui-material3/src/segmented_button.rs`
    - `ecosystem/fret-ui-material3/src/tokens/segmented_button.rs`
    - `ecosystem/fret-ui-material3/src/tokens/v30.rs` (v30 token injection wiring)
    - `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_segmented_button_suite_goldens_v1`).
    - `goldens/material3-headless/v1/material3-segmented-button.scale1_0.dark.tonal_spot.json` (representative; full matrix is generated).
    - `apps/fret-ui-gallery/src/spec.rs` + `apps/fret-ui-gallery/src/ui.rs`.
  - References: `repo-ref/material-web/components/segmentedbutton`, Compose segmented buttons.

## P1

- [x] Top app bar / toolbar primitives (small/center/medium/large).
  - Subtasks:
    - Import `md.comp.top-app-bar.{small,small.centered,medium,large}.*` scalars/colors via `material3_token_import`.
    - Implement top app bar primitives in `ecosystem/fret-ui-material3` with a minimal `scrolled: bool` surface
      (switches to `on-scroll` container tokens).
    - Headless suites: add `material3-top-app-bar.*.json` into `goldens/material3-headless/v1/`.
    - Gallery page: `apps/fret-ui-gallery` page `material3_top_app_bar`.
  - Notes:
    - Material Web uses `role="toolbar"` for certain containers; Fret models top app bar semantics as `Toolbar`.
      If we need explicit accessibility contracts, revisit core `SemanticsRole` coverage as a mechanism follow-up.
  - Evidence:
    - `ecosystem/fret-ui-material3/src/top_app_bar.rs`
    - `ecosystem/fret-ui-material3/src/tokens/top_app_bar.rs`
    - `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs` (token import allowlist)
    - `ecosystem/fret-ui-material3/src/tokens/v30.rs` (v30 token injection wiring)
    - `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_top_app_bar_suite_goldens_v1`)
      - Includes a `small_centered.wide_actions` case to lock in centered title placement under
        asymmetric action widths.
    - `goldens/material3-headless/v1/material3-top-app-bar.scale1_0.dark.tonal_spot.json` (representative; full matrix is generated)
    - `apps/fret-ui-gallery/src/spec.rs` + `apps/fret-ui-gallery/src/ui.rs`

- [x] Top app bar scroll behavior (pinned / enterAlways / exitUntilCollapsed) surface.
  - Goal: replace the v1 boolean `.scrolled(bool)` with a reusable policy object (Compose-style
    `TopAppBarScrollBehavior`) so medium/large app bars can do outcome-correct collapse + elevation
    without per-app re-implementations.
  - Notes:
    - This remains policy-only: the behavior reads `ScrollHandle` deltas/offsets (no nested scroll),
      and does not model fling/snap settling yet.
    - Gallery demos must persist the behavior object across frames; recreating it every frame will
      reset internal state and can make `enterAlways` look incorrect.
    - Keep the mechanism boundary: propose core `fret-ui` additions only if the behavior cannot be
      expressed as policy in `fret-ui-material3`.
  - Alignment notes (Compose / Material Web):
    - Compose models scroll behaviors as a nested-scroll policy + state machine (`TopAppBarState`)
      with optional fling settle + snap (see `TopAppBarDefaults.*ScrollBehavior` and `settleAppBar*`
      in `repo-ref/compose-multiplatform-core/.../AppBar.kt`).
    - Material Web (v30) provides token coverage (`md.comp.top-app-bar.*`) but does not ship a
      public scroll-behavior API in the component set; the catalog uses an internal `top-app-bar`
      element for site layout.
  - Known gaps (tracked for future waves; not required for MVP):
    - [ ] Decide if/when we need nested scroll consumption (pre/post scroll) vs. `ScrollHandle`
          observation only.
    - [x] Add a settle/snap story (fling + snap animation) for `enterAlways` / `exitUntilCollapsed`.
          If unavoidable, propose the minimal mechanism surface in `crates/fret-ui` (e.g. scroll
          velocity / animation driver hook).
          - Decision (v1): implement policy-only idle settle/snap via
            `TopAppBarScrollBehavior::settle_on_idle()`:
            - `enterAlways`: settles height offset via a spring after idle (no fling velocity).
            - `exitUntilCollapsed`: snaps scroll offset within the collapse range to either `0` or
              `collapseRange` after idle (no nested scroll consumption; content moves).
            - Follow-up trigger: if we need "collapse without scrolling content", we must introduce
              nested scroll consumption in `crates/fret-ui`.
    - [x] Expose a `can_scroll` gate (Compose-style) to avoid collapsing when the content cannot
          scroll.
    - [x] Validate behavior under programmatic scroll jumps (ensure-visible / scroll-to) and decide
          whether the policy should reset internal deltas on discontinuities.
          - Decision (v1): treat programmatic jumps as real offset deltas (no special reset); the
            behavior should be stable and recover on subsequent scrolls.
  - Evidence:
    - `ecosystem/fret-ui-material3/src/top_app_bar.rs` (`TopAppBarScrollBehavior::{pinned,enter_always,exit_until_collapsed}`)
      - Tests: `enter_always_respects_can_scroll_gate_when_metrics_are_known`,
        `enter_always_handles_programmatic_scroll_jumps_and_recovers`
    - `apps/fret-ui-gallery/src/ui.rs` (`preview_material3_top_app_bar` scroll demos)
    - `tools/diag-scripts/ui-gallery-material3-top-app-bar-scroll-screenshots.json` (fretboard screenshot harness)
    - `target/fret-diag-top-app-bar-settle` (PASS run_id=1770084041283; includes settle-on-idle screenshots)
    - `target/fret-diag-top-app-bar-scroll-20260203` (PASS run_id=1770122581069)
  - References:
    - Compose `TopAppBarDefaults.*ScrollBehavior` + `TopAppBarState` in
      `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AppBar.kt`.

- [x] Select parity follow-ups (Material Web / Compose exposed dropdown menus).
  - Goal: keep the current MVP `Select` surface stable while tracking a small set of
    "hard-to-change later" behavior and accessibility contracts (avoid large rewrites).
  - Current coverage (already landed in MVP workstream):
    - Roving focus + typeahead + scroll-into-view, anchor width matching, and max height clamping.
    - Open state treated as focused for token branches (Material Web parity note).
    - Menu knobs:
      - `menu_align` (Material Web `menuAlign`: start/end).
      - `match_anchor_width` (Compose `matchAnchorWidth` / Material Web `clampMenuWidth`-style clamping).
      - When unclamped, menu min-width tracks the select width (Material Web behavior) and applies a
        configurable 210px ergonomic floor via `menu_width_floor` (default on).
      - `typeahead_delay_ms` (Material Web `typeaheadDelay`, default 200ms).
      - Decision (v1): keep `menu_width_floor=210px` enabled by default.
        - Rationale:
          - Matches Material Web's ergonomics intent (select host `min-width: 210px`) while still
            letting the menu grow for long labels (unclamped).
          - Avoids "too narrow" menus when the anchor is compact (common in toolbars / dense UIs).
          - Can be disabled explicitly via `menu_width_floor(Px(0.0))` if a product wants strict
            content-driven sizing.
    - Per-option text overrides: `display_text` (trigger display) + `typeahead_text` (prefix matching).
    - Overlay escape hatch: Select-in-Dialog probe to validate listbox overlay is not clipped by
      modal containers.
    - Decision (v1): listbox supporting/trailing-supporting text uses list token defaults
      (`md.comp.list.list-item.*`, sys `body-small`) rather than introducing a new Select-specific
      token surface.
  - Decision (v1): focus moves into the listbox (not trigger + `active_descendant`).
    - We keep the current behavior: when opening the menu, we set initial focus to a listbox
      option (roving focus) and drive keyboard navigation there.
    - Rationale:
      - This matches existing Fret overlay/focus mechanics and avoids a mechanism expansion.
      - Compose-style "focus stays on trigger and listbox navigation is exposed via
        `active_descendant`" requires keeping trigger focus while driving a "virtual focus" across
        listbox options. Fret already supports `active_descendant`, and `fret-ui-kit` can map
        option elements to runtime `NodeId`, but adopting this policy adds cross-frame bookkeeping
        and should be driven by real AT validation rather than premature parity work.
      - We can still revisit this once we have a stable `active_descendant_element` surface in
        `fret-ui` (if assistive tech parity demands it).
  - Decision (v1): keep "listbox-focused" semantics unless an accessibility trigger requires
    Compose-style "trigger retains focus + `active_descendant`" policy.
    - Rationale:
      - Matches Fret's current overlay/focus mechanics with less duplicated state.
      - We already expose portable combobox/listbox relationships (`controls` + `labelled_by`).
      - Compose-style focus retention likely requires a more explicit "value" semantics surface on
        the trigger (or an auxiliary semantics node) to match AT expectations.
    - Follow-up trigger (v2):
      - If real screen reader/AT testing shows degraded announcements or compliance gaps under
        listbox focus.
      - If product requirements demand Compose-like behavior parity.
  - References:
    - Material Web select (`repo-ref/material-web/select/internal/select.ts`) properties:
      `menuPositioning`, `menuAlign`, `clampMenuWidth`, `typeaheadDelay`, `required`, `errorText`,
      plus per-option `displayText` / `typeaheadText` via `SelectOptionController`.
    - Compose exposed dropdown menus:
      `repo-ref/compose-multiplatform-core/.../ExposedDropdownMenu.kt`
      (`ExposedDropdownMenuBox`, `exposedDropdownSize(matchAnchorWidth)`,
      `calculateMaxHeight`, focus / semantics behaviors).
  - Evidence:
    - `apps/fret-ui-gallery/src/ui.rs` (Material 3 Dialog page includes a Select inside the dialog)
    - `tools/diag-scripts/ui-gallery-material3-select-dialog-overlay-screenshots.json` (fretboard screenshots)
      - Expected: both `ui-gallery-material3-dialog-select-listbox` and
        `ui-gallery-material3-dialog-select-bottom-listbox` open without being clipped by the dialog
        panel; the bottom probe should clamp/collide instead of overflowing past the window edge.
      - Verified: `target/fret-diag-select-dialog4c` (PASS run_id=1770039668512)
    - `tools/diag-scripts/ui-gallery-material3-select-dialog-bottom-collision.json` (fretboard scripted bounds check)
      - Expected: `ui-gallery-material3-dialog-select-bottom-listbox` stays within the OS window bounds.
      - Verified: `target/fret-diag-select-dialog-bottom-collision1` (PASS run_id=1770124737905)
    - `tools/diag-scripts/ui-gallery-material3-select-dialog-a11y-bundle.json` (fretboard scripted a11y check)
      - Expected: when opening Select from inside a dialog, focus moves into the listbox option nodes
        (roving) and restores to the trigger on Escape.
      - Verified: `target/fret-diag-select-dialog-a11y8` (PASS run_id=1770122180225)
    - `tools/diag-scripts/ui-gallery-material3-select-dialog-a11y-parity-bundle.json` (fretboard bundle evidence)
      - Note: run with `FRET_DIAG_SEMANTICS=1` (and optionally `FRET_DIAG_SCRIPT_AUTO_DUMP=0` to avoid
        per-step auto dumps).
      - Verified: `target/fret-diag-select-dialog-a11y-parity2` (PASS run_id=1770122235926)
    - `tools/diag-scripts/ui-gallery-material3-select-overlay-parity-screenshots.json` (fretboard screenshots)
      - Requires: `FRET_DIAG_SCREENSHOTS=1`
      - Coverage: default / override, unclamped width floor on/off, rich options, transformed menu positioning.
      - Verified: `target/fret-diag-select-overlay-parity8` (PASS run_id=1770114240224)
    - `tools/diag-scripts/ui-gallery-material3-select-menu-width-floor-screenshots.json` (fretboard screenshots)
      - Verified: `target/fret-diag-select-floor2` (PASS run_id=1770042125679)
    - `tools/diag-scripts/ui-gallery-material3-select-typeahead-delay.json` (fretboard screenshots)
      - Expected: with delay=1000ms, `d` then quick `e` keeps focus on Delta until the buffer expires;
        after waiting long enough, `e` moves focus to Echo. With delay=200ms, `e` moves to Echo after
        a short wait.
      - Verified: `target/fret-diag-select-typeahead-delay1` (PASS run_id=1770049596707)
    - `tools/diag-scripts/ui-gallery-material3-select-rich-options-screenshots.json` (fretboard screenshots)
      - Expected: Select listbox rows render per-option supporting and trailing supporting text.
      - Verified: `target/fret-diag-select-rich-options2` (PASS run_id=1770078178017)
  - Follow-up checklist (ordered by risk / "hard-to-change later"):
    - [x] A11y parity (v1): keep listbox-focused roving navigation (do not switch to
          trigger-focused `active_descendant` yet).
          - Rationale:
            - Matches current Fret overlay/focus mechanics with minimal duplicated state.
            - Avoids committing to a higher-level "virtual focus" contract without real AT
              validation.
          - Evidence (manual inspection): run
            `tools/diag-scripts/ui-gallery-material3-select-a11y-parity-bundle.json` and inspect the
            captured semantics tree snapshots in `tools/fret-bundle-viewer`:
            - When open (v1): focus moves into the listbox option nodes (roving).
            - The trigger/listbox still expose `controls_element` / `labelled_by_element`
              relationships while open.
            - After dismiss (Escape): focus restores to the trigger.
            - Verified: `target/fret-diag-select-a11y-parity5` (PASS run_id=1770104703028).
          - AT acceptance checklist (manual, required to revisit v1):
            - On open: screen reader announces the control as expanded + exposes the list content.
            - Arrow navigation: focused option announcement updates correctly (and skips disabled).
            - Selection: chosen value is announced and reflected on the trigger after close.
            - Dismiss: Escape closes and focus returns to the trigger reliably.
          - Follow-up trigger (v2): only prioritize a behavior switch if real AT testing (NVDA /
            JAWS / VoiceOver / Narrator) reveals announcement/compliance gaps, or product
            requirements demand Compose-style parity.
    - [x] A11y wiring: expose combobox↔listbox relationships while open (portable approximations of
          `aria-controls` and `aria-labelledby`) via `controls_element` + `labelled_by_element`.
          - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
            (`select_exposes_combobox_controls_and_listbox_labelled_by_relations`).
    - [x] Menu sizing rules: preserve anchor width matching + clamp to available space, and support
          "unclamped but ergonomic" menu width with `menu_width_floor`.
          - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
            (`select_menu_matches_anchor_width_and_clamps_height_to_available_space`,
            `select_menu_width_floor_only_applies_when_unclamped`) + `target/fret-diag-select-floor2`
            (PASS run_id=1770042125679).
    - [x] Typeahead delay: support Material Web `typeaheadDelay` with a 200ms default.
          - Evidence: `target/fret-diag-select-typeahead-delay1` (PASS run_id=1770049596707).
    - [x] Option richness: per-option supporting and trailing supporting text in listbox rows.
          - Evidence: `target/fret-diag-select-rich-options2` (PASS run_id=1770078178017).
    - [x] Menu positioning knobs: do not expose Material Web's `menuPositioning` as a Select API.
          - Decision (v1): out-of-scope as an explicit Select API knob.
            - Fret overlays are already rendered in a top-level overlay layer, so "absolute vs
              fixed" positioning is not an author-visible concern in most cases.
            - Anchored overlays are render-transform aware via
              `fret-ui-kit/src/overlay.rs` (`anchor_bounds_for_element` prefers visual bounds).
          - Evidence:
            - `target/fret-diag-select-dialog4c` (PASS run_id=1770039668512).
            - `target/fret-diag-select-menu-positioning-transform7` (PASS run_id=1770095089495).
          - Follow-up trigger (v2): only revisit if we find a real nested clip/transform context
            where anchored overlays cannot obtain stable bounds or are clipped.
    - [x] Validation hooks: keep `required` / errorText override / form association out-of-scope.
          - Decision (v1): out-of-scope for `fret-ui-material3`.
            - We do not implement HTML form association / `reportValidity()` semantics.
            - Apps should render supporting/error text from their own form state and drive
              `Select::error(true)` explicitly.
          - Evidence: `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
            (`material3-select-trigger-error`).

- [ ] Autocomplete (outlined + filled) MVP surface.
  - Goal: provide an editable trigger with a listbox overlay using Material Web autocomplete tokens,
    aligned with Compose exposed dropdown menus at the outcome level.
  - Subtasks:
    - [x] Import `md.comp.{outlined,filled}-autocomplete.*` scalars/colors via `material3_token_import`.
      - Evidence:
        - `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs`
        - `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`
        - `ecosystem/fret-ui-material3/src/tokens/v30.rs`
    - [x] Support `aria-controls` on `TextInput` (declarative element IDs), so combobox-style
          relationships do not require extra wrapper semantics nodes.
      - Evidence:
        - `crates/fret-ui/src/element.rs`
        - `crates/fret-ui/src/declarative/host_widget/semantics.rs`
    - [ ] Implement `Autocomplete` component in `ecosystem/fret-ui-material3` (visual + interaction).
      - Requirements (v1):
        - Outlined + filled variants (token-driven defaults only; no non-Material fallbacks).
        - Overlay sizing parity with `Select` (match-anchor-width + clamp height, collision padding).
        - A11y: `ComboBox` trigger + `ListBox` overlay, with `controls_element` + `labelled_by_element`.
        - Keyboard: ArrowDown/ArrowUp open, Escape dismiss + focus restore, roving in listbox.
        - Headless suites + gallery page + at least one dialog-embedded probe (avoid clipping regressions).
  - References:
    - Material Web tokens:
      `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-outlined-autocomplete.scss`,
      `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-filled-autocomplete.scss`.
    - Compose exposed dropdown menus:
      `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ExposedDropdownMenu.kt`.

- [x] Badge (navigation bar/rail/drawer integration points).
  - Subtasks:
    - Import `md.comp.badge.*` scalars/colors via `material3_token_import`.
    - Implement `Badge::{dot,text}` + placement helper for navigation icons (Material Web labs parity).
    - Integrate badges into `NavigationBarItem` / `NavigationRailItem` and trailing badge labels into `NavigationDrawerItem`.
    - Headless suites: add `material3-badge.*.json` into `goldens/material3-headless/v1/`.
    - Gallery page: `apps/fret-ui-gallery` page `material3_badge`.
  - Evidence:
    - `ecosystem/fret-ui-material3/src/badge.rs`
    - `ecosystem/fret-ui-material3/src/tokens/badge.rs`
    - `ecosystem/fret-ui-material3/src/tokens/v30.rs` (v30 token injection wiring)
    - `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_badge_suite_goldens_v1`).
    - `goldens/material3-headless/v1/material3-badge.scale1_0.dark.tonal_spot.json` (representative; full matrix is generated).
    - `apps/fret-ui-gallery/src/spec.rs` + `apps/fret-ui-gallery/src/ui.rs`.
  - References:
    - `repo-ref/material-web/labs/badge` (placement offsets + large badge padding).
    - `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-badge.scss` (token source).

## P2

- [x] Bottom sheet (modal + standard), including focus and dismissal semantics.
  - Evidence:
    - `ecosystem/fret-ui-material3/src/bottom_sheet.rs`
    - `ecosystem/fret-ui-material3/src/tokens/sheet_bottom.rs`
    - `ecosystem/fret-ui-material3/src/tokens/v30.rs` (v30 token injection wiring)
    - `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs` (token import allowlist includes `md.comp.sheet.bottom.*`)
    - `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_bottom_sheet_suite_goldens_v1`)
    - `goldens/material3-headless/v1/material3-bottom-sheet.scale1_0.dark.tonal_spot.json` (representative; full matrix is generated)
    - `apps/fret-ui-gallery/src/spec.rs` + `apps/fret-ui-gallery/src/ui.rs` + `apps/fret-ui-gallery/src/docs.rs`
  - References:
    - `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-sheet-bottom.scss` (token source)
    - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SheetDefaults.kt`
- [x] Date picker (modal + docked), including focus and dismissal semantics.
  - Evidence:
    - `ecosystem/fret-ui-material3/src/date_picker.rs`
    - `ecosystem/fret-ui-material3/src/tokens/date_picker.rs`
    - `ecosystem/fret-ui-material3/src/tokens/v30.rs` (v30 token injection wiring)
    - `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs` (token import + codegen covers `md.comp.date-picker.*`)
    - `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_date_picker_suite_goldens_v1`)
    - `goldens/material3-headless/v1/material3-date-picker.scale1_0.dark.tonal_spot.json` (representative; full matrix is generated)
    - `apps/fret-ui-gallery/src/spec.rs` + `apps/fret-ui-gallery/src/ui.rs` + `apps/fret-ui-gallery/src/docs.rs`
  - References:
    - `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-date-picker-docked.scss`
    - `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-date-picker-modal.scss`
    - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DatePicker.kt`
- [x] Time picker (modal + docked), including focus and dismissal semantics.
  - Evidence:
    - `ecosystem/fret-ui-material3/src/time_picker.rs`
    - `ecosystem/fret-ui-material3/src/tokens/time_picker.rs`
    - `ecosystem/fret-ui-material3/src/tokens/time_input.rs`
    - `ecosystem/fret-ui-material3/src/tokens/v30.rs` (v30 token injection wiring)
    - `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs` (token import allowlist includes `md.comp.time-picker.*` and `md.comp.time-input.*`)
    - `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`material3_headless_time_picker_suite_goldens_v1`)
    - `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`time_picker_clock_dial_drag_updates_time`, `time_picker_selector_keyboard_arrows_step_time`)
    - `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`time_picker_time_input_replaces_and_auto_advances_hour`)
    - `goldens/material3-headless/v1/material3-time-picker.scale1_0.dark.tonal_spot.json` (representative; full matrix is generated)
    - `apps/fret-ui-gallery/src/spec.rs` + `apps/fret-ui-gallery/src/ui.rs` + `apps/fret-ui-gallery/src/docs.rs`
  - References:
    - `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-time-picker.scss`
    - `repo-ref/material-web/tokens/versions/v30_0/sass/_md-comp-time-input.scss`
    - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePicker.kt`
    - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/TimePickerDialog.kt`

## Mechanism follow-ups (only if required)

- [x] Semantics role coverage: add `SemanticsRole::Toolbar` (and validate with a concrete consumer).
  - Evidence:
    - `crates/fret-core/src/semantics.rs` (`SemanticsRole::Toolbar`)
    - `crates/fret-a11y-accesskit/src/lib.rs` (`map_role` → `Role::Toolbar`)
    - `ecosystem/fret-ui-material3/src/top_app_bar.rs` (`SemanticsRole::Toolbar`)
    - `ecosystem/fret-ui-material3/tests/radio_alignment.rs` (`top_app_bar_exposes_toolbar_semantics_role`)
- [x] Declarative invalidation: treat `ContainerProps` border/padding changes as layout-affecting.
  - Why: `ContainerProps.border` participates in layout sizing/insets (border-box), so border animations
    must invalidate layout to keep child bounds consistent with the painted border.
  - Evidence:
    - `crates/fret-ui/src/declarative/mount.rs` (`declarative_instance_change_mask` container diff)
    - `crates/fret-ui/src/declarative/tests/layout.rs` (`container_border_change_invalidates_child_layout`)
    - `crates/fret-ui/src/element.rs` (`ShadowStyle` / `RingStyle` derive `PartialEq`)
    - `ecosystem/fret-ui-material3/src/text_field.rs` (Filled hover state-layer overlay kept mounted; ADR 0181)
    - `ecosystem/fret-ui-material3/tests/text_field_hover.rs` (`filled_text_field_hover_overlay_survives_focus_transition`)
    - `goldens/material3-headless/v1/material3-text-field.*.json` (regenerated after the fix)
- [ ] Hoisted interaction sources: keep deferred; revisit only when we need deterministic “preview”
  APIs that cannot be expressed via scripted tests.
