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
