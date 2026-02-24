---
title: Diag Fearless Refactor v1 (Milestones)
status: draft
date: 2026-02-24
scope: diagnostics, automation, bundle-schema, refactor
---

# Diag Fearless Refactor v1 (Milestones)

This file tracks milestones for `docs/workstreams/diag-fearless-refactor-v1.md`.

Conventions:

- keep changes additive and compatibility-first (fearless refactor prerequisite),
- prefer bounded artifacts (`bundle.schema2.json` / index / slice / packet) over grepping full `bundle.json`,
- keep a fast gate green after each milestone (recommended: `cargo check -p fret-ui-gallery`).

## Milestones

### M0: First extraction (filesystem triggers)

- [x] Split filesystem-trigger concerns out of `ui_diagnostics.rs`.

Definition of done:

- `UiDiagnosticsService` can still run with filesystem triggers enabled.
- The parent module can call the extracted methods without widening public API beyond `pub(super)`.

### M1: Bundle export modularization

- [x] Extract bundle schema selection + JSON writing into a cohesive module (`bundle_dump.rs`).
- [x] Make schema v2 + semantics-mode defaults explicit and documented.

Definition of done:

- Bundle export logic no longer spans the core `UiDiagnosticsService` file.
- Manual dumps and script dumps produce the same artifact layout with only schema/mode differences.

### M2: Script runner modularization

- [ ] Extract the scripted interaction runner (v1/v2 parsing, step loop, evidence capture) into its own module.
  - [x] Extract runner helpers (`script_runner.rs`).
  - [x] Extract step handlers into dedicated modules (pointer sessions, drag playback, menu, scroll, visibility, etc.).

Definition of done:

- The runner can be reasoned about and tested without navigating the entire diagnostics service file.

### M3: Inspect/pick modularization

- [ ] Extract inspect/pick trigger polling + state into modules.
- [ ] Keep transport concerns isolated (filesystem vs DevTools WS).

Progress:

- [x] Filesystem trigger polling extracted (`poll_pick_trigger`, `poll_inspect_trigger`):
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`
- [x] Inspect-mode state + shortcuts extracted:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/inspect.rs`
- [x] Pick flow extracted (pending resolution + result writing):
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/pick_flow.rs`

Definition of done:

- Inspect/pick code paths are independently editable without touching bundle/schema code.

### M4: Plan 1 closure for AI loops (schema2-first)

- [ ] Ensure “AI packet” is the default shareable artifact path for triage.
- [ ] Ensure sidecars (`bundle.meta.json`, `bundle.index.json`, `test_ids.index.json`, `frames.index.json`) are consistently available
  in pack/repro flows.
  - [x] Runtime writes canonical sidecars on native dumps.
  - [x] Runtime `bundle.index.json` includes a bounded `test_id` bloom (`test_id_bloom_hex`) on tail snapshots for fast `--test-id` triage.
  - [x] Runtime `bundle.index.json` includes bounded `semantics_blooms` keyed by `(window, semantics_fingerprint, semantics_source)` to support `--test-id` triage beyond the tail snapshots.
  - [x] Runtime `bundle.index.json` may include additive script step markers (`script.steps`) when `script.result.json` is present.
  - [x] `diag pack --include-all` includes sidecars under `_root/` (even when the bundle dir is relocated).
  - [x] `diag repro` multi-pack includes the same sidecars under each script prefix’s `_root/`.

Definition of done:

- A typical scripted failure can be debugged from an AI packet without opening the full `bundle.json`.
- Tooling accepts `bundle.schema2.json` in all common “bundle path” entry points (pack/repro/suite/triage/stats).

### M4.1: Remove `bundle.json`-only assumptions in tooling

- [ ] Ensure post-run checks, suite runners, and error messages consistently talk about “bundle artifacts” (not `bundle.json`).
- [x] Extract bundle/repro zip packing into a dedicated module (reduce churn in `crates/fret-diag/src/lib.rs`).
  - Evidence: `crates/fret-diag/src/pack_zip.rs` (`pack_bundle_dir_to_zip`, `pack_repro_zip_multi`).
- [x] Extract evidence indexing into a dedicated module (reduce churn in `crates/fret-diag/src/lib.rs`).
  - Evidence: `crates/fret-diag/src/evidence_index.rs` (`write_evidence_index`).
- [x] Add bundle-artifact aliases to repro summary JSON (keep older `*_bundle_json` keys for compatibility).
  - Evidence: `crates/fret-diag/src/diag_repro.rs` (`selected_bundle_artifact`, `packed_bundle_artifact`, `bundle_artifact` in `packed_bundles`).
- [x] Add one regression test that a schema2-only bundle dir is accepted where we claim it is.
  - Evidence: `crates/fret-diag/src/pack_zip.rs` (`pack_bundle_dir_to_zip_accepts_schema2_only`).
- [x] Add a schema2-only packing option for shareable zips.
  - Evidence: `crates/fret-diag/src/lib.rs` (`--pack-schema2-only` / `--schema2-only`).

### M5: Plan 2 prototype (deferred)

- [ ] Prototype a manifest-first chunked bundle layout + compatibility materializer.

Definition of done:

- At least one workflow produces a manifest bundle and can materialize `bundle.json` on demand.
