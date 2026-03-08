# Fixture schema and harness note

Use this note when you are defining a fixture format or building the thin Rust runner that consumes it.

## 1) Directory conventions

Prefer one of:

- Unit tests (in-crate): `src/<subsystem>/tests/fixtures/*.json`
- Integration tests: `tests/fixtures/*.json`

Use stable filenames across refactors:

- `overlay_placement_v1.json`
- `tooltip_observers.json`
- `web_vs_fret_layout_cases_v1.json`

## 2) Fixture shape (recommended)

Keep fixtures human-diffable:

- stable `id` per case (string)
- avoid derived “pretty names” as primary keys
- prefer integers / discrete enums over floats where possible

Suggested top-level:

```json
{
  "schema_version": 1,
  "cases": [
    { "id": "basic", "input": { ... }, "expected": { ... } }
  ]
}
```

If you need comments, use `notes` fields.

## 3) Thin harness pattern (Rust)

Guidelines:

- keep the harness small (parsing + runner + asserts)
- keep case selection deterministic and discoverable (`id`-based)
- make fixture loading robust in `cargo test` and `nextest`
- prefer `include_str!` + `env!("CARGO_MANIFEST_DIR")` to avoid `cwd` dependence

Minimal pattern:

1. `#[derive(serde::Deserialize)]` fixture structs
2. `let raw = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/.../fixtures/foo.json"));`
3. `let suite: Suite = serde_json::from_str(raw)?;`
4. `for case in suite.cases { run_case(&case); }`

## 4) Migration steps (safe + incremental)

1. **Extract**: copy the existing Rust matrix into a fixture file and keep the old test temporarily
2. **Mirror**: write a new harness test that runs the fixture cases and matches existing assertions
3. **Gate**: run `cargo nextest run -p <crate>` and keep the old test until green and reviewed
4. **Delete**: remove the old matrix and keep the fixture as the source of truth
5. **Document evidence**: add 1–3 anchors to the relevant workstream TODO item

## 5) Reviewability checklist

- the fixture format has a `schema_version`
- each case has a stable `id`
- the harness reports failing case `id` clearly
- the harness avoids runtime filesystem assumptions
- adding a new case does not require touching the harness

## 6) Gates

- inner loop: `cargo nextest run -p <crate>`
- refactor boundary changes: `python3 tools/check_layering.py`
- if fixtures are large and frequently edited, split them into multiple files by subsystem
