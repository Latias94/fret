---
title: Diagnostics Fearless Refactor v1 (TODO)
status: draft
date: 2026-02-22
scope: diagnostics, automation, tooling, refactor
---

# Diagnostics Fearless Refactor v1 (TODO)

## M1: Make the monolith smaller (safe mechanical moves)

- [x] Extract internal script runner state types into `ecosystem/fret-bootstrap/src/ui_diagnostics/script_types.rs`.
- [x] Move script evidence + trace helpers into `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`.
- [x] Extract `drive_script_for_window` finalization into `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`.
- [x] Extract the per-frame driver (`UiDiagnosticsService::drive_script_for_window`) out of
      `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` into `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`.
  - Keep the public entrypoint signature stable.
- [ ] Define a stable “module boundary” inside `ecosystem/fret-bootstrap/src/ui_diagnostics/`:
  - script execution / state / step handlers,
  - bundle dumping + sidecar writers,
  - DevTools WS bridge wiring.
- [x] Extract bundle dump policy helpers into `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_dump_policy.rs`.
- [x] Keep a regression gate: `cargo check -p fret-ui-gallery` after each extraction step.

## M1b: Make `fret-diag` stats less monolithic (mechanical moves)

- [x] Move UI gallery markdown-editor checks out of `crates/fret-diag/src/stats.rs` into
      `crates/fret-diag/src/stats/ui_gallery_markdown_editor.rs`.
- [x] Move UI gallery code-editor checks out of `crates/fret-diag/src/stats.rs` into
      `crates/fret-diag/src/stats/ui_gallery_code_editor.rs`.

## M1c: Make `fret-diag` CLI subcommands less monolithic (mechanical moves)

- [x] Extract `diag run` command handler out of `crates/fret-diag/src/lib.rs` into `crates/fret-diag/src/diag_run.rs`.
- [x] Extract `diag suite` command handler out of `crates/fret-diag/src/lib.rs` into `crates/fret-diag/src/diag_suite.rs`.
- [x] Extract `diag repeat` command handler out of `crates/fret-diag/src/lib.rs` into `crates/fret-diag/src/diag_repeat.rs`.
- [x] Extract `diag repro` command handler out of `crates/fret-diag/src/lib.rs` into `crates/fret-diag/src/diag_repro.rs`.
- [x] Split `diag repro` helpers into dedicated submodules under `crates/fret-diag/src/diag_repro/` (launch/renderdoc/packing/summary/scripts).
- [x] Extract `diag perf` command handler out of `crates/fret-diag/src/lib.rs` into `crates/fret-diag/src/diag_perf.rs`.
- [x] Extract `diag compare` command handler out of `crates/fret-diag/src/lib.rs` into `crates/fret-diag/src/diag_compare.rs`.
- [x] Extract `diag stats` command handler out of `crates/fret-diag/src/lib.rs` into `crates/fret-diag/src/diag_stats.rs`.
- [x] Extract `diag matrix` command handler out of `crates/fret-diag/src/lib.rs` into `crates/fret-diag/src/diag_matrix.rs`.
- [x] Shrink post-run check call sites by using the `apply_post_run_checks(..., &diag_run::RunChecks, ...)` entrypoint (migrate `diag run` / `diag suite` / `diag repro`).
- [x] Remove redundant `SuiteChecks` type (reuse `diag_run::RunChecks` for suite command wiring).
- [x] Extract post-run checks plumbing out of `crates/fret-diag/src/lib.rs` into `crates/fret-diag/src/post_run_checks.rs`.
- [ ] Continue extracting large subcommands into dedicated modules (keep `lib.rs` as CLI wiring + shared helpers):
  - `diag pack` / `diag ai-packet` follow-ups if they become churn hotspots.

- [ ] Reduce churn in `lib.rs` context assembly:
  - move large check-struct literal assembly into helper fns (so adding a new check is localized),
  - keep `lib.rs` as “arg parsing + dispatch only”.

## M2: Shrink + index artifacts (sidecars over monolithic JSON)

