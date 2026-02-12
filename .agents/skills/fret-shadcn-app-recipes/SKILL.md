---
name: fret-shadcn-app-recipes
description: Build good-looking editor-grade apps with Fret using the shadcn/ui v4-aligned component surface (`fret-ui-shadcn`). Use when generating or refactoring app UI, translating shadcn/Tailwind mental models to Fret (`LayoutRefinement`, `ChromeRefinement`, tokens), or adding stable `test_id` + `fretboard diag` repros to prevent regressions.
---

# Fret shadcn app recipes

This skill is organized as:

- **Mind models** (cross-component rules): reusable concepts that prevent repetitive mistakes.
- **Recipes** (component families / app surfaces): concrete “build this UI” guides that *reference* the mind models.

## When to use

Use this skill when:

- Building app UI using `fret-ui-shadcn` and you want the “golden path” patterns.
- Translating shadcn/Tailwind mental models into Fret (`ui()` patches + tokens).
- You want stable `test_id` + scripted repro gates as part of normal component work.

## Inputs to collect (ask the user)

Ask these so you pick the right recipe and leave gates behind:

- What surface are you building (app screen vs one component family)?
- Which shadcn/Radix component family is closest (select/menu/dialog/combobox/table/sidebar)?
- Constraints: density/spacing, keyboard-first requirements, constrained viewports, virtualization?
- What needs to be stable for automation (which controls need `test_id`)?
- What regression gate do we want: diag script, invariant test, or parity harness case?

Defaults if unclear:

- Start from an existing recipe, add stable `test_id` to interactive affordances, and add one diag script for the state machine.

## Quick start (authoring)

- Prefer `use fret_ui_shadcn::prelude::*;` in app code to stay on the shadcn-aligned golden path.
- If the work is about **parity with upstream shadcn/Radix**, use the sibling skill:
  - `fret-shadcn-source-alignment`
- If the work is about **UI polish / visual hierarchy / “make it look good”**, pair:
  - `fret-ui-ux-guidelines`
  - `fret-design-system-styles`
- If the work is about **debugging / scripted repro / packaging**, use:
  - `fret-diag-workflow`

- If the work includes async data + derived state + typed message flow, pair with `fret-app-architecture-and-effects`.

## Workflow

1. Pick the right layer (mechanism vs policy vs shadcn recipe).
2. Start from an existing recipe/mind model and adapt minimally.
3. Add stable `test_id` targets for anything interactive.
4. Add a `tools/diag-scripts/*.json` repro for state machines (menus/select/dialogs/combobox).
5. Add a small invariant test for the most fragile geometry/semantics.

## Definition of done (what to leave behind)

- The UI is built via an existing recipe/mind model (minimal divergence, no new one-off patterns).
- Interactive affordances have stable `test_id` at the recipe/component layer.
- One regression artifact exists for the behavior you touched:
  - state machine ⇒ diag script + bundle evidence, and/or
  - fragile geometry/semantics ⇒ a deterministic Rust invariant test.
- If parity is the goal, the change is paired with `fret-shadcn-source-alignment` evidence (upstream refs + in-tree gates).

## Mind models

- `references/mind-models/mm-layering.md`: pick the correct layer (`fret-ui` vs `fret-ui-kit` vs `fret-ui-shadcn`).
- `references/mind-models/mm-layout-and-sizing.md`: translate Tailwind “box model” into `LayoutRefinement` + constraints.
- `references/mind-models/mm-theme-and-tokens.md`: translate shadcn tokens into `Theme`/`ColorRef`/`MetricRef` usage.
- `references/mind-models/mm-models-actions-and-commands.md`: structure app state (`Model<T>`) and interactions (commands/hooks).
- `references/mind-models/mm-overlays-and-focus.md`: overlay placement, dismiss, focus trap/restore, constrained viewports.
- `references/mind-models/mm-a11y-and-testid.md`: semantics + stable `test_id` strategy for automation and gates.
- `references/mind-models/mm-diagnostics-and-regression-gates.md`: turn bugs into scripts + bundles + gates.

## Recipes

Component families:

- `references/recipes/components/select.md`: listbox-in-overlay select (value/open models).
- `references/recipes/components/dropdown-menu.md`: button-triggered menu + submenus.
- `references/recipes/components/context-menu.md`: pointer-positioned menu (right click).
- `references/recipes/components/dialog-and-sheet.md`: modal surfaces + focus trap/restore.
- `references/recipes/components/combobox.md`: searchable select (popover + command list + a11y).
- `references/recipes/components/tooltip.md`: hover/focus tooltip (no focus stealing).
- `references/recipes/components/popover.md`: non-modal overlay (open model + clamping).
- `references/recipes/components/hover-card.md`: hover intent card (grace + pointer travel).
- `references/recipes/components/menubar.md`: editor menubar (keyboard nav + command integration).
- `references/recipes/components/navigation-menu.md`: top nav + shared viewport/indicator.
- `references/recipes/components/toast-and-sonner.md`: transient notifications (timing + layering + focus).
- `references/recipes/components/tabs.md`: tablist semantics + keyboard nav.
- `references/recipes/components/resizable.md`: resizable panels (drag capture + constraints).
- `references/recipes/components/scroll-area.md`: scroll surface + scrollbars.
- `references/recipes/components/sidebar.md`: sidebar layout surface (nav + scroll + persistence).
- `references/recipes/components/table.md`: base table primitives (small/medium datasets).
- `references/recipes/components/data-grid.md`: canvas-backed data grid (spreadsheet-scale density).

Index/backlog:

- `references/recipes/INDEX.md`: inventory + what’s missing (prioritized).

App surfaces:

- `references/recipes/apps/app-command-palette.md`: cmdk-style command palette (input + listbox + overlay + a11y).
- `references/recipes/apps/app-settings-form.md`: settings panel (fields, labels, validation affordances, keyboard nav).
  - Overlay-heavy building blocks should follow `references/mind-models/mm-overlays-and-focus.md`.
- `references/recipes/apps/app-data-table.md`: TanStack-aligned data table (virtualized body + headless state).
- `references/recipes/apps/app-docking-workspace.md`: editor docking workspace shell (splits + viewports + drag arbitration).
- `references/recipes/apps/app-outliner-tree.md`: file tree / outliner surface (virtualized tree + stable identity).
- `references/recipes/apps/app-inspector-panel.md`: inspector property list (virtualized rows + editor popovers).

## Regression checklist (default)

For any interactive surface (menus/select/dialogs/forms):

1. Add stable `test_id` targets at the component/recipe layer.
2. Add a `tools/diag-scripts/<scenario>.json` repro (click/keys/wait_until + capture_bundle; screenshot optional).
3. Add a small Rust invariant test for the most fragile geometry/semantics (avoid pixel diffs unless needed).

## Evidence anchors (where to look)

- shadcn recipes: `ecosystem/fret-ui-shadcn/src/`
- kit primitives/policy: `ecosystem/fret-ui-kit/src/primitives/`
- ui builder surface: `ecosystem/fret-ui-kit/src/ui_builder.rs`
- parity harness: `ecosystem/fret-ui-shadcn/tests/`
- diag scripts: `tools/diag-scripts/`

## Common pitfalls

- Mixing “parity work” with “new design work” without any regression protection (add a script or invariant test).
- Putting interaction policy into runtime instead of `fret-ui-kit`/`fret-ui-shadcn`.

## Related skills

- `fret-shadcn-source-alignment` (upstream behavior alignment + tests)
- `fret-diag-workflow` (scripted repros + bundles)
- `fret-ui-ux-guidelines` (make it look good)
