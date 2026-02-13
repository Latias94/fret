# Diag simplification v1 - TODO

This TODO list is scoped to **simplification + transport parity** and is intended to be landable via
small, fearless refactors.

## Phase 0: Inventory + gates (prep)

- [ ] Document current transport capability matrix (filesystem vs WS).
- [ ] Define a stable registry policy for:
  - [ ] `reason_code` naming and backward-compat rules
  - [ ] `capabilities` naming (namespaced strings) and ownership
- [ ] Add/confirm a small nextest gate set for diag tooling:
  - [ ] `cargo nextest run -p fret-diag-protocol`
  - [ ] `cargo nextest run -p fret-diag-ws` (if applicable)
  - [ ] `cargo nextest run -p fret-diag` (tooling unit tests)
- [ ] Add a minimal "WS export materialization" smoke test (tooling only; no browser required):
  - feed a `bundle.dumped` message containing embedded bundle JSON
  - assert a local `bundle.json` directory is produced

## Phase 1: Tooling transport abstraction

- [ ] Introduce a transport trait in `crates/fret-diag` (tooling-only):
  - [ ] FS implementation (existing behavior)
  - [ ] WS implementation (existing behavior)
- [ ] Make `diag run` and `diag suite` transport-agnostic:
  - [ ] unify "wait ready" behavior
  - [ ] unify "send script" behavior
  - [ ] unify "read script result" behavior
- [ ] Add a transport-agnostic "streaming results" hook:
  - [ ] allow tooling to consume `script.result` updates incrementally (useful for long suites)

## Phase 2: Artifact materialization parity

- [ ] Define local artifact layout contract (tooling-side):
  - `<out_dir>/<run_id>/bundle.json`
  - `<out_dir>/<run_id>/script.result.json`
  - optional screenshots directories/files
- [x] Implement WS bundle materialization:
  - [x] on `bundle.dumped(bundle=...)`, write `bundle.json` locally
  - [x] write `latest` pointer (same as filesystem mode)
  - [x] plumb through pack/triage/lint paths (operate on local artifacts)
- [x] Make `--pack` work in WS mode by operating on the materialized local artifact.
- [ ] Add artifact size reporting:
  - [x] include bytes on disk and clipped counts in `script.result` evidence (bounded)

## Phase 3: Exit request parity

- [x] Add a WS message for exit request (`app.exit.request` or `diag.exit.request`).
- [x] Wire runtime to honor the exit request (native + wasm).
- [ ] Update tooling:
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

## Phase 5: Artifact format v2 (manifest + chunks)

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
