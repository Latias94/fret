---
name: fret-fixture-driven-harnesses
description: 'This skill should be used when the user asks to "convert a large test matrix", "reduce god test files", "add fixture-driven conformance", or "make repetitive scenarios reviewable". Provides a workflow for JSON fixtures + thin Rust runners (shadcn/Radix conformance, overlay placement matrices) to reduce merge conflicts and keep behavior reviewable.'
---

# Fixture-driven harnesses (Fret)

This skill is for turning **large, repetitive Rust test matrices** into **data-driven fixtures** with a thin harness.
It reduces merge conflicts, makes review easier, and keeps scenario intent visible.

## When to use fixtures (and when not to)

Good fits:

- conformance tables (many cases, same runner, different inputs/expected)
- geometry/layout placement matrices
- policy matrices using the same arbitration rules
- any test file that keeps growing because new behavior means “add another case”

Keep in Rust instead:

- highly procedural interactions where the logic is the test
- cases requiring closures/async hooks or bespoke host wiring
- tests where compile-time types are the primary safety net

## Inputs to collect (ask the user)

- What is the repeated dimension (inputs, expected outputs, environment knobs)?
- Does each case share the same runner/harness, or do you need multiple harnesses?
- What must be stable IDs (case id, scenario id, golden id)?
- Do you need deterministic geometry (avoid floats) or will tolerances be required?
- How should failures be reported (case-id-addressable, diff-friendly output)?

Defaults if unclear:

- Start with a single fixture file with `schema_version` + stable `cases[].id`, and a thin harness that prints failing ids.

## Smallest starting point (one command)

- `cargo nextest run -p fret-ui-shadcn`

## Quick start

1. Pick a single “god test” to extract first.
2. Read the fixture reference note before defining the schema.
3. Mirror the old test with a thin harness before deleting anything.
4. Keep failures case-id-addressable and review-friendly.

## Workflow

### 0) Read the reference note first

Use this note to keep the main skill lean:

- `.agents/skills/fret-fixture-driven-harnesses/references/fixture-schema-and-harness.md`

### 1) Decide whether fixtures are the right tool

Use fixtures when the runner stays the same and only the data changes.

If the behavior is a procedural multi-frame interaction, prefer `fret-diag-workflow` instead.

### 2) Extract one matrix incrementally

- copy the existing Rust matrix into a fixture file
- keep the old test temporarily
- mirror the assertions in a thin harness

### 3) Keep the harness thin and deterministic

The harness should mostly be:

- parse fixture,
- iterate cases,
- call `run_case`,
- assert with stable case ids in failure output.

### 4) Gate first, delete second

Only remove the old “god test” after:

- the fixture harness is green,
- failures are clearly case-id-addressable,
- reviewers can inspect the fixture diff without decoding harness internals.

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro (fixture suite), Gate (nextest), Evidence (case-id failures). See `fret-skills-playbook`.
- Fixture suite has `schema_version` and stable case `id` keys.
- Harness is thin (parse → `run_case` → asserts) and does not depend on `cwd`.
- Failing output is case-id-addressable.
- Old matrix tests are removed only after the fixture harness is green and reviewed.

## Evidence anchors

- Fixture-driven test examples:
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`
  - `ecosystem/fret-ui-shadcn/tests/data_grid_layout.rs`
  - `ecosystem/fret-ui-shadcn/tests/resizable_panel_group_layout.rs`
- Goldens (overview + conventions): `goldens/README.md`
- Layering boundaries: `tools/check_layering.py`
- Diagnostics scripts for procedural cases: `tools/diag-scripts/ui-gallery-intro-idle-screenshot.json`
- This skill’s reference:
  - `.agents/skills/fret-fixture-driven-harnesses/references/fixture-schema-and-harness.md`

## Examples

- Example: replace a growing matrix test with fixtures
  - User says: "This test file keeps growing—every change adds another case."
  - Actions: extract cases into `fixtures/*.json`, keep a thin harness, and make failures case-id-addressable.
  - Result: smaller diffs, fewer conflicts, clearer intent.

## Common pitfalls

- Using fixtures for procedural state machines.
- Making fixtures too clever (unstable IDs, floats everywhere, hard-to-diff shape).
- Letting the harness grow beyond parsing + `run_case` + asserts.

## Troubleshooting

- Symptom: fixtures depend on `cwd` and fail in CI.
  - Fix: load with `include_str!` + `env!("CARGO_MANIFEST_DIR")`.
- Symptom: fixtures are hard to review.
  - Fix: keep stable `id`s, prefer integers/enums, and split large suites by subsystem.

## Related skills

- `fret-diag-workflow`
- `fret-shadcn-source-alignment`
- `fret-boundary-checks`
