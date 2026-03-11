---
name: fret-ui-review
description: "This skill should be used when the user asks to \"review a Fret UI\", \"polish UX\", \"audit focus/overlays\", or \"check token drift and `test_id` stability\". Provides a framework-aligned audit workflow (tokens, focus-visible, overlays, commands gating) with recommended regression gates and evidence anchors."
---

# Fret UI review (framework-aligned audit)

This skill is the “audit lens” companion to the builder/recipe skills. It is inspired by the style of
guideline-based skills (e.g. Vercel’s web interface checks), but tailored to Fret’s layered architecture.

## When to use

- “Review my Fret UI” / “audit UX” / “polish this screen”.
- You suspect a regression caused by layering, focus, overlay policy, or token drift.
- You want to ensure an app repo uses Fret in a way that stays stable across refactors.

## Choose this vs adjacent skills

- Use this skill when the task is **review/audit** (produce findings + recommended fixes/gates).
- Use `fret-app-ui-builder` when you want to build/refactor a UI via a golden-path workflow.
- Use `fret-diag-workflow` when the primary deliverable is a deterministic repro + gate + bundle.
- If a component behavior seems wrong, treat it as a framework/eco responsibility:
  - leave a minimal `fret-diag-workflow` repro (script + bundle) and file it for the component owners.

## Inputs to collect (ask the user)

- What files or directories should be reviewed (`src/`, a specific module, or one component)?
- Target surface: settings/forms, workspace shell, data table, overlay-heavy flows?
- Platform(s): native/web; keyboard-first expectations?
- Is the review about user-facing behavior only, or also about whether the code teaches the intended Fret authoring surface?
- What is the acceptance criterion: “looks cohesive”, “no focus bugs”, “parity with Radix”, “no perf hitches”?

Defaults if unclear:

- Review the smallest surface that shows the issue and prioritize: authoring-surface drift + tokens + focus-visible + overlays + gating + `test_id`.

## Smallest starting point (one command)

- `rg -n "Px\\(|\\.overflow\\(|test_id|when\\s*:\\s*\\\"|OverlayRequest::" src`

## Quick start

Audit in this priority order:

1. **Authoring surface**: examples/snippets teach the intended facade/import/build pattern.
2. **Theme/tokens**: token-driven spacing/radius/colors (avoid per-component magic numbers).
3. **Focus-visible**: focus ring visible, not clipped; keyboard-first paths work.
4. **Overlays**: dismiss + focus restore rules are in policy layers (`ecosystem/`), not runtime.
5. **Commands/keymaps**: stable `CommandId` + explicit `when` gating (avoid firing in text inputs/IME).
6. **Automation stability**: stable `test_id` targets for interactive affordances.
7. **Regression gates**: at least one script/test for the most fragile interaction.
8. **Polish pass** (style-agnostic): apply `rule_id` checklist from:
   - `.agents/skills/fret-app-ui-builder/references/polish/polish-pass.md`

## Output format (recommended)

Keep findings terse and reviewable (Vercel-style):

- `path:line - category/rule_id - message (what to change + why)`

## Workflow

1) Identify the review scope (files/patterns) and the smallest runnable target.

1.5) If this is a first-party surface, compare the right UI Gallery layers before reviewing styling:

- snippet file = canonical example source
- page file = docs composition around the snippet
- driver/render flow = geometry/test-id/diagnostics ownership

Also verify the intended surface against:

- `docs/crate-usage-guide.md`
- `docs/shadcn-declarative-progress.md`

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
- For layout ownership/size negotiation: geometry assertions or `capture_layout_sidecar`.
- For visual chrome/clipping/focus rings: `capture_screenshot` when a screenshot carries more signal than a prose note.
- For perf-sensitive changes: a small perf probe/baseline.

## Definition of done (what to leave behind)

Minimum deliverables (3-pack): Repro (smallest surface), Gate (script/test/perf), Evidence (anchors + command). See `fret-skills-playbook`.

- Findings are reported as concrete issues with evidence anchors (file paths + key functions).
- Fix recommendations map to the correct layer (mechanism vs policy vs recipe).
- At least one regression artifact is proposed (or added) for the highest-risk issue.

## Evidence anchors

- Shared conventions: `.agents/skills/fret-skills-playbook/SKILL.md`
- Crate/layer selection: `docs/crate-usage-guide.md`
- Shadcn authoring golden path: `docs/shadcn-declarative-progress.md`
- Build playbook (tokens + recipes): `.agents/skills/fret-app-ui-builder/SKILL.md`, `.agents/skills/fret-app-ui-builder/references/`
- Polish pass rules: `.agents/skills/fret-app-ui-builder/references/polish/polish-pass.md`
- Contracts/ADRs: `docs/architecture.md`, `docs/runtime-contract-matrix.md`, `docs/adr/`
- Diag gates: `.agents/skills/fret-diag-workflow/SKILL.md`
- UI Gallery exemplar + evidence note: `.agents/skills/fret-shadcn-source-alignment/references/ui-gallery-exemplar-and-evidence.md`
- UI Gallery authoring gates: `apps/fret-ui-gallery/src/lib.rs`
- UI Gallery snippet exemplars: `apps/fret-ui-gallery/src/ui/snippets/`
- UI Gallery geometry/test-id helpers: `apps/fret-ui-gallery/src/driver/render_flow.rs`

## Examples

- Example: audit focus + keyboard UX
  - User says: "Tab order feels wrong and focus ring is inconsistent."
  - Actions: check focus-visible rules, roving tabindex patterns, and command/keymap integration.
  - Result: a concrete list of issues + recommended gates (scripts/tests).

- Example: overlay correctness review
  - User says: "Menus/tooltips sometimes appear in the wrong place."
  - Actions: verify overlay placement, outside-press dismissal, and viewport constraints.
  - Result: actionable fixes + evidence anchors.

## Common pitfalls

- Reviewing page glue while ignoring the snippet file that actually teaches the public example surface.
- Over-polishing visuals without fixing focus/keyboard paths.
- Fixing policy mismatches by adding runtime knobs.
- Missing `when` gating, causing shortcuts to fire inside text inputs / IME.
- Missing `test_id`, causing scripts to rot immediately.

## Troubleshooting

- Symptom: review findings are hard to prove.
  - Fix: pair the review with a minimal `fretboard diag` script + bundle evidence.
- Symptom: there are too many potential issues.
  - Fix: triage into P0 correctness (focus, dismissal, input) vs P1 polish (spacing, tokens).

## Related skills

- `fret-app-ui-builder`
- `fret-external-app-mode`
- `fret-diag-workflow`
  - Framework/eco authors only: `fret-shadcn-source-alignment`
