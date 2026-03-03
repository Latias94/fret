# Diagnostics Architecture (Fearless Refactor v1) — Demo Policy vs General Engine

Status: Draft (boundary note)

One of the main sources of churn in `crates/fret-diag` is mixing:

- “general diagnostics engine” responsibilities (portable artifacts + deterministic tooling), and
- “demo/app-specific policy” (UI gallery/docking scenario choices, selectors, default env knobs).

This note documents the current split and the desired direction.

## Today: where demo policy lives

Examples of demo-specific or app-specific policy that currently lives inside tooling code:

- UI gallery script policy helpers:
  - `crates/fret-diag/src/diag_policy.rs` (`ui_gallery_*` predicates)
- Builtin suite script mapping + default env injection:
  - `crates/fret-diag/src/diag_suite.rs` (`resolve_builtin_suite_scripts`)
  - `crates/fret-diag/src/diag_suite_scripts.rs` (inputs for builtin suites)

These are valuable, but they are not “framework diagnostics engine” concerns.

## Today: general engine responsibilities

These are “portable tooling engine” pieces we want to keep stable and reusable:

- Protocol types:
  - `crates/fret-diag-protocol/src/lib.rs`
- Artifact reading/writing/packing:
  - `crates/fret-diag/src/artifacts/*`
- Transport seam (FS vs WS):
  - `crates/fret-diag/src/transport/*`
- Indexing and derived views:
  - `crates/fret-diag/src/bundle_index.rs`
  - `crates/fret-diag/src/commands/*` (query/slice/resolve/etc.)

## Direction: keep demo policy pluggable

Short-term (M1):

- Keep demo policy where it is, but isolate it behind explicit seams:
  - `SuiteRegistry` / `SuiteResolver` (suite name → scripts)
  - `CheckRegistry` (check name → implementation)
  - table-driven builtin mapping (already started)

Medium-term (M2):

- Allow ecosystem crates to contribute “policy” cheaply:
  - suites, check presets, and runtime extensions
- Prefer putting UI gallery specific defaults in an ecosystem-owned crate (or data directory)
  and “register” them instead of hard-coding match statements.

Non-goal:

- Moving everything out-of-tree immediately. We want boundaries that make future moves cheap, not
  a large one-shot migration.

