# Fret Examples Build Latency v1 - Milestones

Status: active

## M0 - Baseline And First Source Gate

Exit criteria:

- The lane records current assumptions and gates.
- One representative pure source-marker check runs without compiling `fret-examples`.
- The deleted Rust unit test has equivalent source coverage elsewhere.

## M1 - Source-Policy Test Migration Plan

Status: complete

Exit criteria:

- Remaining source-marker tests in `apps/fret-examples/src/lib.rs` are grouped by owner surface.
- Tests that only need text scanning have a Python gate migration plan.
- Tests that need Rust type checking remain in `fret-examples` with an explicit reason.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M1_SOURCE_POLICY_AUDIT_2026-04-29.md`
- `tools/gate_imui_facade_teaching_source.py`
- `tools/gate_table_source_policy.py`
- `tools/gate_examples_source_tree_policy.py`

## M2 - Demo Build Split Decision

Status: complete

Exit criteria:

- Single-demo build coupling is measured on at least one representative IMUI demo.
- The lane chooses between feature-family split, separate examples crates, or direct demo-local bins.
- The chosen split has a small compatibility gate before broad migration.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M2_DEMO_BUILD_SPLIT_DECISION_2026-04-29.md`
- `apps/fret-examples-imui/Cargo.toml`
- `tools/gate_fret_examples_imui_split_source.py`

## M3 - Profile Policy Decision

Status: complete

Exit criteria:

- The macOS incremental-link workaround is either kept global with evidence or narrowed through a
  documented developer profile path.
- Windows iteration guidance is updated if `dev-fast` becomes the recommended local path.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M3_PROFILE_POLICY_DECISION_2026-04-29.md`
- `Cargo.toml`
