# Diagnostics Architecture (Fearless Refactor v1) — Current Architecture Map

Status: Draft (evidence note)

This is a “where is what?” map for the diagnostics stack as it exists today. It is intentionally
evidence-first (clickable anchors) so refactors can stay grounded.

## 1) CLI entrypoints (how `fretboard diag` reaches the engine)

- `apps/fretboard/src/cli.rs` (top-level CLI dispatch)
- `apps/fretboard/src/diag.rs` (thin shim)
  - calls `fret_diag::diag_cmd(...)`
- `crates/fret-diag/src/lib.rs` (`pub fn diag_cmd(args: Vec<String>) -> Result<(), String>`)

## 2) “Tooling engine” command modules (`crates/fret-diag`)

The `diag_cmd` parser in `crates/fret-diag/src/lib.rs` dispatches into command modules:

- `crates/fret-diag/src/diag_suite.rs` — suite orchestration (resolve suite → scripts → run → post-run checks)
  - builtin suite mapping: `resolve_builtin_suite_scripts`
- `crates/fret-diag/src/diag_run.rs` — run a single script (or script id) with launch/transport wiring
- `crates/fret-diag/src/diag_repro.rs` — “one-button repro” runner (script + launch + optional pack)
- `crates/fret-diag/src/diag_list.rs` — list suites/scripts/sessions (registry-backed)
- `crates/fret-diag/src/diag_pick.rs` / `crates/fret-diag/src/diag_inspect.rs` — inspector/picker workflows

Supporting “engine” subsystems:

- `crates/fret-diag/src/transport/*` — FS transport + WS transport seam
- `crates/fret-diag/src/artifacts/*` — bundle/manifest materialization and packing
- `crates/fret-diag/src/bundle_index.rs` — indexing + derived views (test ids, windows, dock routing, etc.)
- `crates/fret-diag/src/commands/*` — smaller subcommands (query/slice/config/resolve/etc.)

## 3) Registries (de-monolithization seams inside `crates/fret-diag`)

Goal: new suites/checks should plug in without growing a central match statement.

- `crates/fret-diag/src/registry/suites.rs`
  - `SuiteRegistry` (suite name → suite definition)
  - `SuiteResolver` (suite name → scripts; promoted suites + suite-dir suites)
- `crates/fret-diag/src/registry/checks.rs`
  - `CheckRegistry` scaffolding (seam only; first real migration pending)

## 4) Protocol types (portable schema)

- `crates/fret-diag-protocol/src/lib.rs`

This crate is the “wire + artifact” schema surface. It should remain portable (wasm-friendly) and
avoid pulling in native-only dependencies.

## 5) Runtime capture + script execution (where artifacts come from)

Runtime-side capture/export (native + web runners) lives outside the tooling engine:

- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`

This is where:

- snapshots are captured (semantics/debug surfaces),
- scripted actions are executed,
- bundles and sidecars are written for tooling to consume.

## 6) Layout deep-debug escape hatch (today)

There is already a “best-effort” layout dump path (native-focused):

- `crates/fret-ui/src/tree/layout/taffy_debug.rs`
- `crates/fret-ui/src/runtime_config.rs` (env wiring)

Workstream direction (later milestones): connect this to `diag` artifacts via a bounded sidecar
contract so layout issues can be debugged without ad-hoc UI changes.

