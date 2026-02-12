---
name: fret-app-ui-builder
description: "Product-oriented workflow for building a good-looking, usable app with Fret quickly: choose a baseline style, apply theme overrides, compose shadcn recipes, and leave regression gates. Use when starting a new app or refactoring an existing app UI."
---

# Fret app UI builder (product-oriented)

This skill is the “golden path” for **framework users** and **app developers** who want to build a
good-looking, editor-grade UI with Fret (not just “it renders”).

It composes the existing skills into one product-oriented workflow:

- style + tokens (`fret-design-system-styles`)
- visual hierarchy (`fret-ui-ux-guidelines`)
- shadcn composition recipes (`fret-shadcn-app-recipes`)
- commands + keymaps (`fret-commands-and-keymap`)
- debug/gates (`fret-diag-workflow`, `fret-perf-workflow` when perf matters)

## When to use

- Starting a new app with Fret and you want it to look cohesive on day 1.
- Refactoring an existing app UI that “works” but is visually inconsistent or hard to use.
- You want a repeatable way to produce a good UI (style + layout + behavior + gates).

## Inputs to collect (ask the user)

- Surface type: editor workspace / settings forms / dashboard / content viewer.
- Scheme + density: light/dark + compact/default/comfortable.
- Vibe keywords: minimal / soft / neubrutal / hud / glass overlays / high-contrast.
- Primary modality: keyboard-first / mouse-first / mixed.
- Must-have flows: command palette, settings, navigation sidebar, data table, docking workspace.
- Evidence needs: diag script gate only, or screenshots/pixel checks too?

Defaults if unclear:

- Build a dark + compact editor shell, with command palette enabled and stable `test_id` targets.

## Smallest starting point (one command)

If you are in the Fret mono-repo (fastest way to start from a runnable template):

- `cargo run -p fretboard -- new todo --name my-app --command-palette`

If you are in an external app repo, start with `fret-external-app-mode` first.

## Quick start

1) Pick a baseline style and generate `ThemeConfig` overrides:

- `python3 .agents/skills/fret-design-system-styles/scripts/stylegen.py --suggest "dark compact editor"`
- `python3 .agents/skills/fret-design-system-styles/scripts/stylegen.py --style editor-compact > theme_overrides.json`

2) Apply the baseline preset + overrides (see `fret-design-system-styles`).

3) Compose one “app surface” recipe and keep it minimal:

- command palette: `fret-shadcn-app-recipes` → `references/recipes/apps/app-command-palette.md`
- settings: `fret-shadcn-app-recipes` → `references/recipes/apps/app-settings-form.md`
- workspace shell/docking: `fret-shadcn-app-recipes` → `references/recipes/apps/app-docking-workspace.md`

4) Add stable `test_id` to interactive affordances and leave a diag script gate:

- `tools/diag-scripts/<scenario>.json` (schema v2 preferred) + `capture_bundle`.

## Workflow

1) **Lock the look first (theme + density).**
   - Pick a baseline preset (New York v4).
   - Apply a small `ThemeConfig` override (1–2 axes at a time: density + ring first).
2) **Compose the shell before details.**
   - Decide: sidebar + top bar + center viewport + inspector (optional) + bottom panel (optional).
   - Keep one scroll root per pane.
3) **Make it keyboard-first by default.**
   - Commands have stable IDs.
   - Keymaps have explicit `when` gating (avoid firing inside text inputs).
   - Focus-visible is consistent (ring width/offset).
4) **Use recipes for interaction-heavy components.**
   - Overlays (select/menu/dialog/combobox) must follow Radix-aligned policy.
   - Put policy in `ecosystem/` layers (hooks + primitives), not runtime.
5) **Leave gates early (don’t wait).**
   - Add `test_id` targets.
   - Add one diag script per interaction state machine (open → interact → dismiss).
   - If perf is a concern, run a small perf probe before landing.

## Definition of done (what to leave behind)

Minimum deliverables (3-pack): Repro (smallest app surface), Gate (script/test), Evidence (anchors + command). See `fret-skills-playbook`.

- A cohesive baseline style is applied (preset + `ThemeConfig` override checked in).
- The primary shell is usable (consistent spacing rhythm, focus-visible, predictable layering).
- At least one end-to-end interaction gate exists:
  - `tools/diag-scripts/*.json` with stable `test_id` selectors and `capture_bundle`.
- Evidence anchors are recorded (recipe files used + key code paths + script/test path).

## Evidence anchors

- Shared conventions: `.agents/skills/fret-skills-playbook/SKILL.md`
- Style generation: `.agents/skills/fret-design-system-styles/scripts/stylegen.py`
- Recipes + mind models: `.agents/skills/fret-shadcn-app-recipes/references/`
- Diag gates: `.agents/skills/fret-diag-workflow/SKILL.md`, `tools/diag-scripts/`

## Common pitfalls

- Styling per-component with magic numbers instead of token overrides.
- Building overlay state machines without leaving a diag script gate.
- Missing `test_id` targets (scripts rot immediately).
- Mixing parity work with new design work without gates.

## Related skills

- `fret-external-app-mode`
- `fret-skills-playbook`
- `fret-design-system-styles`
- `fret-ui-ux-guidelines`
- `fret-shadcn-app-recipes`
- `fret-diag-workflow`
