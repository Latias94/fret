# Polish pass checklist (rule-based)

Goal: turn a “working UI” into a **cohesive, usable** UI without picking a bespoke aesthetic.

Apply this to **one surface at a time** (settings form, dashboard, table view, editor shell). Use the `rule_id`s
to report findings from `fret-ui-review`.

## Preconditions (don’t polish too early)

- The surface is already functionally correct (navigation works; no major focus bugs).
- A baseline theme is applied (New York v4 + small overrides).
- Interactive nodes have stable `test_id` where it matters.

## Rule set (high leverage)

### `polish.hierarchy.text_scale`

- Use a **small typographic scale** (usually 3 sizes):
  - title (screen/section), body, helper/caption
- Make secondary text visually secondary (use theme “muted” tokens, not ad-hoc gray).
- Avoid mixing many font weights; prefer 2 (regular + semibold).

Fret mapping hints:

- Prefer theme-driven text sizing/line-height tokens (see `references/theme/token-groups.md`).
- Prefer `ColorRef`/theme keys over raw hex/rgba.

Regression gate:

- One screenshot bundle of the surface (so typography drift is visible).

### `polish.hierarchy.spacing_rhythm`

- Use **one spacing rhythm** (consistent gaps/padding).
- Avoid “almost the same but different” padding between adjacent cards/panels.
- Align baselines: labels, inputs, buttons in a row should share a baseline.

Fret mapping hints:

- Drive padding/gaps via theme metrics and `Space` tokens; avoid repeated `Px(...)` constants.

Regression gate:

- None required by default; add if spacing bugs have regressed before.

### `polish.surface.contrast_and_layers`

- Ensure clear surface separation:
  - background vs panel vs overlay
- Prefer either border or shadow as the primary separator for a surface class (not both everywhere).
- Use elevation only where it communicates layering (dialogs, menus, hovercards).

Fret mapping hints:

- Use theme shadow tokens and border colors; keep them centralized in `ThemeConfig` overrides.

Regression gate:

- Capture bundle after opening an overlay stack (popover → submenu, or dialog → nested popover).

### `polish.states.interactive_consistency`

- Every interactive control has consistent states:
  - hover, pressed, disabled, focused, error (where applicable)
- Disabled is not “just opacity”: ensure contrast and affordance are still clear.
- Pressed feedback should be immediate (no hidden animation delay).

Fret mapping hints:

- Keep state styling in recipes/components (not app screens) so it stays consistent.

Regression gate:

- A diag script that exercises: open → interact → dismiss for one overlay-heavy component.

### `polish.focus.focus_visible_and_clip`

- Focus ring is visible everywhere keyboard users can reach.
- Rings are not clipped by container overflow (only clip inside chrome when intended).
- Tab order is predictable (no surprise jumps into offscreen/hidden nodes).

Fret mapping hints:

- Tune ring width/offset via theme metrics first (`references/theme/token-groups.md`).
- Avoid `overflow: clip` on pressable roots.

Regression gate:

- A diag script that navigates by keyboard and asserts focus moves to expected `test_id` targets.

### `polish.forms.labels_help_errors`

- Forms use one consistent layout:
  - label position, helper text placement, error message placement
- Errors are specific and actionable; error color is from theme and meets contrast.
- Inputs align to a consistent height (density-driven).

Fret mapping hints:

- Use component size tokens (input/button heights) instead of per-form magic numbers.

Regression gate:

- A diag script that triggers one validation error and captures a bundle.

### `polish.tables.rows_density_and_scan`

- Tables/lists are scannable:
  - consistent row height, predictable separators, aligned text
- Use subtle zebra/hover only if it improves scanning; avoid visual noise.
- Column alignment: numbers right, text left, actions aligned and easy to hit.

Fret mapping hints:

- Row height should come from tokens (`component.table.row_min_h`, `component.list.row_height`).
- If large: prefer virtualization and stable keys.

Regression gate:

- A diag script that scrolls + selects a row and captures a bundle (virtualization regressions show up fast).

### `polish.empty_loading_error_states`

- Every data-driven surface has:
  - empty state (no data)
  - loading state (skeleton/spinner)
  - error state (message + retry)
- Empty state should teach the “first action” (CTA or hint), not just say “No items”.

Fret mapping hints:

- Keep empty/loading/error states as reusable components so they stay consistent across screens.

Regression gate:

- Optional: a screenshot bundle for each state (only if historically flaky).

### `polish.overlays.width_height_and_scroll`

- Overlays have a predictable sizing policy:
  - max height with internal scroll for long content
  - width floors for menus/selects so items don’t wrap awkwardly
- Dismiss works (escape/outside press) and focus returns to the trigger.

Fret mapping hints:

- Keep overlay policy in component recipes; app screens should only wire state + content.

Regression gate:

- A diag script that opens the overlay, scrolls inside it, then dismisses and asserts focus restored.

### `polish.motion.small_and_intentional`

- Motion is used sparingly (open/close, hover emphasis), not everywhere.
- Use short durations (fast UI) and respect reduced motion preferences.

Fret mapping hints:

- Prefer runner-owned scheduling/timers; avoid continuous frames when a one-shot will do.

Regression gate:

- Only if motion is critical to correctness (e.g. hover-intent delays).

## Inspiration (optional reading)

These projects influenced the structure (data-backed rules + checklist), but this checklist is **Fret-specific** and
kept intentionally small:

- UI UX Pro Max skill: `repo-ref/ui-ux-pro-max-skill/`
- Vercel Agent Skills: `repo-ref/agent-skills/skills/web-design-guidelines/SKILL.md`

