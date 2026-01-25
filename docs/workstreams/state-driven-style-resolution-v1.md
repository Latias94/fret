# State-Driven Style Resolution v1 (Workstream)

Contract gate:

- `docs/adr/1158-state-driven-style-resolution-v1.md`

## Goal

Make “state → style” authoring consistent across ecosystem component libraries, so users can build:

- shadcn/ui-aligned components (`fret-ui-shadcn`),
- future design systems (e.g. Material 3) with predictable override points,
- policy-heavy UI kits without re-inventing state precedence rules.

## Planned Worktree

- Worktree path: `F:\SourceCodes\Rust\fret-worktrees\state-driven-style-resolution-v1`
- Branch: `refactor/state-driven-style-resolution-v1`

## Current Baseline (main)

- `WidgetStates` + `WidgetStateProperty<T>` exists in `fret-ui-kit`.
- `ColorFallback::ThemeTokenAlphaMul` supports minimal hover/active derivation.
- Pilot migration: `fret-ui-shadcn::Button` uses per-state tokens + focus-visible border semantics.

## Worktree Progress

- `ButtonStyle` exists and is exported in `fret-ui-shadcn` (v0: optional overrides; merged into the variant-derived defaults).

## TODO (Priority Order)

Status legend: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

### P0 — API Shape + First-Class Overrides

- [x] SDSR-000 Add `WidgetStates` / `WidgetStateProperty<T>` and minimal token-derived fallback (ADR 1158 baseline).
- [x] SDSR-010 Pilot: migrate `Button` to state-driven background + focus-visible border semantics.
- [x] SDSR-011 Worktree: introduce and export `ButtonStyle` (override background/foreground/border with per-state properties).

- [x] SDSR-020 Define a shared “style struct” pattern for shadcn controls:
  - `*Style` structs with `Option<WidgetStateProperty<...>>` fields.
  - `merged()` semantics (right-biased overrides, no deep merge).
  - Per-control `style(...)` builder method.

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

### P2 — Overlay/Menu Surfaces (Radix-like)

- [x] SDSR-200 DropdownMenu items: unify hover/active/disabled/open states via `WidgetStates`.
- [x] SDSR-201 Menubar items: unify hover/active/disabled/open states via `WidgetStates`.
- [x] SDSR-210 Tooltip / HoverCard: keep policy-only styling in v1 (document rationale).

### P3 — Token Naming + Slot Vocabulary

- [x] SDSR-300 Document state token key conventions for shadcn components (background/foreground/border/ring).
- [x] SDSR-310 Decide when to use semantic base keys (`primary`, `destructive`) vs component keys (`button.*`).
- [x] SDSR-320 Decide how “selected” maps to tokens (e.g. `*.selected.background`) for toggles/tabs/list rows.

### P4 — Performance & Ergonomics

- [ ] SDSR-400 Avoid heap allocations in hot paths (e.g. store overrides inline or in smallvec; measure before changing).
- [x] SDSR-410 Add utilities to compute `WidgetStates` from `PressableState` + focus-visible policy (reduce copy/paste).

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
- SDSR-210 decision: keep Tooltip/HoverCard styling policy-only in v1 (theme tokens + overlay motion); no `WidgetStates`-driven surface overrides yet because the trigger is user-supplied and the content surface is not an interactive control.
- SDSR-410 evidence: `WidgetStates::from_pressable(...)` in `ecosystem/fret-ui-kit/src/style/state.rs`, applied in `ecosystem/fret-ui-shadcn/src/tabs.rs`, `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`, `ecosystem/fret-ui-shadcn/src/menubar.rs`.
- SDSR-300/310/320: `docs/shadcn-style-token-conventions.md`
- SDSR-020: `docs/shadcn-style-override-patterns.md` + exported `*Style` structs in `ecosystem/fret-ui-shadcn/src/{button,checkbox,radio_group,select,switch,toggle,tabs,input}.rs`
- SDSR-140/141: `CheckboxStyle` + `WidgetStates` in `ecosystem/fret-ui-shadcn/src/checkbox.rs` and `ecosystem/fret-ui-shadcn/src/lib.rs`
- SDSR-150/151: `SwitchStyle` + `WidgetStates` in `ecosystem/fret-ui-shadcn/src/switch.rs` and `ecosystem/fret-ui-shadcn/src/lib.rs`
- SDSR-160/161: `RadioGroupStyle` + `WidgetStates` in `ecosystem/fret-ui-shadcn/src/radio_group.rs` and `ecosystem/fret-ui-shadcn/src/lib.rs`
- SDSR-170/171: `SelectStyle` + `WidgetStates` in `ecosystem/fret-ui-shadcn/src/select.rs` and `ecosystem/fret-ui-shadcn/src/lib.rs`
