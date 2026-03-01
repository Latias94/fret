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
- `diag perf <suite-name>` suite expansion is single-sourced (avoid drift between perf entrypoints and seed policy) and
  is derived from promoted registry suite memberships (`tools/diag-scripts/index.json` + `suite_memberships`).
- A script library taxonomy decision is recorded (folder layout + suite definition strategy).
- Basic script discoverability exists (avoid “grep the repo”):
  - `diag run` accepts promoted `script_id` (registry-backed),
  - `diag list scripts` prints `script_id -> path` from the promoted registry.
- Basic suite discoverability exists (avoid “find the right folder”):
  - `diag list suites` prints known `suite_memberships` (registry-derived).
- Script library drift is detectable via a bounded, read-only tooling command:
  - `diag doctor scripts` checks for root canonical scripts, broken redirects, and registry drift.
- Tool-launched runs have a first-class “parallel agents” escape hatch:
  - `--session-auto` allocates an isolated session root under `<base_dir>/sessions/<session_id>/` for `--launch` runs,
  - sessions are discoverable (`diag list sessions`) and cleanable (`diag sessions clean`).

Evidence anchors:

- Docs: `docs/ui-diagnostics-and-scripted-tests.md`
- Example config: `tools/diag-configs/diag.config.example.json`
- Suites: `tools/diag-scripts/suites/README.md`
- Concurrency hygiene: `docs/workstreams/diag-v2-hardening-and-switches-v1/concurrency-and-sessions.md`
- Agent-era plan: `docs/workstreams/diag-v2-hardening-and-switches-v1/ai-era-debugging-stack.md`

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
- Tool-launched runs have a single explicit escape hatch to re-enable raw `bundle.json` writing:
  - `--launch-write-bundle-json` (requires `--launch`; not supported for `diag matrix`).
- A migration checklist is complete for in-repo scripts and CI gates.

## M5: Agent-native scripting and multi-viewport evidence (P1/P2)

Outcome:

- Scripts become easier for agents/humans to author and review (smaller diffs, less repetition), and multi-window docking
  failures become explainable with bounded evidence rather than opaque timeouts.

Exit criteria:

- Named references / scopes exist (ImGui `SetRef(...)`-style ergonomics, semantics-first):
  - scripts can set/clear a base ref,
  - selector-driven steps can use relative selectors scoped to that base,
  - failures report a stable `reason_code` when a ref is missing or resolves to multiple nodes.
- Multi-viewport docking evidence exists (bounded, queryable):
  - a bounded `window.map.json` sidecar is exported (window ids + last bounds + hover detection),
  - routing decisions that matter for docking (hover/click target window selection) are recorded in a bounded log
    (`dock.routing.json`),
  - tooling offers bounded queries to inspect this without opening large artifacts (`diag windows`, `diag dock-routing`).
- Fast mode is an explicit policy (config-driven) and has a small smoke suite proving it doesn’t introduce flake.

Evidence anchors (expected):

- Protocol: `crates/fret-diag-protocol/src/lib.rs`
- Runtime script engine: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- Tooling: `crates/fret-diag/src/*`

Status (2026-02-28):

- Base ref steps (`set_base_ref` / `clear_base_ref`) exist and runtime scopes selector resolution while active.
- Named ref map + relative selector syntax are still pending (v1 covers the most common “scope to panel” case).
- Bundles export a bounded `window.map.json` (window ids + last bounds + hover detection) and `dock.routing.json` (routing
  evidence for docking/tear-out); tooling provides bounded reports (`diag windows`, `diag dock-routing`).
- Script runtime hardening reduces avoidable “timeout” flakes by failing fast with stable `reason_code`:
  - oversized targets (`scroll_into_view`, `ensure_visible(within_window=true)`),
  - impossible stability configs (`stable_frames > timeout_frames` for stability-gated steps).
