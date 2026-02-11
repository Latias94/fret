# State-Driven Style Resolution v1 (Workstream)

Contract gate:

- `docs/adr/0219-state-driven-style-resolution-v1.md`
- `docs/adr/0220-ecosystem-style-override-surface-v1.md`

## Goal

Make “state → style” authoring consistent across ecosystem component libraries, so users can build:

- shadcn/ui-aligned components (`fret-ui-shadcn`),
- future design systems (e.g. Material 3) with predictable override points,
- policy-heavy UI kits without re-inventing state precedence rules.

## Planned Worktree

- Worktree path: `F:\SourceCodes\Rust\fret-worktrees\state-driven-style-resolution-v1`
- Branch: `refactor/state-driven-style-resolution-v1`

Note: this workstream was initially executed in a dedicated worktree, but the resulting contracts
and migrations have since been merged into `main`. The sections below reflect the current
repository state, not the historical worktree.

## Current Baseline (main)

- `WidgetStates` + `WidgetStateProperty<T>` exists in `fret-ui-kit`.
- `ColorFallback::ThemeTokenAlphaMul` supports minimal hover/active derivation.
- Pilot migration: `fret-ui-shadcn::Button` uses per-state tokens + focus-visible border semantics.

## Worktree Progress

- `fret-ui-shadcn` exports v1 `*Style` override surfaces for core interactive controls (ADR 0220).
- `fret-ui-kit` exports shared helpers to resolve ADR 0220 override slots (`resolve_override_slot*`).
- Some shadcn surfaces still use ad-hoc `PressableState` branching (see adoption snapshot).

Material 3 is tracked separately:

- `docs/workstreams/material3-todo.md`
- `docs/workstreams/material3-style-api-alignment-v1.md`

## TODO (Priority Order)

Status legend: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

### P0 — API Shape + First-Class Overrides

- [x] SDSR-000 Add `WidgetStates` / `WidgetStateProperty<T>` and minimal token-derived fallback (ADR 0219 baseline).
- [x] SDSR-010 Pilot: migrate `Button` to state-driven background + focus-visible border semantics.
- [x] SDSR-011 Worktree: introduce and export `ButtonStyle` (override background/foreground/border with per-state properties).

- [x] SDSR-020 Define a shared “style struct” pattern for shadcn controls:
  - `*Style` structs with `OverrideSlot<T>` fields (`Option<WidgetStateProperty<Option<T>>>`).
  - `merged()` semantics (right-biased overrides, no deep merge).
  - Per-control `style(...)` builder method.

- [x] SDSR-030 Standardize stateful `*Style` slots to `WidgetStateProperty<Option<T>>` and resolve-time fallback (Flutter-style partial overrides).
- [x] SDSR-031 Add ADR 0220 for the ecosystem `*Style` override surface and update `docs/shadcn-style-override-patterns.md`.

### P1 — Migrate Core Interactive Controls

- [x] SDSR-100 Toggle: replace ad-hoc hovered/pressed/focused branches with `WidgetStates` + `WidgetStateProperty`.
- [x] SDSR-110 Toggle: add and export `ToggleStyle`.

- [x] SDSR-120 Tabs: migrate trigger/button-like styling to `WidgetStates`.
- [x] SDSR-121 Tabs: add and export `TabsStyle` (at least tab trigger background/foreground/border/ring).

- [x] SDSR-130 Input: introduce `InputStyle` for chrome/background/border/ring (note: some chrome already resolves via `fret-ui-kit::recipes::input`).
- [x] SDSR-131 Input: ensure focus-visible semantics for ring/border (aligned with ADR 0061).

- [x] SDSR-140 Checkbox: migrate checked/focus-visible styling to `WidgetStates`.
- [x] SDSR-141 Checkbox: add and export `CheckboxStyle`.

- [x] SDSR-150 Switch: migrate checked/focus-visible styling to `WidgetStates`.
- [x] SDSR-151 Switch: add and export `SwitchStyle`.

- [x] SDSR-160 RadioGroup: migrate hover/active/focus-visible styling to `WidgetStates`.
- [x] SDSR-161 RadioGroup: add and export `RadioGroupStyle`.

