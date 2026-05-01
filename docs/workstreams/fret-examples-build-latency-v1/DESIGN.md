# Fret Examples Build Latency v1

Status: maintenance

## Problem

`fret-examples` is a diagnostics-heavy, cross-demo library. A narrow source-policy check such as
the IMUI sortable table marker currently pays for compiling and linking the whole examples crate,
and every `fret-demo` bin depends on the same monolithic library.

This lane owns the build-latency invariant for examples and demo validation: source-only checks
should stay source-only, and single-demo iteration should not accidentally depend on unrelated demo
families when a cleaner split is available.

## Scope

- Move pure source-marker gates out of `apps/fret-examples/src/lib.rs` when they do not need Rust
  type checking.
- Keep runtime gates for behavior that actually needs the launched app, diagnostics script, or
  compile-time API validation.
- Identify safe crate/profile splits for `fret-examples` and `fret-demo` without changing public
  framework contracts.
- Preserve `cargo check -p fret-examples --lib --jobs 1` as the compatibility gate until the
  examples surface is deliberately split.

## Non-Goals

- No change to IMUI table sorting behavior.
- No removal of launched `fretboard diag` proof where runtime behavior is the point.
- No workspace-wide profile churn without measured evidence.
- No compatibility shim for obsolete demo structure; if a split is needed, delete the bad coupling.

## Assumptions First

- Area: lane status
  - Assumption: this is a new active workstream, not a reopening of the closed sortable table gate.
  - Evidence: `docs/workstreams/imui-table-sortable-diag-gate-v1/WORKSTREAM.json` is closed and the
    build-latency problem is not listed as a dedicated lane in `docs/workstreams/README.md`.
  - Confidence: Confident.
  - Consequence if wrong: this lane could duplicate a hidden active tracker.

- Area: source markers
  - Assumption: many `authoring_surface_policy_tests` checks are pure text assertions and can move
    to Python gates without compiling `fret-examples`.
  - Evidence: `apps/fret-examples/src/lib.rs` includes source files and docs through `include_str!`
    inside `#[cfg(test)]`.
  - Confidence: Confident.
  - Consequence if wrong: a migrated gate could miss a type-level contract and need to stay in Rust.

- Area: demo build latency
  - Assumption: `fret-demo` single-bin iteration pays for the whole `fret-examples` library because
    each bin calls `fret_examples::<demo>::run()`.
  - Evidence: `apps/fret-demo/Cargo.toml` depends on `fret-examples`, and bin entrypoints under
    `apps/fret-demo/src/bin/` dispatch directly into that crate.
  - Confidence: Confident.
  - Consequence if wrong: later crate-split work would need a different target.

- Area: profile policy
  - Assumption: the unconditional `profile.dev.package.fret-examples.incremental = false` is a
    macOS stability workaround that may be too broad for Windows iteration.
  - Evidence: the root `Cargo.toml` comment names macOS incremental link artifacts, while the setting
    is package-wide.
  - Confidence: Likely.
  - Consequence if wrong: enabling incremental locally could reintroduce nondeterministic link
    artifacts.

## First Slice

Migrate the just-added sortable table source-marker gate to a lightweight Python script:

- read `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs`;
- read `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json`;
- normalize whitespace and assert the same markers as the Rust unit test;
- delete the redundant Rust unit test from the monolithic examples crate.

The runtime diagnostics script remains the behavior proof. The Python gate only protects source and
selector drift.
