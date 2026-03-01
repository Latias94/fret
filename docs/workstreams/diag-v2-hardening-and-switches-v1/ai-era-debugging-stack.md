---
title: Agent-Era UI Debugging Stack (Contracts + Evidence + Automation) v1
status: draft
date: 2026-02-28
scope: diagnostics, automation, devtools, AI agents
---

# Agent-era UI debugging stack (v1)

This document describes a **contract-first, evidence-first** debugging capability stack for Fret in an “AI code agent”
world. The intent is to make UI bugs:

- reproducible (deterministic scripted repros),
- explainable (bounded, structured evidence + stable `reason_code`),
- shareable (portable artifacts),
- safe to run in parallel (multiple agents, multiple terminals).

This doc is a **design + roadmap**. Implementation lives across `crates/fret-diag` (tooling), `fret-bootstrap` (runtime),
and optional UIs (`apps/fret-devtools`).

## Why this matters

AI agents are excellent at iterating when the system offers:

1) stable query surfaces (selectors, indices, typed evidence),
2) deterministic execution (bounded nondeterminism),
3) small, portable artifacts (no “grep a 200MB JSON”),
4) easy composition (named references, scoping, reusable steps).

Without these, agents fall back to brittle heuristics (timing sleeps, pixel diffs, ad-hoc logs).

## Reference: ImGui Test Engine (what to learn, what not to copy)

We keep `repo-ref/imgui_test_engine/` as a reference point for automation ergonomics. In particular:

- headless runs,
- fast vs human-speed modes,
- screenshot/video capture,
- high-level action APIs (scoped references like `SetRef(...)`),
- “test engine” harness discipline (deterministic execution, crisp failure reporting, replay-friendly artifacts).

However, Fret is not immediate-mode ImGui. We should copy the **capability outcomes**, not the IO injection
implementation details.

Where ImGui injects inputs at the app layer, many UI test systems inject at the OS layer (moving the real cursor,
spamming OS events). For Fret, the default should be:

- **internal injection** (runner/app sees “synthetic input” without warping the real OS cursor),
- **optional external input isolation** while a script is active (avoid human interference),
- explicit “escape hatches” for interactive debugging sessions.

## Capability stack (6 layers)

Think of this as a “minimum viable agent loop” ladder. Each layer should be usable independently, but the full
experience comes from the stack.

### Layer 0: Session isolation (parallel agents)

Problem: filesystem transport uses shared control-plane files (`script.json`, `*.touch`, `latest.txt`), so concurrent
runs in the same `out_dir` race.

Baseline policy:

- Treat `--dir` / `FRET_DIAG_DIR` as a **session boundary**.
- For tool-launched runs (`--launch`), prefer `--session-auto` so tooling allocates
  `<base_dir>/sessions/<session_id>/` automatically.

Evidence:

- Sessions design and CLI: `docs/workstreams/diag-v2-hardening-and-switches-v1/concurrency-and-sessions.md`

### Layer 1: Evidence contracts (bounded artifacts)

Agents need a small, typed artifact to work from:

- per-run `manifest.json`,
- `script.result.json` with stable `reason_code`,
- sidecars/index/meta/slices,
- AI packets (`ai.packet.json`) with budget/clip summaries.

Guideline:

- Prefer schema2 + sidecars; treat raw `bundle.json` as an explicit escape hatch.

Related:

- `docs/workstreams/diag-ai-agent-debugging-v1.md`
- `docs/workstreams/diag-v2-hardening-and-switches-v1/per-run-layout.md`

### Layer 2: Stable targets (selectors)

Agents must be able to reliably target UI elements:

- `test_id` (primary),
- semantics role/name/path,
- optional window targeting for multi-window/multi-viewport.

Goal:

- avoid pixel-only targets except for explicitly “visual correctness” tests.

### Layer 3: Deterministic runner semantics

Make playback deterministic without warping the OS:

- isolate external (non-script) pointer/keyboard input during scripted runs,
- capability-gate advanced runner features (cursor overrides, screenshots),
- make “fast mode” explicit (see below).

