# Diag simplification v1 - TODO

This TODO list is scoped to **simplification + transport parity** and is intended to be landable via
small, fearless refactors.

## Phase 0: Inventory + gates (prep)

- [x] Document current transport capability matrix (filesystem vs WS).
- [x] Define a stable registry policy for:
  - [x] `reason_code` naming and backward-compat rules
  - [x] `capabilities` naming (namespaced strings) and ownership
- [x] Add/confirm a small nextest gate set for diag tooling:
  - [x] `cargo nextest run -p fret-diag-protocol`
  - [x] `cargo nextest run -p fret-diag-ws --no-tests=pass` (currently no tests; compile gate only)
  - [x] `cargo nextest run -p fret-diag` (tooling unit tests)
- [x] Add a minimal "WS export materialization" smoke test (tooling only; no browser required):
  - feed a `bundle.dumped` message containing embedded bundle JSON
  - assert a local `bundle.json` directory is produced

Evidence pointers:

- M0 baseline note: `docs/workstreams/diag-simplification-v1-m0-baseline.md`
- WS materialization smoke tests: `crates/fret-diag/src/lib.rs` (`materialize_devtools_bundle_dumped_*`)

## Phase 1: Tooling transport abstraction

- [x] Introduce a transport trait in `crates/fret-diag` (tooling-only):
  - [x] FS implementation (existing behavior)
  - [x] WS implementation (existing behavior)
  - Evidence: `crates/fret-diag/src/transport/mod.rs` (`trait DiagTransport`, `ToolingDiagClient`), `crates/fret-diag/src/transport/fs.rs`, `crates/fret-diag/src/transport/ws.rs`
- [x] Make `diag run` and `diag suite` transport-agnostic:
  - [x] unify "wait ready" behavior
  - [x] unify "send script" behavior
  - [x] unify "read script result" behavior
- [x] Route `diag repro` through the shared orchestration path:
  - [x] use `run_script_over_transport` for script execution
  - [x] make bundle selection/dump fully transport-agnostic
- [x] Add a transport-agnostic "streaming results" hook:
  - [x] allow tooling to consume `script.result` updates incrementally (useful for long suites)
    - Evidence: `crates/fret-diag/src/lib.rs` (`run_script_over_transport` incremental writes)

## Phase 2: Artifact materialization parity

- [x] Define local artifact layout contract (tooling-side):
  - `<out_dir>/<run_id>/bundle.json`
  - `<out_dir>/<run_id>/script.result.json`
  - optional screenshots directories/files
- [x] Implement WS bundle materialization:
  - [x] on `bundle.dumped(bundle=...)`, write `bundle.json` locally
  - [x] write `latest` pointer (same as filesystem mode)
  - [x] plumb through pack/triage/lint paths (operate on local artifacts)
- [x] Make `--pack` work in WS mode by operating on the materialized local artifact.
- [x] Add artifact size reporting:
  - [x] include bytes on disk and clipped counts in `script.result` evidence (bounded)

## Phase 3: Exit request parity

- [x] Add a WS message for exit request (`app.exit.request` or `diag.exit.request`).
- [x] Wire runtime to honor the exit request (native + wasm).
- [x] Update tooling:
  - [x] in `--launch` mode, exit after run by default (new behavior)
  - [x] add `--keep-open` to preserve existing workflows
  - [x] keep `--touch-exit-after-run` as an alias or deprecate it in favor of transport-neutral naming

## Phase 4: Retention and evidence (bounded)

- [x] Add a bounded per-run event log (step start/end, dumps, major traces).
- [x] Include the event log in `script.result` evidence (bounded).
- [ ] Add an option to export snapshots around step boundaries (bounded; future).
- [ ] Ensure every failure path produces:
  - [ ] stable `reason_code`
  - [ ] bounded structured evidence (not just "timeout")
- [x] Add a "capabilities missing" failure mode that is explicit and immediate.
- [x] Ensure tooling-side failures/timeouts write a local `script.result.json` with stable `reason_code` (avoid "no artifact + timeout").
- [x] Ensure suite setup/driver errors write `suite.summary.json` with `error_reason_code` + a local `script.result.json`.
- [x] Ensure repro setup/driver errors write `repro.summary.json` with `error_reason_code` + a local `script.result.json`.
- [x] Ensure repeat setup/driver errors write `repeat.summary.json` with `error_reason_code` + a local `script.result.json`.

## Phase 5: Artifact format v2 (manifest + chunks)

- [x] Tooling writes a minimal per-run `manifest.json` (index + size stats) alongside v1 artifacts.
- [ ] Define `manifest.json` + chunk directory layout (v2 artifact format).
- [ ] Keep `bundle.json` as a compatibility artifact (either generated or optional).
- [ ] Update pack/triage/lint to accept both v1 and v2 artifact layouts.
- [ ] Introduce chunking policy for WS:
  - [ ] avoid giant single messages
  - [ ] support content-addressing or chunk ids (future)

## Phase 6: Config consolidation (compat-first)

- [ ] Add `FRET_DIAG_CONFIG_PATH` support and a canonical config file.
- [ ] Make tooling generate and pass the config file when launching.
- [ ] Deprecate ambiguous env vars by introducing explicit replacements (keep old names supported).