- [x] Define the “minimum useful bundle” contract (what must be in `bundle.json` vs what can be in sidecars).
  - `docs/workstreams/diag-fearless-refactor-v1/minimum-useful-bundle.md`
- [x] Reduce default bundle size by defaulting non-script dumps to `FRET_DIAG_BUNDLE_SEMANTICS_MODE=changed` (script dumps still default to `last`).
- [x] Reduce default bundle size further by defaulting non-script dumps to schema v2 (dedup semantics via `tables.semantics`).
- [x] Cap non-script dump semantics nodes by default (via `FRET_DIAG_BUNDLE_DUMP_MAX_SEMANTICS_NODES`, defaulting to 10,000).
- [x] Optional: export only `test_id` semantics nodes plus their ancestors (via `FRET_DIAG_BUNDLE_DUMP_SEMANTICS_TEST_IDS_ONLY`).
- [x] Apply dump-time semantics policies to schema v2 `tables.semantics` entries (not just inline `debug.semantics`).
- [x] Prune schema v2 semantics tables to only referenced snapshots after applying semantics mode (avoid retaining dropped frames).
- [x] Add query-friendly indexes (sidecars) for tools/agents (implemented in `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_dump.rs`):
  - `bundle.index.json` (snapshot selectors, semantics fingerprints, test-id bloom),
  - `bundle.meta.json` (bundle-level counters + uniqueness summaries),
  - `test_ids.index.json` (test-id catalog / lookup),
  - `frames.index.json` (per-frame lightweight stats + selectors; columnar rows for agent-friendly triage),
  - `script.result.json` + `bundle.index.json.script` (script step → snapshot mapping for fast evidence lookup, script dumps only).
- [x] Make sidecars forward-compatible:
  - versioned schema (`kind` + `schema_version`),
  - additive-only evolution,
  - documented failure behavior when sidecars are missing,
  - CLI-side validation + on-demand regeneration when invalid.
  - `docs/workstreams/diag-fearless-refactor-v1/sidecar-schema-policy.md`
  - Evidence anchors:
    - `crates/fret-diag/src/commands/sidecars.rs`
    - `crates/fret-diag/src/commands/query.rs`
    - `crates/fret-diag/src/commands/index.rs`
    - `crates/fret-diag/src/commands/artifacts.rs`
    - `crates/fret-diag/src/lib.rs`

## M2b: Sidecar access is fully centralized (finish the sweep)

- [x] Migrate remaining commands to the shared sidecar helpers (start with `crates/fret-diag/src/commands/slice.rs`).
- [x] Add a small “doctor” command that reports missing/invalid sidecars and suggests the exact regen command.
  - Evidence: `crates/fret-diag/src/commands/doctor.rs`
  - Related: `fretboard diag test-ids-index <bundle>` (explicit generator for `test_ids.index.json`).
  - Bonus: `diag ai-packet` now writes `doctor.json` into the packet for agent-friendly preflight.
  - Agent ergonomics: `diag doctor --fix` can materialize `bundle.json` from manifest chunks (when present) and regenerate common sidecars (including `frames.index.json`).
  - Agent ergonomics: `diag doctor --fix-dry-run` prints/exports a plan without writing files.
  - CI/agents: `diag doctor --check` (required sidecars) / `--check-all` (all listed sidecars) exits non-zero when unmet.
  - Repair guidance: `doctor.json` includes `repairs[]` with concrete commands like `--fix-bundle-json` / `--fix-sidecars` for self-healing loops.
- [x] Add `--bundle-doctor` integration for `diag run` / `diag suite` / `diag perf` (per-bundle preflight).
  - Modes: `check` / `check-all` / `fix` / `fix-dry-run`.
  - Evidence anchors:
    - `crates/fret-diag/src/lib.rs`
    - `crates/fret-diag/src/commands/doctor.rs`

## M3: Tooling + AI loop

- [x] Define CLI “agent presets” (commands + env vars) for repeatable triage.
  - `fretboard diag agent <bundle> --warmup-frames <n>`
- [x] Document a recommended “agent loop” that prefers sidecars over large `bundle.json`.
  - `docs/workstreams/diag-fearless-refactor-v1/agent-loop.md`