- [x] SDSR-170 Select: migrate trigger/option hover/active/open styling to `WidgetStates`.
- [x] SDSR-171 Select: add and export `SelectStyle`.

- [x] SDSR-180 Slider: migrate drag/focus-visible styling to `WidgetStates`.
- [x] SDSR-181 Slider: add and export `SliderStyle`.

- [x] SDSR-190 ToggleGroup: migrate hover/active/focus-visible styling to `WidgetStates`.
- [x] SDSR-191 ToggleGroup: add and export `ToggleGroupStyle`.

### P2 — Overlay/Menu Surfaces (Radix-like)

- [x] SDSR-200 DropdownMenu items: unify hover/active/disabled/open states via `WidgetStates`.
- [x] SDSR-201 Menubar items: unify hover/active/disabled/open states via `WidgetStates`.
- [x] SDSR-210 Tooltip / HoverCard: keep policy-only styling in v1 (document rationale).

### P3 — Token Naming + Slot Vocabulary

- [x] SDSR-300 Document state token key conventions for shadcn components (background/foreground/border/ring).
- [x] SDSR-310 Decide when to use semantic base keys (`primary`, `destructive`) vs component keys (`button.*`).
- [x] SDSR-320 Decide how “selected” maps to tokens (e.g. `*.selected.background`) for toggles/tabs/list rows.

### P4 — Performance & Ergonomics

- [x] SDSR-400 Avoid heap allocations in hot paths (e.g. store overrides inline or in smallvec; measure before changing).
- [x] SDSR-410 Add utilities to compute `WidgetStates` from `PressableState` + focus-visible policy (reduce copy/paste).

### P5 — Material 3 Consumer (moved)

Material 3 is tracked in a dedicated workstream so this document can stay design-system-agnostic:

- `docs/workstreams/material3-style-api-alignment-v1.md`

Note: Material3 has since evolved into a broader Compose-inspired foundation refactor using the
`md.sys.*` / `md.comp.*` token namespaces. The authoritative tracking docs are the Material3
workstreams linked above.

## Shadcn adoption snapshot (main)

This snapshot focuses on “state → style” authoring consistency, not full visual parity.

**Completed (exports `*Style` + `.style(...)`)**

- `ecosystem/fret-ui-shadcn/src/button.rs`
- `ecosystem/fret-ui-shadcn/src/checkbox.rs`
- `ecosystem/fret-ui-shadcn/src/combobox.rs`
- `ecosystem/fret-ui-shadcn/src/input.rs`
- `ecosystem/fret-ui-shadcn/src/item.rs`
- `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`
- `ecosystem/fret-ui-shadcn/src/radio_group.rs`
- `ecosystem/fret-ui-shadcn/src/select.rs`
- `ecosystem/fret-ui-shadcn/src/slider.rs`
- `ecosystem/fret-ui-shadcn/src/switch.rs`
- `ecosystem/fret-ui-shadcn/src/tabs.rs`
- `ecosystem/fret-ui-shadcn/src/toggle.rs`
- `ecosystem/fret-ui-shadcn/src/toggle_group.rs`

**Migrated to `WidgetStates` but no public `*Style` surface (token-driven only, for now)**

- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- `ecosystem/fret-ui-shadcn/src/menubar.rs`

**Likely still ad-hoc `PressableState` branching (candidates for follow-up)**

