---
title: Diag v2 Hardening + Switches Refactor v1 - Milestones
status: draft
date: 2026-02-26
scope: diagnostics, automation, artifacts, config, refactor
---

# Milestones

Each milestone should land reviewable, additive steps with clear evidence anchors. Do not batch-rewrite.

## M0: Switches are unambiguous (P0)

Outcome:

- A single canonical config resolution story exists (tooling + runtime), and the example config matches reality.

Exit criteria:

- `FRET_DIAG_CONFIG_PATH` is the primary entry point in docs and in tooling `--launch` runs.
- Deprecated env vars/flags are documented and have compatibility shims.
- A tooling command prints the effective config and highlights deprecated inputs.
- Tooling produces schema v2 scripts by default (runtime v1 parsing is not required for the common case).
- Built-in scripted suites are not defined by Rust-side hard-coded file lists (directory inputs + redirect stubs are acceptable as an intermediate step).
- A script library taxonomy decision is recorded (folder layout + suite definition strategy).
- Basic script discoverability exists (avoid “grep the repo”):
  - `diag run` accepts promoted `script_id` (registry-backed),
  - `diag list scripts` prints `script_id -> path` from the promoted registry.
- Script library drift is detectable via a bounded, read-only tooling command:
  - `diag doctor scripts` checks for root canonical scripts, broken redirects, and registry drift.

Evidence anchors:

- Docs: `docs/ui-diagnostics-and-scripted-tests.md`
- Example config: `tools/diag-configs/diag.config.example.json`
- Suites: `tools/diag-scripts/suites/README.md`

## M1: Manifest exists for every run (P1)

Outcome:

- Every run produces a per-run directory with a manifest, for both filesystem and DevTools WS transports.

Exit criteria:

- `diag run/suite/repro/perf` always emits `<out_dir>/<run_id>/manifest.json`.
- Tooling failures still write a local `script.result.json` with stable `reason_code`.
- Filesystem dump requests can carry dump metadata (label/max snapshots/request id), matching WS semantics.
- Tooling can validate per-run directories via `diag artifact lint` (manifest + chunks + sidecars + run_id/timestamps).

Evidence anchors:

- Tooling: `crates/fret-diag/src/run_artifacts.rs`, `crates/fret-diag/src/tooling_failures.rs`, `crates/fret-diag/src/transport/fs.rs`
- Tooling lint: `crates/fret-diag/src/artifact_lint.rs`, `crates/fret-diag/src/commands/artifact.rs`
- Runtime: `ecosystem/fret-bootstrap/src/ui_diagnostics/*`, `ecosystem/fret-bootstrap/src/ui_diagnostics/fs_triggers.rs`

## M2: Manifest-first pack + AI packet (P1)

Outcome:

- Sharing and triage are “small by default”; packing does not require bundle monoliths.

Exit criteria:

- `diag pack --ai-only` succeeds using manifest + sidecars (no `bundle.json` required).
- `diag ai-packet` prefers manifest/chunks and records clip/drop decisions in `ai.packet.json`.
- Script suites are not tightly coupled to flat filenames; suites are directory + redirect-stub driven, and a minimal
  generated registry exists for “promoted” scripts (suite-reachable + `_prelude`) to make drift visible and reviewable.

Evidence anchors:

- Tooling: `crates/fret-diag/src/commands/ai_packet.rs`, `crates/fret-diag/src/pack_zip.rs`
- Scripts: `tools/diag-scripts/index.json`, `tools/check_diag_scripts_registry.py`
- Workstream: `docs/workstreams/diag-ai-agent-debugging-v1.md`

## M3: Compatibility logic boxed (P2)

Outcome:

- Legacy behaviors still work, but are isolated and trackable.

Exit criteria:

- Compat fallbacks live in explicit `compat/` modules.
- `triage.json` and/or `ai.packet.json` record when compat fallbacks were used.
  - Status (2026-02-27): triage and AI packets include a bounded `compat.markers` list derived from bundle schema/version
    sniffing, legacy capabilities detection, and `script.result.json` `compat.*` event log entries.
- Transport differences (filesystem vs DevTools WS) are isolated behind an explicit tooling seam contract.
  - Status (2026-02-27): `crates/fret-diag/src/transport/seam.rs` consolidates request-id correlation and baseline-race mitigations.
- Multi-window targeting semantics are consistent across selector-driven v2 steps (no silent “window-local only” gaps).
  - Status (2026-02-27): selector-driven v2 steps now support optional `window` targeting and tooling infers
    `diag.multi_window` when targeting “other windows”.

Evidence anchors:

- Tooling: `crates/fret-diag/src/*`

## M4: Legacy writers off by default (P3)

Outcome:

- Default runs write the minimal artifacts needed for triage and gates; legacy paths become opt-in.

Exit criteria:

- Default scripted runs do not materialize `bundle.json` unless explicitly requested.
- A migration checklist is complete for in-repo scripts and CI gates.
