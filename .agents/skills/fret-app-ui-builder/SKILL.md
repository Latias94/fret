---
name: fret-app-ui-builder
description: "This skill should be used when the user asks to \"build a Fret app UI\", \"refactor an existing Fret UI\", \"pick a theme/tokens baseline\", or \"compose shadcn-style recipes\". Provides a product-oriented workflow (style → recipes → UX polish → gates) to ship a cohesive, usable UI with early regression protection."
---

# Fret app UI builder

Use this skill when the goal is to build or refactor a **cohesive app surface** in Fret rather than chase one small parity bug.

## When to use

- You want to build a new Fret app UI from a design direction or product brief.
- You want to refactor an existing Fret UI into a more cohesive, token-driven surface.
- You need to pick a baseline theme/tokens setup before composing components.
- You want to compose shadcn-style recipes into an app shell and leave regression protection behind.

## Choose this vs adjacent skills

- Use this skill for **shipping a usable surface** with a clear style, shell, and gate.
- Use `fret-shadcn-source-alignment` when the main task is parity with shadcn/Radix.
- Use `fret-ui-review` when the main task is auditing an existing UI.
- Use `fret-diag-workflow` when the primary deliverable is a scripted repro or artifact.

## Inputs to collect (ask the user)

- What app surface are we building first (settings, command palette, workspace shell, inspector, data table, etc.)?
- What is the design direction (keywords, tone, density, contrast, product personality)?
- Which baseline style/preset should anchor the UI?
- Which interaction-heavy components are in scope (dialogs, menus, popovers, tables, docking, commands)?
- What regression protection is needed: diag script, test, perf probe, or all three?
- Which stable `test_id` surfaces do we need from day one?

Defaults if unclear:

- Start with one runnable screen, one baseline preset, one small token override, and one interaction gate.

## Design brief (1 minute, non-negotiable)

Write this down before styling anything:

- Product keywords (3–5): e.g. `dark`, `compact`, `technical`, `editor`, `high signal`
- Surface priority: e.g. `command palette first`, `settings first`, `workspace shell first`
- Constraints: e.g. `dense`, `keyboard-first`, `dockable`, `multi-panel`, `web + native`
- Differentiation hook: what should feel distinct from the default baseline?

See the direction mind model first:

- `references/mind-models/mm-design-direction.md`

## Smallest starting point (one command)

If you are in the Fret mono-repo:

- `cargo run -p fretboard -- new todo --name my-app --command-palette`

If you are in an external repo, start with `fret-external-app-mode` first.

## Quick start

1. Lock the 1-minute design brief.
2. Pick a baseline style and generate small token overrides.
3. Compose one app surface from an existing recipe.
4. Add stable `test_id` early and leave one interaction gate.
5. Run a polish pass one screen at a time.

## Workflow

### 0) Read the right reference note first

Use these references to keep the main skill lean:

- Design direction:
  - `references/mind-models/mm-design-direction.md`
- Theme and tokens:
  - `references/theme/token-groups.md`
  - `references/theme/editor-presets.md`
  - `references/mind-models/mm-theme-and-tokens.md`
- App recipes:
  - `references/recipes/INDEX.md`
  - `references/architecture/app-architecture-recipes.md`
- Engineering deep dives:
  - `references/engineering-notes.md`
- Polish pass:
  - `references/polish/polish-pass.md`

### 1) Lock direction + look first

- Pick a baseline preset.
- Keep overrides small and theme-driven.
- Use `stylegen.py` to suggest or generate a starting override set.

Helpful commands:

- `python3 .agents/skills/fret-app-ui-builder/scripts/stylegen.py --suggest "dark compact editor"`
- `python3 .agents/skills/fret-app-ui-builder/scripts/stylegen.py --style editor-compact > theme_overrides.json`

### 2) Compose the shell before details

- Decide the shell first: sidebar, top bar, center viewport, inspector, bottom panel.
- Keep one scroll root per pane.
- Start from a proven recipe instead of inventing a surface from scratch.

Suggested starts:

- command palette: `references/recipes/apps/app-command-palette.md`
- settings: `references/recipes/apps/app-settings-form.md`
- docking workspace: `references/recipes/apps/app-docking-workspace.md`

### 3) Make it keyboard-first and scriptable

- Commands need stable ids and explicit `when` gating.
- Add stable `test_id` to interactive affordances before the surface gets large.
- Leave one diag script per non-trivial interaction state machine.

### 4) Use engineering notes for the deep dives

When the surface is visually “almost there” but implementation details start fighting you, switch to:

- `references/engineering-notes.md`

That note covers:

- theme/tokens
- layout and overflow
- interaction policy and overlays
- commands/keymaps and IME
- virtualization and scheduling
- regression gates and automation surfaces

### 5) Finish with a style-agnostic polish pass

- Run the polish checklist on one screen at a time.
- Keep high-impact tweaks token-driven where possible.
- Avoid one-off per-widget magic numbers unless you are proving a recipe outcome first.

## Definition of done (what to leave behind)

Minimum deliverables (3-pack): Repro (smallest app surface), Gate (script/test), Evidence (anchors + command). See `fret-skills-playbook`.

- A cohesive baseline style is applied (preset + `ThemeConfig` override checked in).
- A short design brief exists (keywords + chosen baseline style + differentiation hook).
- The primary shell is usable (consistent spacing rhythm, focus-visible, predictable layering).
- At least one end-to-end interaction gate exists:
  - `tools/diag-scripts/*.json` with stable `test_id` selectors and `capture_bundle`.
- Evidence anchors are recorded (recipe files used + key code paths + script/test path).

## Evidence anchors

- Shared conventions: `.agents/skills/fret-skills-playbook/SKILL.md`
- Style generation: `.agents/skills/fret-app-ui-builder/scripts/stylegen.py`
- Style catalog: `.agents/skills/fret-app-ui-builder/references/style_catalog.json`
- Recipes + mind models: `.agents/skills/fret-app-ui-builder/references/`
- Engineering deep dives: `.agents/skills/fret-app-ui-builder/references/engineering-notes.md`
- Polish pass: `.agents/skills/fret-app-ui-builder/references/polish/polish-pass.md`
- Diag + perf gates: `.agents/skills/fret-diag-workflow/SKILL.md`, `tools/diag-scripts/`, `tools/perf/`

## Examples

- Example: compose a cohesive settings screen
  - User says: "Build a settings page with shadcn-style components."
  - Actions: pick a baseline theme, compose a recipe, add `test_id` early, then leave a gate.
  - Result: a shippable page that is easy to regress-test.

- Example: polish a UI without redesigning everything
  - User says: "It works but looks off—polish spacing/typography."
  - Actions: adjust density, radius, elevation, and hierarchy; keep diffs token-driven.
  - Result: high-impact polish with low churn.

## Common pitfalls

- Styling per-component with magic numbers instead of token overrides.
- Skipping the design brief and shipping the default baseline with no point of view.
- Building overlay state machines without leaving a diag script gate.
- Missing `test_id` targets, so scripts rot immediately.
- Mixing parity work with new design work without gates.

## Troubleshooting

- Symptom: visual tweaks keep regressing.
  - Fix: push changes into tokens/recipes; avoid one-off per-widget overrides.
- Symptom: you cannot script the new UI reliably.
  - Fix: add `test_id` and use `fret-diag-workflow` to lock the flow with a script.

## Related skills

- `fret-external-app-mode`
- `fret-skills-playbook`
- `fret-diag-workflow`
- `fret-ui-review`
- `fret-shadcn-source-alignment`
