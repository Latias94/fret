---
name: fret-ui-review
description: "Review Fret app/UI code for framework-aligned UI/UX correctness: tokens, focus-visible, overlays, commands gating, `test_id` stability, and regression gates. Use when asked to review a Fret UI, polish UX, or audit for common pitfalls."
---

# Fret UI review (framework-aligned audit)

This skill is the “audit lens” companion to the builder/recipe skills. It is inspired by the style of
guideline-based skills (e.g. Vercel’s web interface checks), but tailored to Fret’s layered architecture.

## When to use

- “Review my Fret UI” / “audit UX” / “polish this screen”.
- You suspect a regression caused by layering, focus, overlay policy, or token drift.
- You want to ensure an app repo uses Fret in a way that stays stable across refactors.

## Inputs to collect (ask the user)

- What files or directories should be reviewed (`src/`, a specific module, or one component)?
- Target surface: settings/forms, workspace shell, data table, overlay-heavy flows?
- Platform(s): native/web; keyboard-first expectations?
- What is the acceptance criterion: “looks cohesive”, “no focus bugs”, “parity with Radix”, “no perf hitches”?

Defaults if unclear:

- Review the smallest surface that shows the issue and prioritize: tokens + focus-visible + overlays + gating + `test_id`.

## Smallest starting point (one command)

- `rg -n "Px\\(|\\.overflow\\(|test_id|when\\s*:\\s*\\\"|OverlayRequest::" src`

## Quick start

Audit in this priority order:

1. **Theme/tokens**: token-driven spacing/radius/colors (avoid per-component magic numbers).
2. **Focus-visible**: focus ring visible, not clipped; keyboard-first paths work.
3. **Overlays**: dismiss + focus restore rules are in policy layers (`ecosystem/`), not runtime.
4. **Commands/keymaps**: stable `CommandId` + explicit `when` gating (avoid firing in text inputs/IME).
5. **Automation stability**: stable `test_id` targets for interactive affordances.
6. **Regression gates**: at least one script/test for the most fragile interaction.

## Workflow

1) Identify the review scope (files/patterns) and the smallest runnable target.

2) Check layering alignment first:

- mechanism/contract surfaces belong in `crates/*`
- interaction policy and recipes belong in `ecosystem/*`

3) Run a “token drift” scan:

- Prefer tokens (`Space`/`Radius`/`MetricRef`/`ColorRef`) and `UiBuilder` patches.
- Flag large or repeated `Px(...)` usage unless justified.

4) Check focus-visible and overflow:

- Focus rings must remain visible (avoid clipping at pressable/root level).
- Confirm overlay focus trap/restore semantics are deterministic.

5) Check commands and gating:

- Global shortcuts should gate on focus/composition state (`focus.is_text_input == false` unless intentionally editing).

6) Ensure automation stability:

- Add stable `test_id` to interactive affordances that must be gated by diag scripts.
- Avoid selector strategies based on geometry/pixels.

7) Leave a regression artifact:

- For state machines: a `tools/diag-scripts/*.json` script (schema v2 preferred) + `capture_bundle`.
- For deterministic logic: unit/integration tests.
- For perf-sensitive changes: a small perf probe/baseline.

## Definition of done (what to leave behind)

Minimum deliverables (3-pack): Repro (smallest surface), Gate (script/test/perf), Evidence (anchors + command). See `fret-skills-playbook`.

- Findings are reported as concrete issues with evidence anchors (file paths + key functions).
- Fix recommendations map to the correct layer (mechanism vs policy vs recipe).
- At least one regression artifact is proposed (or added) for the highest-risk issue.

## Evidence anchors

- Shared conventions: `.agents/skills/fret-skills-playbook/SKILL.md`
- UX hierarchy: `.agents/skills/fret-ui-ux-guidelines/SKILL.md`
- Tokens/styles: `.agents/skills/fret-design-system-styles/SKILL.md`, `.agents/skills/fret-layout-and-style/SKILL.md`
- Overlays: `.agents/skills/fret-overlays-and-focus/SKILL.md`
- Commands: `.agents/skills/fret-commands-and-keymap/SKILL.md`
- Diag gates: `.agents/skills/fret-diag-workflow/SKILL.md`

## Common pitfalls

- Over-polishing visuals without fixing focus/keyboard paths.
- Fixing policy mismatches by adding runtime knobs.
- Missing `when` gating, causing shortcuts to fire inside text inputs / IME.
- Missing `test_id`, causing scripts to rot immediately.

## Related skills

- `fret-app-ui-builder`
- `fret-external-app-mode`
- `fret-diag-workflow`
- `fret-shadcn-source-alignment`
