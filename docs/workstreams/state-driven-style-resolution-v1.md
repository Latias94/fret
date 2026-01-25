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

- [ ] SDSR-020 Define a shared “style struct” pattern for shadcn controls:
  - `*Style` structs with `Option<WidgetStateProperty<...>>` fields.
  - `merged()` semantics (right-biased overrides, no deep merge).
  - Per-control `style(...)` builder method.

### P1 — Migrate Core Interactive Controls

- [ ] SDSR-100 Toggle: replace ad-hoc hovered/pressed/focused branches with `WidgetStates` + `WidgetStateProperty`.
- [ ] SDSR-110 Toggle: add and export `ToggleStyle`.

- [ ] SDSR-120 Tabs: migrate trigger/button-like styling to `WidgetStates`.
- [ ] SDSR-121 Tabs: add and export `TabsStyle` (at least tab trigger background/foreground/border/ring).

- [ ] SDSR-130 Input: introduce `InputStyle` for chrome/background/border/ring (note: some chrome already resolves via `fret-ui-kit::recipes::input`).
- [ ] SDSR-131 Input: ensure focus-visible semantics for ring/border (aligned with ADR 0061).

### P2 — Overlay/Menu Surfaces (Radix-like)

- [ ] SDSR-200 Menu items (dropdown/menu-bar): unify hover/active/disabled selection states via `WidgetStates`.
- [ ] SDSR-210 Tooltip / HoverCard: decide whether per-state styling is needed or keep policy-only (document rationale).

### P3 — Token Naming + Slot Vocabulary

- [ ] SDSR-300 Document state token key conventions for shadcn components (background/foreground/border/ring).
- [ ] SDSR-310 Decide when to use semantic base keys (`primary`, `destructive`) vs component keys (`button.*`).
- [ ] SDSR-320 Decide how “selected” maps to tokens (e.g. `*.selected.background`) for toggles/tabs/list rows.

### P4 — Performance & Ergonomics

- [ ] SDSR-400 Avoid heap allocations in hot paths (e.g. store overrides inline or in smallvec; measure before changing).
- [ ] SDSR-410 Add utilities to compute `WidgetStates` from `PressableState` + focus-visible policy (reduce copy/paste).

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
