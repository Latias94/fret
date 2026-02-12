# Polish pass checklist (rule-based, style-agnostic)

Goal: turn a “working UI” into a **cohesive, usable** UI without picking a bespoke aesthetic.

This is intentionally generic: it does not assume your app has “settings”, “workspace shell”, or “data tables”.

Use the `rule_id`s below to report findings from `fret-ui-review`.

## Preconditions (don’t polish too early)

- The surface is already functionally correct (navigation works; no major focus bugs).
- A baseline theme is applied (New York v4 + small overrides).
- Interactive nodes have stable `test_id` where it matters.

If a component behavior seems wrong, do not chase “upstream parity” as an app author. Leave a minimal diag repro
(script + bundle) and hand it to the framework/eco owners.

## P0 (must-do) rules

Apply these first. Each rule is either **always** applicable or **applicable when the surface uses that capability**
(overlays, data loading, etc.).

| Priority | `rule_id` | Applies when | Outcome |
| --- | --- | --- | --- |
| P0 | `polish.hierarchy.text_scale` | always | clear text hierarchy |
| P0 | `polish.hierarchy.spacing_rhythm` | always | consistent spacing rhythm |
| P0 | `polish.surface.contrast_and_layers` | always | readable surfaces + stable layering cues |
| P0 | `polish.states.interactive_consistency` | any interactive UI | consistent hover/pressed/disabled/focus states |
| P0 | `polish.focus.focus_visible_and_clip` | any keyboard path | focus-visible works and is not clipped |
| P0 | `polish.overlays.width_height_and_scroll` | if the surface uses overlays | overlays size/scroll/dismiss predictably |
| P0 | `polish.empty_loading_error_states` | if data can be empty/loading/error | users are never “stuck” |
| P0 | `polish.motion.small_and_intentional` | if you add motion | motion supports intent, not noise |

### `polish.hierarchy.text_scale`

Why:

- Users should instantly see “what matters” (title vs body vs helper).

Check:

- Use a small typographic scale (usually 3 sizes): title, body, helper/caption.
- Secondary text uses theme “muted” tokens (not ad-hoc gray).
- Avoid many font weights; prefer 2 (regular + semibold).

Fix:

- Prefer theme-driven text sizing/line-height tokens (see `references/theme/token-groups.md`).
- Prefer `ColorRef`/theme keys over raw hex/rgba.

Gate (optional):

- One screenshot bundle of the surface (typography drift is high-signal).

### `polish.hierarchy.spacing_rhythm`

Why:

- Inconsistent gaps/padding reads as “unpolished”, even if the UI works.

Check:

- Use one spacing rhythm (consistent gaps/padding within a surface class).
- Avoid “almost the same but different” padding between adjacent panels/cards.
- Align baselines: items in a row should share a baseline.

Fix:

- Drive padding/gaps via theme metrics and `Space` tokens; avoid repeated `Px(...)` constants.

Gate:

- None required by default.

### `polish.surface.contrast_and_layers`

Why:

- Users need to understand surface hierarchy (base vs panel vs overlay) at a glance.

Check:

- Clear separation between background vs panel vs overlay.
- Prefer one primary separator per surface class (border or shadow), not both everywhere.
- Elevation is reserved for “layering meaning” (dialogs, menus, hovercards).

Fix:

- Use theme shadow tokens and border colors; keep overrides centralized in `ThemeConfig`.

Gate (recommended when overlays exist):

- Capture a bundle after opening an overlay stack (popover → submenu, dialog → nested popover).

### `polish.states.interactive_consistency`

Why:

- Inconsistent states break affordance and make UI feel “random”.

Check:

- Every interactive control has consistent states: hover, pressed, disabled, focused (and error if applicable).
- Disabled is not “just opacity”: ensure contrast and affordance remain clear.
- Pressed feedback is immediate (no hidden delay).

Fix:

- Keep state styling in recipes/components (not app screens) so it stays consistent.

Gate (recommended for state machines):

- A diag script that exercises: open → interact → dismiss (or click → toggle → undo) for one fragile interaction.

### `polish.focus.focus_visible_and_clip`

Why:

- Keyboard-first users (and power users) rely on focus-visible for confidence.

Check:

- Focus ring is visible everywhere keyboard users can reach.
- Rings are not clipped by container overflow (clip only inside chrome when intended).
- Tab order is predictable (no surprise jumps into hidden/offscreen nodes).

Fix:

- Tune ring width/offset via theme metrics first (`references/theme/token-groups.md`).
- Avoid `overflow: clip` on pressable roots.

Gate (recommended):

- A diag script that navigates by keyboard and asserts focus moves to expected `test_id` targets.

### `polish.overlays.width_height_and_scroll`

Why:

- Menus/dialogs/popovers are where UX breaks first (scroll, sizing, dismissal, focus restore).

Check:

- Max height with internal scroll for long content.
- Width floors for menus/selects so items don’t wrap awkwardly.
- Dismiss works (escape/outside press) and focus returns to the trigger.

Fix:

- Keep overlay policy in component recipes; app screens should only wire state + content.

Gate (recommended):

- A diag script that opens the overlay, scrolls inside it, then dismisses and asserts focus restored.

### `polish.empty_loading_error_states`

Why:

- Data-driven screens feel broken without a “next step” when empty/error/loading.

Check:

- If the surface can be empty/loading/error, it has explicit states for each.
- Empty state teaches the “first action” (CTA or hint), not just “No items”.
- Error state includes a retry path when reasonable.

Fix:

- Keep empty/loading/error states as reusable components so they stay consistent.

Gate (optional):

- Screenshot bundles for empty/error states (only if historically flaky).

### `polish.motion.small_and_intentional`

Why:

- Subtle motion improves clarity; excessive motion reads as “theme noise”.

Check:

- Motion is sparse (open/close, hover emphasis), not everywhere.
- Durations are short (fast UI) and reduced-motion is respected.

Fix:

- Prefer runner-owned scheduling/timers; avoid continuous frames when a one-shot will do.

Gate:

- Only if motion is critical to correctness (e.g. hover-intent delays).

## Optional modules (apply only when you have them)

These are still “universal” patterns, but they depend on the surface.

### If you have forms: `polish.forms.labels_help_errors`

Check:

- One consistent layout: label position, helper placement, error placement.
- Errors are specific and actionable; error color is from theme and meets contrast.
- Inputs align to a consistent height (density-driven).

Fix:

- Use component size tokens (input/button heights) instead of per-form magic numbers.

Gate:

- A diag script that triggers one validation error and captures a bundle.

### If you have dense lists/tables: `polish.tables.rows_density_and_scan`

Check:

- Consistent row height, predictable separators, aligned text.
- Zebra/hover only if it improves scanning; avoid visual noise.
- Column alignment: numbers right, text left, actions aligned and easy to hit.

Fix:

- Row height from tokens (`component.table.row_min_h`, `component.list.row_height`).
- If large: prefer virtualization and stable keys.

Gate:

- A diag script that scrolls + selects a row and captures a bundle (virtualization regressions show up fast).

## Inspiration (optional reading)

These projects influenced the “rule-based checklist” shape, but this polish pass is intentionally **Fret-specific**
and kept small:

- UI UX Pro Max skill (search on GitHub): `nextlevelbuilder/ui-ux-pro-max-skill`
- Vercel Agent Skills (guideline-style review): `vercel-labs/agent-skills` (skill: `web-design-guidelines`)
