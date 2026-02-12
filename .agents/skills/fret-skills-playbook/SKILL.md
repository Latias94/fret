---
name: fret-skills-playbook
description: "Shared playbook for Fret agent skills: layering decisions (mechanism vs policy), regression gate types, `test_id` and diag script conventions, and evidence discipline. Use when writing/updating skills or when you want a consistent ‘what to leave behind’ checklist."
---

# Fret skills playbook (shared conventions)

This skill is the shared “glue” for the rest of the Fret skill set: it defines conventions that keep
agent work **reviewable**, **reproducible**, and **architecture-aligned**.

## When to use

- You are writing or updating a skill.
- You want consistent outputs across different contributors/agents.
- You are unsure what regression gate + evidence artifacts to leave behind.

## Inputs to collect (ask the user)

- What is the user-facing invariant (correctness, UX, parity, perf)?
- What is the smallest runnable target (demo/gallery/script) that shows it?
- Which layer should own the change (mechanism vs policy vs recipe)?
- What artifacts must be produced (gate + evidence)?

Defaults if unclear:

- Start from the smallest runnable demo and leave a regression artifact + evidence anchors.

## Smallest starting point (one command)

- `python3 .agents/skills/fret_skills.py validate --strict`

## Quick start

- Use the “deliverables 3-pack” for any non-trivial change:
  - Repro (smallest target or script)
  - Gate (test/script/perf)
  - Evidence (anchors + command)

## Workflow

### 1) Layering decision (non-negotiable)

Use this rule of thumb:

- `crates/*`: mechanisms and hard-to-change contracts (routing, focus primitives, overlay roots, layout/semantics).
- `ecosystem/*`: policy + composition + recipes (dismiss/focus restore rules, roving/typeahead, shadcn recipes).

If the change is “interaction policy” (dismiss rules, focus restore, keyboard nav rules), it almost never belongs in
`crates/fret-ui`.

### 2) The deliverables 3-pack (Repro + Gate + Evidence)

Every non-trivial change should leave these three deliverables:

- **Repro**: a smallest runnable target (demo/gallery page) or a `tools/diag-scripts/*.json` script.
- **Gate**: at least one regression gate:
  - unit/integration test for deterministic logic, and/or
  - `fretboard diag` script for event sequences/state machines, and/or
  - perf gate/baseline when perf is the goal.
- **Evidence**: 1–3 evidence anchors (file paths + key functions/tests/scripts) so reviewers can verify quickly.

### 3) `test_id` conventions (automation stability)

Goal: scripts should select **intent-level** targets, not pixel coordinates.

- Put `test_id` at the recipe/component layer (often `ecosystem/fret-ui-shadcn`) so it survives layout refactors.
- Use stable, namespaced ids (examples):
  - `gallery.command_palette.trigger`
  - `select.trigger`
  - `docking.tab_bar.drag_anchor`
- Avoid using list indices as ids; use model ids.

### 4) Diag script conventions (reviewable and gate-friendly)

- Prefer schema v2 for new scripts.
- Prefer selectors by `test_id`.
- Name scripts so they can be used as a gate label:
  - `ui-gallery-<surface>-<behavior>-<expectation>.json`
  - `docking-<scenario>-<expectation>.json`
- Keep scripts minimal: one scenario, one or two assertions, at least one `capture_bundle`.

### 5) Evidence discipline (make it reversible)

When you fix a tricky issue, record:

- exact command(s) used,
- output dir / bundle path(s),
- the smallest script/test added,
- the conclusion (“what changed” + “why it’s correct”).

## Definition of done (what to leave behind)

Minimum deliverables (3-pack):

- Repro: smallest runnable target or diag script.
- Gate: test/script/perf gate that fails before and passes after.
- Evidence: 1–3 anchors (paths/functions/tests) and a copy/pasteable command.

## Evidence anchors

- Layering and contracts: `docs/architecture.md`, `docs/runtime-contract-matrix.md`
- Diag scripts and workflows: `tools/diag-scripts/`, `.agents/skills/fret-diag-workflow/SKILL.md`
- Perf gates and baselines: `.agents/skills/fret-perf-workflow/SKILL.md`, `tools/perf/`, `docs/workstreams/perf-baselines/`

## Common pitfalls

- Fixing policy mismatches by adding runtime knobs in `crates/fret-ui`.
- Leaving no gate behind (“works on my machine” regressions).
- Unstable selectors (`test_id` missing/duplicated), leading to flaky scripts.
- Writing long narratives instead of a small reproducible repro + gate + evidence anchors.

## Related skills

- `fret-repo-orientation`
- `fret-diag-workflow`
- `fret-perf-workflow`
- `fret-shadcn-source-alignment`
