---
title: Diagnostics Extensibility + Capabilities v1 - Determinism
status: draft
date: 2026-02-10
scope: diagnostics, determinism, flake, triage
---

# Diagnostics Extensibility + Capabilities v1 - Determinism

This document is a sub-part of `docs/workstreams/diag-extensibility-and-capabilities-v1/README.md`.

Goal: turn “flaky” regressions into actionable reports by capturing environment fingerprints and enabling repeat-run
triage workflows.

## Environment fingerprint (bundle-level)

Bundles SHOULD include a deterministic fingerprint of inputs that commonly cause nondeterminism:

- platform + runner kind (`native`/`web`),
- DPI / scale factor,
- font selection/fallback summary,
- feature flags relevant to UI behavior (view cache, redaction, screenshots),
- timing sources used by the runner (monotonic clocks, vsync policy),
- versions (app, framework, renderer).

This is not for blame; it is for explainability and reproducibility.

Current bundle surface (implemented):

- `bundle.json.env`:
  - `runner_kind` (`native`/`web`),
  - target triple summary (`target_os`/`target_family`/`target_arch`),
  - diagnostics flags (semantics, redaction, screenshots, WS transport),
  - declared `diag.*` capabilities,
  - `monitor_topology` (host monitor inventory + virtual desktop bounds when the runner publishes
    that environment source),
  - `scale_factors_seen` (last-known per-window scale factors).

Important boundary:

- `monitor_topology` is the host environment fingerprint.
- `scale_factors_seen` remains run-observed per-window evidence collected during the diagnostics
  run; it is not a substitute for host monitor inventory.
- `debug.environment` remains a per-window runtime/debug surface, not a campaign preflight
  contract.
- host-environment predicates belong to the dedicated follow-on lane
  `docs/workstreams/diag-environment-predicate-contract-v1/`.

## Repeat-run triage

Add a workflow that runs the same script N times and classifies differences:

- semantics diffs,
- layout diffs (bounds/overflow),
- hit-test/routing diffs (trace),
- performance diffs (frame stats).

Outputs:

- `repeat.summary.json` (machine-readable),
- `repeat.summary.json.highlights` aggregates stage/reason-code counts, lint totals, evidence summaries, and worst perf run pointers,
- links/paths to the worst-case bundles for inspection.

Command (native, filesystem-trigger transport):

- `cargo run -p fretboard-dev -- diag repeat <script.json> --repeat <n> [--launch -- <cmd...>]`

Behavior:

- writes `repeat.summary.json` under `--dir` (default: `target/fret-diag/`),
- exits with code 1 if any run fails or if successful runs diverge from the baseline bundle.

## Flake mitigation knobs (runner/tooling)

Prefer contract-backed mitigation (no wall-clock sleeps):

- intent-level steps (`click_stable`, `ensure_visible`),
- predicate-based waits (`wait_until`),
- bounded retries with structured reasons.

If a mitigation requires support (e.g. ROI screenshots, coordinate injection), gate it via capabilities.