Multi-window / docking / multi-viewport specific notes:

- deterministic routing requires stable window identity (not “whatever is focused now”),
- scripts need first-class window targeting and cursor model overrides to drive hover routing,
- evidence must explain “which window received the input and why” (otherwise failures look like random timeouts).

### Layer 4: Script authoring ergonomics (agent-native DSL)

This is where ImGui’s `SetRef(...)` is instructive: long scripts need structure.

Missing building blocks (planned):

- **named references / scopes** (e.g. “set base selector to this window/panel”),
- script variables (e.g. reuse a picked selector across steps),
- first-class “intent steps” with clear preconditions (ensure visible, click stable, menu select, drag-to).

This layer is primarily about **reducing diff noise and flake surface**, not adding more steps.

Status (2026-02-28):

- Base reference scoping exists in schema v2 scripts via `set_base_ref` / `clear_base_ref`.
- Bounded docking/multi-window evidence sidecars exist for bundle export dirs:
  - `window.map.json` + `diag windows` (window ids + last bounds + hover detection),
  - `dock.routing.json` + `diag dock-routing` (bounded routing evidence for docking/tear-out).

### Layer 5: Human GUI (DevTools) + agent integration

We already have a DevTools direction (`apps/fret-devtools`, MCP variants), but it should be **built on top of the same
contracts** (bundles, scripts, capabilities, sessions) rather than inventing new ones.

Key principle:

- real-time UI is for iteration speed,
- bundles remain the portable “source of truth”.

Related:

- `docs/workstreams/diag-devtools-gui-v1.md`

## Gaps vs ImGui Test Engine (summary)

This is intentionally outcome-focused:

- Headless: Fret is not yet “headless-first” for diag suites.
- Fast mode semantics: we have cursor overrides + isolation, but no single “fast mode policy” contract.
- Video/GIF capture: we have screenshots; time-series capture is missing.
- Named references/scopes: we have selectors; scoping/naming is missing.
- Multi-viewport evidence: we now have a window map + bounded routing logs, but still need richer per-frame probes
  (e.g. viewport capture, drop candidate summaries, explicit target window selection traces).
- App-level probes: we can test via UI semantics; a structured “app probe surface” is still immature.

## Development plan (fearless refactor)

We explicitly sequence work so we don’t block ongoing diag core progress or devtools UI work.

### P1 (core): agent-native script ergonomics

1) Named references / scopes:
   - equivalent of `SetRef(...)` but semantics-first (selector + optional window).
2) Fast mode policy:
   - config-driven execution policy (stabilization defaults, animation handling, teleport eligibility).
3) Headless seed:
   - one minimal headless target and a single smoke suite that runs without a visible window.

### P2 (evidence): time-series visual evidence

Add an opt-in export for time-series evidence:

- bounded PNG sequences for N frames around failures,
- optional GIF encoding as a post-process (tooling-side).

### P3 (UX): devtools UI alignment

After P1/P2 stabilize, fold these into the DevTools GUI workstream:

- session browsing + cleanup,
- script studio with named references,
- “run → bundle → triage → share” end-to-end flows.

## Definition of done (for agent-ready debugging)

We consider the stack “agent-ready” when:

- parallel agents can run without stepping on each other (`--session-auto` is the happy path),
- the default triage loop never requires opening/grepping `bundle.json`,
- a flaky UI bug can be converted into a deterministic script with stable selectors and bounded evidence,
- the same script can be re-run in fast mode and (eventually) headless mode.

## Practical guidance (AI agents + humans)

If you are automating with multiple agents:

- treat `--dir` as a base bucket (agent-owned),
- use `--session-auto` for tool-launched runs so control-plane files never race,
- do not rely on a global `latest.txt` outside a session (use `diag list sessions` and per-run manifests instead).

If you are debugging docking/multi-viewport:

- require window targeting in scripts (avoid “current focused window” ambiguity),
- prefer internal cursor overrides over OS cursor warping,
- capture a small number of screenshots at key moments; add time-series capture only when needed.
