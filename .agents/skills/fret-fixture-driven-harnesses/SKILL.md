---
name: fret-fixture-driven-harnesses
description: "Convert large Rust test/conformance scenario matrices into fixture-driven harnesses (JSON fixtures + thin Rust runner), keeping behavior reviewable and reducing merge-conflict risk during bottom-up refactors. Use for shadcn/Radix conformance, overlay placement matrices, and any ‘god test file’ with repetitive cases."
---

# Fixture-driven harnesses (Fret)

This skill is for turning **large, repetitive Rust test matrices** into **data-driven fixtures** with a thin harness.
It reduces merge conflicts, makes review easier, and keeps scenario intent visible.

## When to use fixtures (and when not to)

Good fits:

- “Conformance tables” (many cases; same runner; different inputs/expected).
- Geometry/layout placement matrices (same algorithm; many points/rects/options).
- Policy matrices (same arbitration rules; many parameter combos).
- Any test file that keeps growing because new behavior is “add another case”.

Keep in Rust instead:

- Highly procedural interactions (multi-frame pointer/IME sequences) where the logic is the test.
- Cases requiring closures/async hooks or bespoke host wiring.
- Tests where compile-time types are the primary safety net (fixtures would weaken intent).

## Inputs to collect (ask the user)

Ask these before extracting fixtures (so you don’t over-fit the schema):

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

1. Pick a single “god test” to extract first (keep the old test while mirroring).
2. Define a fixture schema with `schema_version` and stable `id` per case.
3. Write a thin harness that loads fixtures without `cwd` assumptions.
4. Run: `cargo nextest run -p <crate>` and keep failures case-id-addressable.

## Directory conventions

Prefer one of:

- Unit tests (in-crate): `src/<subsystem>/tests/fixtures/*.json`
- Integration tests: `tests/fixtures/*.json`

Use filenames that remain stable across refactors:

- `overlay_placement_v1.json`
- `tooltip_observers.json`
- `web_vs_fret_layout_cases_v1.json`

## Fixture shape (recommended)

Keep fixtures human-diffable:

- Stable `id` per case (string).
- Avoid derived “pretty names” as primary keys.
- Prefer integers / discrete enums over floats where possible.

Suggested top-level:

```json
{
  "schema_version": 1,
  "cases": [
    { "id": "basic", "input": { ... }, "expected": { ... } }
  ]
}
```

If you need comments, use `notes` fields (JSON has no comments).

## Thin harness pattern (Rust)

Guidelines:

- Keep the harness small (parsing + runner + asserts).
- Keep “case selection” deterministic and discoverable (`id`-based).
- Make the fixture loading robust in `cargo test` and `nextest`:
  - Prefer `include_str!` + `env!("CARGO_MANIFEST_DIR")` to avoid `cwd` dependence.

Minimal pattern:

1. `#[derive(serde::Deserialize)]` fixture structs.
2. `let raw = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/.../fixtures/foo.json"));`
3. `let suite: Suite = serde_json::from_str(raw)?;`
4. `for case in suite.cases { run_case(&case); }`

## Workflow

### Migration steps (safe + incremental)

1. **Extract**: copy the existing Rust matrix into a fixture file (keep the old test temporarily).
2. **Mirror**: write a new harness test that runs the fixture cases and matches existing assertions.
3. **Gate**: run `cargo nextest run -p <crate>` and keep the old test until green and reviewed.
4. **Delete**: remove the old matrix and keep the fixture as the source of truth.
5. **Document evidence**: add 1–3 anchors to the relevant workstream TODO item.

### Reviewability checklist

- Fixture format has a `schema_version`.
- Each case has a stable `id`.
- The harness reports failing case `id` clearly (include `id` in panic/assert context).
- The harness avoids runtime filesystem assumptions (no `current_dir()` reliance).
- Adding a new case does not require touching the harness.

### Gates

- Inner loop: `cargo nextest run -p <crate>`
- Refactor boundary changes: `python3 tools/check_layering.py`
- If fixtures are large and frequently edited: consider splitting into multiple files by subsystem.

## Definition of done (what to leave behind)

- Fixture suite has `schema_version` and stable case `id` keys.
- Harness is thin (parse → run_case → asserts) and does not depend on `cwd`.
- Failing output is case-id-addressable (reviewers can locate the exact case quickly).
- Old “god test” is removed only after the fixture harness is green and reviewed.
- If the fixture suite is part of a contract/parity surface, it is wired into the relevant workstream gate.

## Evidence anchors

- Fixture examples (when present): `ecosystem/fret-ui-shadcn/tests/` and `goldens/`
- Diagnostics scripts (for procedural/state-machine cases): `tools/diag-scripts/`

## Common pitfalls

- Using fixtures for procedural state machines (use `fretboard diag` scripts instead).
- Making fixtures too “clever” (hard to diff; unstable IDs; floats everywhere).
- Letting the harness grow (keep it parsing + run_case + asserts only).

## Related skills

- `fret-diag-workflow` (scripted interactions + bundles)
- `fret-shadcn-source-alignment` (parity work often benefits from fixtures)
- `fret-boundary-checks` (guardrails for refactors that move tests across crates)