- `ecosystem/fret-ui-shadcn/src/breadcrumb.rs`
- `ecosystem/fret-ui-shadcn/src/calendar.rs`
- `ecosystem/fret-ui-shadcn/src/calendar_range.rs`
- `ecosystem/fret-ui-shadcn/src/command.rs`
- `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- `ecosystem/fret-ui-shadcn/src/dialog.rs`
- `ecosystem/fret-ui-shadcn/src/input_group.rs`
- `ecosystem/fret-ui-shadcn/src/pagination.rs`
- `ecosystem/fret-ui-shadcn/src/sidebar.rs`

## Milestones

1. Define stable per-component style structs
   - Example: `ButtonStyle { background: WidgetStateProperty<ColorRef>, ... }`
   - Provide `*_style_from_theme(theme)` helpers and override/merge rules.

2. Migrate core shadcn components to the unified primitive
   - Buttons, inputs, toggles, tabs, menus, list rows.
   - Replace ad-hoc hover/active/focus handling with `WidgetStates`.

3. Lock token naming conventions + slot vocabulary
   - Document recommended keys (background/foreground/border/ring).
   - Decide how “semantic base keys” map to component-level keys.

4. Performance / ergonomics hardening
   - Avoid per-frame heap allocations in hot paths.
   - Ensure derived fallbacks are cheap and deterministic.

## Evidence Log (append as work progresses)

- `ecosystem/fret-ui-kit/src/style/state.rs`
- `ecosystem/fret-ui-kit/src/style/tokens.rs`
- `ecosystem/fret-ui-shadcn/src/button.rs`
- `ecosystem/fret-ui-shadcn/src/lib.rs`
- `ecosystem/fret-ui-shadcn/src/toggle.rs`
- `ecosystem/fret-ui-shadcn/src/combobox.rs`
- `ecosystem/fret-ui-shadcn/src/tabs.rs`
- `ecosystem/fret-ui-shadcn/src/input.rs`
- `crates/fret-ui/src/text_input/widget.rs`
- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- `ecosystem/fret-ui-shadcn/src/menubar.rs`
- `ecosystem/fret-ui-shadcn/src/tooltip.rs`
- `ecosystem/fret-ui-shadcn/src/hover_card.rs`
- `ecosystem/fret-ui-shadcn/src/checkbox.rs`
- `ecosystem/fret-ui-shadcn/src/switch.rs`
- `ecosystem/fret-ui-shadcn/src/radio_group.rs`
- `ecosystem/fret-ui-shadcn/src/select.rs`
- `ecosystem/fret-ui-shadcn/src/slider.rs`
- `ecosystem/fret-ui-shadcn/src/toggle_group.rs`
- `ecosystem/fret-ui-material3/src/button.rs`
- `ecosystem/fret-ui-material3/src/icon_button.rs`
- `ecosystem/fret-ui-material3/src/checkbox.rs`
- `ecosystem/fret-ui-material3/src/switch.rs`
- `ecosystem/fret-ui-material3/src/radio.rs`
- `ecosystem/fret-ui-material3/src/tabs.rs`
- `ecosystem/fret-ui-material3/src/text_field.rs`
- SDSR-210 decision: keep Tooltip/HoverCard styling policy-only in v1 (theme tokens + overlay motion); no `WidgetStates`-driven surface overrides yet because the trigger is user-supplied and the content surface is not an interactive control.
- SDSR-410 evidence: `WidgetStates::from_pressable(...)` in `ecosystem/fret-ui-kit/src/style/state.rs`, applied in `ecosystem/fret-ui-shadcn/src/tabs.rs`, `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`, `ecosystem/fret-ui-shadcn/src/menubar.rs`.
- SDSR-300/310/320: `docs/shadcn-style-token-conventions.md`
- SDSR-020: `docs/shadcn-style-override-patterns.md` + exported `*Style` structs in `ecosystem/fret-ui-shadcn/src/{button,checkbox,radio_group,select,slider,switch,toggle,toggle_group,tabs,input}.rs`
- SDSR-180/181: `SliderStyle` + `WidgetStates` in `ecosystem/fret-ui-shadcn/src/slider.rs` and `ecosystem/fret-ui-shadcn/src/lib.rs`
- SDSR-140/141: `CheckboxStyle` + `WidgetStates` in `ecosystem/fret-ui-shadcn/src/checkbox.rs` and `ecosystem/fret-ui-shadcn/src/lib.rs`
- SDSR-150/151: `SwitchStyle` + `WidgetStates` in `ecosystem/fret-ui-shadcn/src/switch.rs` and `ecosystem/fret-ui-shadcn/src/lib.rs`
- SDSR-160/161: `RadioGroupStyle` + `WidgetStates` in `ecosystem/fret-ui-shadcn/src/radio_group.rs` and `ecosystem/fret-ui-shadcn/src/lib.rs`
- SDSR-170/171: `SelectStyle` + `WidgetStates` in `ecosystem/fret-ui-shadcn/src/select.rs` and `ecosystem/fret-ui-shadcn/src/lib.rs`