- [x] Add `diag triage --lite` as the default-first entrypoint for huge bundles (frames-index based).
- [x] Add `diag hotspots --lite` as a frames-index-based fallback when `bundle.json` is too large to analyze as JSON.
- [x] Include lite reports in `diag ai-packet` (so agents can start from `triage.lite.json` / `hotspots.lite.json`).
- [x] Publish an explicit migration plan (Option 1 first, Option 2 later).
  - `docs/workstreams/diag-fearless-refactor-v1/migration-plan.md`
- [ ] Prefer structured evidence diffs over screenshot diffs where possible.
- [ ] Document a recommended script authoring style for stability (selectors first, bounded waits).

## M4: Remove debt (finish migration, delete redundant code)

Goal: after the mechanical moves land, **remove the temporary shims** and delete redundant/legacy code paths so the
diagnostics stack stays easy to evolve.

- [ ] Tighten module boundaries in `ecosystem/fret-bootstrap/src/ui_diagnostics/` (no backsliding into the monolith):
  - Script execution / state / step handlers live under `script_engine.rs` + `script_steps_*`.
  - Bundle dumping + sidecar writers live under `bundle_dump.rs` / `bundle_index.rs`.
  - DevTools WS wiring lives under `ui_diagnostics_devtools_ws.rs`.
- [ ] Delete transitional glue after migration:
  - [x] Remove the old inline `diag repro` implementation from `crates/fret-diag/src/lib.rs` after extraction (no redundant copies).
  - remove duplicated per-step dispatch code from the old location(s),
  - avoid “forwarder wrappers” that exist only because of historical file layout.
- [x] Delete the legacy `apply_post_run_checks(...)` mega-signature once all callers are on `diag_run::RunChecks`.
- [ ] Remove redundant semantics traversal helpers in gates:
  - prefer `crate::json_bundle::SemanticsResolver` + shared helpers (no `debug.semantics.nodes` re-greps).
- [ ] Publish and enforce a bundle schema compatibility matrix (v1/v2) for in-tree workflows.
  - Doc home: `docs/workstreams/diag-fearless-refactor-v1/migration-plan.md`
  - Tooling: `diag doctor` should warn when bundles are produced with legacy-only knobs or unexpected schema versions.
- [ ] Reduce “stats mega-module” churn permanently:
  - keep `crates/fret-diag/src/stats.rs` as a small index/exports surface,
  - large check families stay in `crates/fret-diag/src/stats/*.rs`.
- [ ] Audit and remove dead/legacy code paths once consumers have migrated:
  - legacy env knobs that are no longer used,
  - legacy schema compatibility layers that are no longer needed for in-tree workflows.

## Evidence anchors (keep updated as implementation changes)

- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_types.rs`
- `crates/fret-diag/src/stats.rs`
- `crates/fret-diag/src/stats/ui_gallery_markdown_editor.rs`
- `crates/fret-diag/src/stats/ui_gallery_code_editor.rs`

## Notes (2026-02-23)

- [x] Extract sidecar writing helpers out of `bundle_dump.rs` into
      `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_sidecars.rs` to reduce dump churn.
- [x] Split schema-specific dump logic into `dump_schema_v1` / `dump_schema_v2` helpers to keep
      `dump_bundle_with_options` mostly about option resolution + dispatch.
- [x] Factor dump finalization (write bundle.json, WS notify, sidecars, counters) into a shared helper
      to reduce v1/v2 duplication.
- [x] Reuse `UiArtifactStatsV1` directly as the dump return value (remove intermediate count structs).
- [x] Migrate `diag bundle-v2` conversion to shared semantics helpers (`json_bundle::snapshot_semantics`) and
      prune `tables.semantics` after applying semantics mode (avoid oversized converted bundles).
- [x] Extract `build_semantics_table_entries_from_windows` helper into `crates/fret-diag/src/json_bundle.rs` to
      avoid repeating v1→v2 semantics table construction logic.
- [x] Update `fret-diag` lints to resolve semantics via `json_bundle::SemanticsResolver` so schema v2 bundles are linted correctly.
