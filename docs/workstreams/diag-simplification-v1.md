# Diag simplification v1 (fearless refactor)

Status: Active (workstream tracker)

Current state (as of 2026-02-13):

- WS artifact materialization + `--pack` parity landed (tooling materializes `bundle.dumped` to a local `bundle.json` directory).
- Artifact size stats are reported in `script.result.json` for locally materialized bundles (bytes + bounded counts).
- Bundle path resolution now prefers the stable per-run `<out_dir>/<run_id>/bundle.json` when `script.result.json` is present (less reliance on `latest.txt`).
- Tooling failures now produce a deterministic `script.result.json` with stable `reason_code` (e.g. `tooling.*`, `timeout.tooling.*`) instead of degrading to "no artifact + timeout".
- Tooling-side "failure artifact" helpers are now isolated into focused modules (`crates/fret-diag/src/tooling_failures.rs`, `crates/fret-diag/src/run_artifacts.rs`) to reduce monolith churn risk.
- Tooling now writes a minimal per-run `manifest.json` next to `script.result.json`/`bundle.json` (v2 direction; still v1-compatible).
- DevTools WS bundle dumps can now be delivered as chunked `bundle.dumped` messages to avoid oversized single payloads; tooling reassembles and materializes locally.
- `diag repro` setup/driver failures now write `repro.summary.json` with `error_reason_code` (and still produce a local `script.result.json`).
- `diag repeat` setup/driver failures now write `repeat.summary.json` with `error_reason_code` (and still produce a local `script.result.json`).
- `script.result.json` now includes a bounded per-run event log (step start/end + bundle dump events) with clipped counts reported.
- Missing required diagnostics capabilities now fail fast with a stable `reason_code` and structured evidence (avoid timeouts).
- WS exit request landed (`app.exit.request`) and tooling supports `--exit-after-run` (`--touch-exit-after-run` remains as an alias).
- Default deterministic exit in `--launch` mode landed and `--keep-open` preserves long-running/manual workflows.

## Context

Fret diagnostics ("diag") currently serves multiple audiences:

- Humans doing interactive triage (evidence-first debugging).
- CI/regression gating (scripts + assertions + deterministic checks).
- Automated agents (headless orchestration, "run and exit" semantics, stable artifacts).

The system works, but key capabilities are split across transports (filesystem vs DevTools WS), and
the end-to-end artifact pipeline is not uniform. This workstream proposes a **fearless refactor**
that simplifies the architecture without breaking existing scripts/contracts.

Related docs:

- `docs/ui-diagnostics-and-scripted-tests.md`
- ADR: `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- Existing workstream: `docs/workstreams/diag-extensibility-and-capabilities-v1/README.md`
- M0 baseline: `docs/workstreams/diag-simplification-v1-m0-baseline.md`

## Goals

1. **Transport parity**: `diag run/suite/repro` behave the same over filesystem and DevTools WS.
2. **Artifact parity**: a run always produces a local, shareable artifact bundle (bundle + result + optional screenshots),
   even when the app cannot write to disk (web runner).
3. **Explicit lifecycle**: scripts do not rely on long timeouts; launched processes can exit deterministically.
4. **Bounded but useful evidence**: evidence remains bounded, but supports "long tail" debugging beyond "last N frames".
5. **Compatibility**: preserve existing script schemas, reason codes, and CLI flags where possible.

## Non-goals

- Replace the semantics system (ADR 0033) or rewrite the UI runtime.
- Invent a brand-new scripting DSL; JSON v1/v2 remain valid.
- Full replay of GPU command streams; this is diagnostics/gating, not full capture/replay.

## Current pain points (summary)

- **Filesystem vs WS mode is not symmetric** (feature gaps, different artifact paths, different UX).
- **Bundles are ring-buffered and time-windowed**, which is great for size control but can hide long causal chains.
- **Run termination is not a first-class concept**; users/agents fall back to large `--timeout-ms`.
- **Config surface is large** (`FRET_DIAG_*` env vars + CLI + script meta defaults), and mistakes often degrade to timeouts.
- **Implementation is monolithic** (large files with many responsibilities), increasing merge/refactor risk.

## Principles (fearless refactor posture)

- **Keep contracts stable**: prefer additive protocol fields and shim layers over breaking changes.
- **Make the happy path smaller**: default commands should "just work" without knowing transport details.
- **Separate mechanism from policy**:
  - Mechanism: collecting snapshots, executing steps, emitting structured evidence.
  - Policy/tooling: packing, linting, suite orchestration, diff/compare, reporting.
- **Evidence-first**: every failure should have a stable `reason_code` and bounded structured evidence.
- **Extensibility by seams**: new steps/predicates/checks should plug into narrow interfaces, not grow monoliths.
- **Determinism is a feature**: avoid "best effort" fallbacks in gating; missing capabilities should fail fast.
- **Bounded cost**: diagnostics must have explicit size/time bounds and report when evidence is clipped.

## Proposed architecture (v1)

### 1) A single transport abstraction

Introduce a `DiagTransport` concept in tooling (not in the UI runtime):

- `FilesystemTransport`: watches/touches files under `FRET_DIAG_DIR`.
- `DevtoolsWsTransport`: sends/receives messages over WS using the existing protocol surface.

Both transports expose the same *logical* operations:

- `capabilities() -> AvailableCapabilities`
- `send_script(...)`
- `wait_script_result(run_id, timeout)`
- `request_bundle_dump(label, options)`
- `subscribe_results(...)` (optional; streaming `script.result` and `bundle.dumped`)
- `subscribe_events(...)` (optional; used for streaming results/logs)
- `request_exit(reason, delay)` (see "Exit semantics")

Tooling should become transport-agnostic; transport choice is a detail.

### 2) Artifact materialization is always local

Define a tooling-side invariant:

> Every successful `capture_bundle` produces a local directory containing `bundle.json` and a stable index pointer.

How it works:

- Filesystem mode: the app writes `bundle.json` directly (status quo).
- WS mode:
  - `bundle.dumped` must embed `bundle` (or provide a stable content address).
  - tooling materializes the bundle payload into a local directory under the run output dir,
    matching filesystem layout (`bundle.json`, optional `frame.bmp`, optional screenshots).

This removes the "web runner cannot write to disk" mismatch and simplifies triage/pack/lint.

#### Artifact format evolution (manifest + chunks)

To keep refactors fearless, start by materializing the existing `bundle.json` layout in both transports.
However, plan for an artifact v2 format to prevent `bundle.json` from becoming a scaling bottleneck:

- `manifest.json` (small, stable index: run_id, timestamps, capabilities, chunk list, size stats).
- `chunks/` (bounded, optional):
  - `snapshots.jsonl.zst` (or split per snapshot id)
  - `evidence.json` (script evidence + traces)
  - `screenshots/` (PNG, request logs)

Tooling should accept either:

- v1: `bundle.json` monolith
- v2: `manifest.json` + chunks (preferable for web/remote transport and large suites)

### 3) Exit semantics become a first-class request

Add a transport-neutral "exit request":

- Filesystem: tooling touches `${FRET_DIAG_EXIT_PATH}` (status quo).
- WS: add a message like `app.exit.request` (or `diag.exit.request`) with `{ reason?, delay_ms? }`.

Policy:

- If tooling launches the app (`--launch`), it should default to deterministic exit:
  - `diag run` / `diag suite`: exit after the script completes (success or failure), unless `--keep-open`.
- If tooling does not launch the app (`--reuse-launch` / WS without launch), exit is opt-in.

### 4) Bundle retention evolves beyond "last N frames"

Keep the bounded snapshot ring buffer (it is necessary), but add bounded "key event markers":

- A compact per-run event log (bounded) emitted alongside the script result:
  - step start/end
  - selector resolution outcome
  - click stable/bounds stable traces
  - focus/IME routing traces
  - overlay placement decisions
  - bundle dumps (label + timestamp)

Additionally, support two export modes for `capture_bundle`:

- `max_snapshots` (already supported): caps snapshot count in the dump.
- `window`/`frame_range` (future): export snapshots around an explicit marker
  (e.g. "last N before failure", "from step K to step K+M"), still bounded.

### 5) Config surface consolidation (compat-first)

Do not remove env vars, but add a single canonical config file entry point:

- `FRET_DIAG_CONFIG_PATH` (tooling sets this when launching).
- The config file contains all paths/caps and feature toggles.
- Env vars remain as overrides (for manual workflows), but tooling defaults should come from the config file.

Also, deprecate ambiguous env names by introducing unambiguous replacements (keep old ones working):

- Prefer a single `FRET_DIAG_SCREENSHOTS=1` for scripted screenshot steps.
- Use a single `FRET_DIAG_BUNDLE_SCREENSHOT=1` (or similar) for per-dump frame image output.

### 6) Implementation split (reduce monolith risk)

Within the repo:

- Keep `crates/fret-diag-protocol` as the stable contract crate.
- Split `crates/fret-diag` tooling crate:
  - `transport/` (fs + ws)
  - `artifact_store/` (materialize/pack)
  - `run/` (run/suite orchestration)
  - `post_checks/` (stats/lint/triage/compare)
- Keep `ecosystem/fret-bootstrap` runtime pieces, but isolate:
  - script executor
  - bundle exporter
  - ws bridge
  - ring buffers + evidence collectors

The objective is not to rewrite logic, but to move code behind clearer seams so changes are smaller and safer.

### 7) Extensibility hooks (avoid growing a new monolith)

Define explicit plugin points:

- Step execution:
  - Keep low-level steps (pointer/keyboard) minimal and deterministic.
  - Prefer new behavior as v2 "intent steps" (compose low-level primitives + evidence).
- Predicate evaluation:
  - New predicates must produce structured trace on failure (bounded).
- Post-run checks:
  - Model checks as pure functions: `artifact -> report`.
  - Keep transport out of the check layer.

This enables future additions (mobile/ADB transport, remote runners, new checks) without rewriting core flows.

### 8) Determinism and budgets

Determinism:

- Prefer fixed frame clock in gating and suite runs when possible.
- Make nondeterministic sources explicit capabilities (e.g. "wall_clock_time").

Budgets:

- Artifact size and evidence size must be bounded and reported (clipped counts, bytes).
- Web transport must avoid unbounded WS messages; chunking/content-addressing should be the long-term posture.

### 9) Security and privacy posture

- Redaction must remain on by default; treat any "raw text" export as an explicit opt-in capability.
- Tokens (DevTools) should have a clear lifecycle; tooling should avoid logging secrets.
- Artifact packs should be safe to share by default (no sensitive env dumps unless explicitly requested).

## Compatibility and migration plan (high level)

Phase 0: "no behavior change" refactor seams

- Introduce tooling transport abstraction and route existing code through it.
- Materialize bundles locally in WS mode without changing script schema.

Phase 1: exit request parity

- Add WS exit request message and wire it into runtime + tooling.
- Switch launched runs to exit by default; add `--keep-open` to preserve old behavior.

Phase 2: retention/evidence improvements

- Add bounded per-run event log and unify how evidence is surfaced in `script.result`.
- Add optional export selection around markers (future).

Phase 3: artifact format v2

- Introduce `manifest.json` + chunk layout, keeping `bundle.json` as a compatibility layer.
- Add tooling support to pack, triage, and lint from either v1 or v2 artifact format.

## Risks and mitigations

- Risk: expanding protocol surface breaks older tooling.
  - Mitigation: additive fields + schema versioning + tolerant parsing.
- Risk: artifact sizes grow unexpectedly (especially in web mode).
  - Mitigation: keep strict bounds; make embed bundle opt-in except for wasm; include size stats in results.
- Risk: transport parity hides platform-specific failure modes.
  - Mitigation: capabilities gate must fail fast with structured evidence ("missing capability", not timeout).
- Risk: `bundle.json` becomes a long-term performance bottleneck.
  - Mitigation: plan for v2 manifest + chunks; avoid coupling checks to a single monolithic file.
- Risk: new checks/steps increase coupling and regress determinism.
  - Mitigation: enforce "intent step" discipline + bounded trace + explicit capabilities for nondeterminism.

## Success criteria

- A single `fretboard diag run` command can run against native or web and always produces:
  - `script.result.json`
  - a local bundle directory containing `bundle.json` (and optional screenshots)
  - a packable artifact (zip) via the same command path
- A launched run exits deterministically without relying on large timeouts.
- Adding a new step/predicate/check does not require modifying large monolithic files; it plugs into a narrow seam.
- WS mode can export artifacts without requiring the app to write to disk (materialization in tooling).
