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

- Surface type: editor workspace / settings forms / dashboard / content viewer.
- Scheme + density: light/dark + compact/default/comfortable.
- Vibe keywords: minimal / soft / neubrutal / hud / glass overlays / high-contrast.
- Differentiation hook: what should a user remember after 3 seconds?
- Typography constraints: any required fonts, mono usage, or code-heavy surfaces?
- Motion tolerance: none / subtle / expressive (and whether reduced-motion is required).
- Primary modality: keyboard-first / mouse-first / mixed.
- Must-have flows: command palette, settings, navigation sidebar, data table, docking workspace.
- Evidence needs: diag script gate only, or screenshots/pixel checks too?
- State ownership: plain local snapshot, narrow bridge, or shared `Model<T>`?
- What authoring surface should the result teach: app-facing `fret`, direct `fret_ui_shadcn`, or lower-level ecosystem internals?
- What app surface are we building first (settings, command palette, workspace shell, inspector, data table, etc.)?
- What is the design direction (keywords, tone, density, contrast, product personality)?
- Which baseline style/preset should anchor the UI?
- Which interaction-heavy components are in scope (dialogs, menus, popovers, tables, docking, commands)?
- What regression protection is needed: diag script, test, perf probe, or all three?
- Which stable `test_id` surfaces do we need from day one?

Defaults if unclear:

- Start with one runnable screen, one baseline preset, one small token override, and one interaction gate.
- For first-party direct-crate shadcn examples, prefer `use fret_ui_shadcn::{facade as shadcn, prelude::*};`; keep raw escapes explicit via `shadcn::raw::*`.

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
2. Decide which authoring surface the UI should teach before copying imports or helper patterns.
3. Pick a baseline style and generate small token overrides.
4. Compose one app surface from an existing recipe or UI Gallery exemplar.
5. Add stable `test_id` early and leave one interaction gate.
6. Run a polish pass one screen at a time.

## Workflow

### 0) Read the right reference note first

Use these references to keep the main skill lean:

- Design direction:
  - `references/mind-models/mm-design-direction.md`
- Theme and tokens:
  - `references/theme/token-groups.md`
  - `references/theme/editor-presets.md`
  - `references/mind-models/mm-theme-and-tokens.md`
- Surface/layer selection:
  - `docs/crate-usage-guide.md`
  - `docs/shadcn-declarative-progress.md`
- First-party shadcn exemplar + evidence workflow:
  - `.agents/skills/fret-shadcn-source-alignment/references/ui-gallery-exemplar-and-evidence.md`
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

2.5) Run a widget-state sanity check before freezing the app-side state shape:

- `references/mind-models/mm-widget-state-surfaces.md`
- If a simple local list needs per-row `Model<T>` just to satisfy a widget recipe, stop and audit component parity before adding app helpers.

3) Compose one “app surface” recipe and keep it minimal (start from the in-skill references):
- Decide the shell first: sidebar, top bar, center viewport, inspector, bottom panel.
- Keep one scroll root per pane.
- Start from a proven recipe instead of inventing a surface from scratch.
- For first-party shadcn pages, prefer snippet-backed UI Gallery exemplars:
  - snippet file = canonical example source
  - page file = documentation composition
  - driver/diag glue = automation and geometry ownership
- Do not mix app-facing `fret` examples with direct `fret_ui_shadcn` examples in the same teaching surface.

Suggested starts:

- command palette: `references/recipes/apps/app-command-palette.md`
- settings: `references/recipes/apps/app-settings-form.md`
- docking workspace: `references/recipes/apps/app-docking-workspace.md`

### 3) Make it keyboard-first and scriptable

- Commands need stable ids and explicit `when` gating.
- Add stable `test_id` to interactive affordances before the surface gets large.
- Leave one diag script per non-trivial interaction state machine.
- When layout ownership is fragile, add geometry assertions or `capture_layout_sidecar` before screenshot churn.

### 3.5) Capture evidence before polishing tokens

- Layout drift first:
  - align `w_full`, `flex_1`, `min_w_0`, stretch/shrink ownership
  - prove it with geometry assertions or `capture_layout_sidecar`
- Visual drift next:
  - use `capture_screenshot` for chrome, clipping, focus rings, and constrained viewport evidence
- Interaction drift always:
  - keep `capture_bundle` in the final scriptable path

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
- When porting from shadcn/Tailwind, align **layout constraints first** (e.g. `w-full`, `flex-1`, `items-stretch`, `min-w-0`) before chasing pixels/tokens:
  - `references/mind-models/mm-layout-and-sizing.md`
