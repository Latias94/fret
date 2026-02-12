---
name: fret-app-ui-builder
description: "Product-oriented workflow for building a good-looking, usable app with Fret quickly: choose a baseline style, apply theme overrides, compose shadcn recipes, and leave regression gates. Use when starting a new app or refactoring an existing app UI."
---

# Fret app UI builder (product-oriented)

This skill is the “golden path” for **framework users** and **app developers** who want to build a
good-looking, editor-grade UI with Fret (not just “it renders”).

It is intentionally **self-contained**: it includes theming, recipes, and the most common “deep dives”
(overlays, commands/keymaps, text input/IME, virtualization, scheduling), plus diagnostics gates.

## When to use

- Starting a new app with Fret and you want it to look cohesive on day 1.
- Refactoring an existing app UI that “works” but is visually inconsistent or hard to use.
- You want a repeatable way to produce a good UI (style + layout + behavior + gates).

## Choose this vs adjacent skills

- Use this skill for an end-to-end **product workflow** (style → recipes → UX polish → gates).
- Use `fret-ui-review` when the task is “review/audit this UI” rather than building/refactoring it.
- Use `fret-diag-workflow` when the primary goal is a deterministic repro + gate (not “make it look good”).
  - If you believe a framework/eco component is wrong, use `fret-diag-workflow` to produce a minimal repro bundle + script.
    Parity/alignment work is typically owned by framework/eco authors (see `fret-framework-maintainer-guide`).

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

- `python3 .agents/skills/fret-app-ui-builder/scripts/stylegen.py --suggest "dark compact editor"`
- `python3 .agents/skills/fret-app-ui-builder/scripts/stylegen.py --style editor-compact > theme_overrides.json`

2) Apply the baseline preset + overrides (keep it small: density + ring first).
   - Token cheat sheet: `references/theme/token-groups.md`
   - Starter presets: `references/theme/editor-presets.md`

3) Compose one “app surface” recipe and keep it minimal (start from the in-skill references):

- command palette: `references/recipes/apps/app-command-palette.md`
- settings: `references/recipes/apps/app-settings-form.md`
- workspace shell/docking: `references/recipes/apps/app-docking-workspace.md`

3.5) Run a style-agnostic polish pass on that surface (one screen at a time):

- `references/polish/polish-pass.md`

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
   - Put policy in `ecosystem/` layers (action hooks + policy helpers), not runtime.
5) **Leave gates early (don’t wait).**
   - Add `test_id` targets.
   - Add one diag script per interaction state machine (open → interact → dismiss).
   - If perf is a concern, run a small perf probe before landing.
6) **Polish pass (style-agnostic).**
   - Apply the checklist to one surface at a time:
     - `references/polish/polish-pass.md`

## Engineering notes (the “deep dives” you will hit)

These are the most common places app authors get stuck. Keep the work **layered**:

- `crates/*`: mechanisms and hard contracts
- `ecosystem/*`: policy and recipes

### Theme/tokens (make it cohesive fast)

- Prefer “theme-first”: one preset + small overrides. Avoid per-component magic numbers.
- Generator helper:
  - `python3 .agents/skills/fret-app-ui-builder/scripts/stylegen.py --suggest "<keywords>"`
- High-leverage knobs:
  - `references/theme/token-groups.md`
- Editor-oriented presets (copy/paste overrides):
  - `references/theme/editor-presets.md`

### Layout/overflow (avoid clipped focus rings)

- Use token-driven layout/chrome via `UiBuilder` (declarative-only).
- Don’t clip focus rings by accident: keep the pressable/root overflow visible; clip only inside chrome.

### Interaction policy (press/dismiss/roving/typeahead/timers)

Rule: `crates/fret-ui` is mechanism-only; policy belongs in components via action hooks.

- Pressable activate/toggle: prefer `fret-ui-kit` helpers such as `cx.pressable_toggle_bool(&open)`
- Dismiss policy for overlays: attach a dismiss hook (component-owned), don’t bake dismissal into runtime widgets

### Overlays + focus (Radix-aligned outcomes)

- Pick the correct family: menu vs popover vs modal.
- Menus are usually non-click-through on outside press.
- Focus restore to the trigger on close (unless explicitly overridden).

### Commands/keymaps (keyboard-first without breaking typing)

- Treat `CommandId` as stable contracts.
- Always add explicit `when` gating for global shortcuts (block inside text inputs / IME composition).

### Text input + IME (don’t break composition)

- Keep channels separate: `KeyDown` vs `TextInput` vs `ImeEvent` (ADR 0012).
- While composing, IME gets first refusal on Tab/Escape/arrows/etc.
- Provide caret rect feedback for candidate window placement.

### Virtualized lists (stable identity is non-negotiable)

- Use keyed virtualization; keys must come from the model (never the row index).
- Prefer fixed row heights when possible (editor UIs).

### Scheduling/animation (don’t leak continuous frames)

- Tie continuous frames leases to element lifetime (store in element-local state).
- Prefer runner-owned timers/effects (deterministic and diagnosable).

## Definition of done (what to leave behind)

Minimum deliverables (3-pack): Repro (smallest app surface), Gate (script/test), Evidence (anchors + command). See `fret-skills-playbook`.

- A cohesive baseline style is applied (preset + `ThemeConfig` override checked in).
- The primary shell is usable (consistent spacing rhythm, focus-visible, predictable layering).
- At least one end-to-end interaction gate exists:
  - `tools/diag-scripts/*.json` with stable `test_id` selectors and `capture_bundle`.
- Evidence anchors are recorded (recipe files used + key code paths + script/test path).

## Evidence anchors

- Shared conventions: `.agents/skills/fret-skills-playbook/SKILL.md`
- Style generation: `.agents/skills/fret-app-ui-builder/scripts/stylegen.py`
- Style catalog (used by `stylegen.py`): `.agents/skills/fret-app-ui-builder/references/style_catalog.json`
- Recipes + mind models: `.agents/skills/fret-app-ui-builder/references/`
- Polish pass rules: `.agents/skills/fret-app-ui-builder/references/polish/polish-pass.md`
- Diag + perf gates: `.agents/skills/fret-diag-workflow/SKILL.md`, `tools/diag-scripts/`, `tools/perf/`

## Common pitfalls

- Styling per-component with magic numbers instead of token overrides.
- Building overlay state machines without leaving a diag script gate.
- Missing `test_id` targets (scripts rot immediately).
- Mixing parity work with new design work without gates.

## Related skills

- `fret-external-app-mode`
- `fret-skills-playbook`
- `fret-diag-workflow`
- `fret-ui-review`
  - Framework/eco authors only: `fret-shadcn-source-alignment`
