---
name: fret-skill-evolution
description: "Capture reusable Fret learnings as agent skills: turn fixes into repeatable workflows by writing/refreshing skills, adding invariant tests, and adding `fretboard diag` scripted repro gates. Use after resolving a tricky bug or discovering a new stable pattern."
---

# Fret skill evolution (capture learnings)

## When to use

Use this skill after you:

- Fix a tricky UI bug (interaction state machine, overlays, focus/IME, virtualization).
- Discover a new reusable authoring pattern for components or app architecture.
- Add a new diagnostics script or test gate and want it to be discoverable.

Goal: convert “we figured it out once” into “the agent can do it reliably next time”.

## Quick start

1. Identify the “owner skill” (diag/perf/shadcn/text/etc).
2. Add a short workflow + evidence anchors (keep the main SKILL.md lean).
3. Land at least one regression artifact (test, scripted repro, or parity gate).

## Workflow

1. Pick the right destination:
   - UI repro/gates: `fret-diag-workflow` + `tools/diag-scripts/*.json`
   - shadcn/Radix parity: `fret-shadcn-source-alignment` + targeted tests
   - component authoring gotchas: `fret-component-authoring`
   - perf baselines/gates: `fret-perf-workflow`
2. Update the SKILL.md with the standard headings:
   - `## When to use`, `## Quick start`, `## Workflow`, `## Evidence anchors`, `## Common pitfalls`, `## Related skills`
3. If content grows, move long writeups into `references/` and link from the skill body.
4. Add a gate (at least one):
   - unit/integration test, and/or
   - `tools/diag-scripts/*.json` scripted repro, and/or
   - web-vs-fret parity harness entry.
5. Validate skills locally:
   - `pwsh -File tools/validate_skills.ps1` (use `-Strict` for “no warnings” mode)

## What to capture (pick one)

- **Workflow**: a short, repeatable procedure (“how to reproduce + fix + gate”).
- **Invariant**: a small test that locks the outcome.
- **Repro gate**: a `tools/diag-scripts/*.json` scripted interaction that fails on regressions.

## Where to put it

- Interaction correctness + repros: update `fret-diag-workflow` and add/refresh a script under `tools/diag-scripts/`.
- shadcn/Radix alignment patterns: update `fret-shadcn-source-alignment`.
- Component authoring gotchas: update `fret-component-authoring`.
- Performance workflows: update `fret-perf-workflow`.

If the new knowledge is substantial, create a **new** `fret-*` skill folder under `.agents/skills/`.

## How to write the update (template)

Keep SKILL bodies short. Prefer “just enough workflow” + evidence anchors.

1. Add/refresh:
   - **When to use**
   - **Quick start**
   - **Workflow**
   - **Evidence anchors**
   - **Common pitfalls**
   - **Related skills**
2. Add a regression asset:
   - Unit/integration test (fast) and/or
   - `fretboard diag` script (state machines) and/or
   - golden/parity harness entry (layout/style outcomes)

## Templates (copy/paste)

- Pattern write-up: `references/pattern-template.md`
- Troubleshooting entry: `references/troubleshooting-template.md`
- Gate checklist: `references/gate-template.md`

## Quality bar (do this every time)

- Prefer the **smallest** reproducible target (UI gallery page or demo binary).
- Prefer stable selectors (`test_id` / semantics) over pixel coordinates.
- Land at least one “red-to-green” artifact: test, script, or parity gate.

## Evidence anchors

- Skills root + index: `.agents/skills/`, `.agents/skills/README.md`
- Skill validator: `tools/validate_skills.ps1`
- Diagnostics scripts/gates: `tools/diag-scripts/`, `docs/ui-diagnostics-and-scripted-tests.md`
- Parity harness and goldens (when applicable): `ecosystem/fret-ui-shadcn/tests/`, `goldens/`

## Common pitfalls

- Turning a fix into a long narrative without a repeatable workflow.
- Landing “the fix” without any gate (regressions return as human-only bugs).
- Putting large diffs into SKILL bodies; prefer `references/` for long content.

## Related skills

- `fret-diag-workflow`
- `fret-perf-workflow`
- `fret-shadcn-source-alignment`