- Tailwind → Fret (declarative) constraint mapping cheat sheet (common “why does my port look wrong?” causes):
  - `w-full` / `h-full` → `.ui().w_full()` / `.ui().h_full()`
  - `flex-1` (≈ `flex: 1 1 0%`) → `.ui().flex_1()` (tip: pair with `.ui().min_w_0()` for text-heavy rows)
  - `flex-none` → `.ui().flex_none()`
  - `items-stretch` → on flex containers: `ui::h_flex(...).items_stretch()` / `ui::v_flex(...).items_stretch()`
  - `min-w-0` / `min-h-0` → `.ui().min_w_0()` / `.ui().min_h(Px(0.0))`
  - `truncate` / `overflow-hidden` → `.ui().truncate()` / `.ui().overflow_hidden()`
  - Rule of thumb: Fret does not implicitly “stretch” children; if a subtree should behave like a block-level element, make it explicit (`w_full`, `items_stretch`, `flex_1`, `min_w_0`).
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

### Widget state surfaces (don’t pay shared-state cost by accident)

- For small view-owned collections, prefer plain local snapshots plus typed actions when the widget
  surface allows it.
- For text widgets with model-backed internals, use the narrow bridge (`Input::new(&local_text)`,
  `Textarea::new(&local_text)`) instead of widening to a generic `IntoModel<T>` story.
- Use explicit `Model<T>` when state is intentionally shared, externally synchronized, or
  runtime-owned.
- If app code only introduces per-row `Model<T>` because a shadcn widget contract demands it,
  escalate to `fret-shadcn-source-alignment` before adding more helper sugar.
- Reference: `references/mind-models/mm-widget-state-surfaces.md`

### Virtualized lists (stable identity is non-negotiable)

- Use keyed virtualization; keys must come from the model (never the row index).
- Prefer fixed row heights when possible (editor UIs).

### Scheduling/animation (don’t leak continuous frames)

- Tie continuous frames leases to element lifetime (store in element-local state).
- Prefer runner-owned timers/effects (deterministic and diagnosable).
- Run the polish checklist on one screen at a time.
- Keep high-impact tweaks token-driven where possible.
- Avoid one-off per-widget magic numbers unless you are proving a recipe outcome first.
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
- Crate/layer selection: `docs/crate-usage-guide.md`
- Shadcn authoring golden path: `docs/shadcn-declarative-progress.md`
- Style generation: `.agents/skills/fret-app-ui-builder/scripts/stylegen.py`
- Style catalog: `.agents/skills/fret-app-ui-builder/references/style_catalog.json`
- Recipes + mind models: `.agents/skills/fret-app-ui-builder/references/`
- Design direction mind model: `.agents/skills/fret-app-ui-builder/references/mind-models/mm-design-direction.md`
- Widget state surface mind model: `.agents/skills/fret-app-ui-builder/references/mind-models/mm-widget-state-surfaces.md`
- UI Gallery exemplar + evidence note: `.agents/skills/fret-shadcn-source-alignment/references/ui-gallery-exemplar-and-evidence.md`
- UI Gallery authoring gates: `apps/fret-ui-gallery/src/lib.rs`
- UI Gallery snippet exemplars: `apps/fret-ui-gallery/src/ui/snippets/`
- Polish pass rules: `.agents/skills/fret-app-ui-builder/references/polish/polish-pass.md`
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
- Accepting per-row `Model<T>` boilerplate in a small local list as “normal app code” when the real issue may be widget contract drift.

## Troubleshooting

- Symptom: visual tweaks keep regressing.
  - Fix: push changes into tokens/recipes; avoid one-off per-widget overrides.
- Symptom: you cannot script the new UI reliably.
  - Fix: add `test_id` and use `fret-diag-workflow` to lock the flow with a script.
- Symptom: a simple todo-like list explodes into per-row models just to keep checkbox/switch/toggle rows.
  - Fix: re-run `references/mind-models/mm-widget-state-surfaces.md`; if the widget surface is the blocker, escalate to `fret-shadcn-source-alignment` instead of adding more app-level helpers.

## Related skills

- `fret-external-app-mode`
- `fret-skills-playbook`
- `fret-diag-workflow`
- `fret-ui-review`
- `fret-shadcn-source-alignment`
