---
title: App Iteration Fast Restart + State Restore (v1)
status: in_progress
date: 2026-02-15
scope: app authoring devloop (native-first; wasm follow-up)
---

# App Iteration Fast Restart + State Restore (v1) — Workstream

This workstream targets the **app authoring** inner loop: “edit → observe”.

Bottom line:

- Default posture: **rebuild + restart** (watch mode), with **best-effort state restore**.
- Optional accelerator: Subsecond hotpatch (covered by `docs/workstreams/hotpatch-devloop-alignment-v1/hotpatch-devloop-alignment-v1.md`).
- High-frequency UI tweaks should prefer **no-compile reload channels** (theme/assets/literals), not Rust patching.

The goal is a workflow that feels “hot” while staying conservative and portable.

## Current status (2026-02-15)

Implemented on branch `ws/app-iteration-fast-restart-v1` (worktree); not merged to `main` yet.

- Dev-state file: `.fret/dev_state.json` (v1, versioned, forward-compatible, atomic writes).
- Window geometry restore:
  - `main` window restore
  - multi-window restore by stable keys (e.g. `main`, `floating-1`, `aux:<id>`), including app-registered keys.
  - off-screen restored positions are dropped (best-effort clamp-by-drop).
- Docking layout restore:
  - `docking_demo` and `docking_arbitration_demo` persist/restore layout via dev-state hooks.
- App-owned state (opt-in):
  - `todo_demo` restores a small snapshot via hooks.
- Restart hygiene:
  - flush dev-state on window close / quit
  - watch-mode restarts request a graceful exit before killing the process

## Goals

1. Make rebuild+restart feel like hot reload:
   - restart is fast and predictable,
   - the app comes back in the “same place” (window/docking/navigation).
2. Define a minimal, stable **dev-state contract**:
   - what is captured,
   - where it is stored,
   - how it is restored,
   - and how it degrades when incompatible.
3. Keep all devloop machinery out of kernel crates.

## Non-goals (v1)

- In-process “full Rust code hot reload” without restrictions.
- ABI-stable plugin systems for production.
- Automatic state migration across incompatible versions of app state.

## Contract anchors

- Hotpatch + safety boundary rules: `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- Golden-path driver (recommended app surface): `docs/adr/0110-golden-path-ui-app-driver-and-pipelines.md`
- Workstream for Subsecond UX + no-compile channels: `docs/workstreams/hotpatch-devloop-alignment-v1/hotpatch-devloop-alignment-v1.md`

## Target UX (native)

Recommended default (no Subsecond required):

- `fretboard dev native --bin <app> --watch`

Build note:

- Dev-state is a dev-only feature gate. In external apps, enable `fret` feature `devloop`
  (or `fret-launch` feature `dev-state`) for state restore support.

Optional (Subsecond + fallback ladder):

- `fretboard dev native --bin <app> --hotpatch`

User expectation after a code change:

- within ~1–2s (project dependent), the app restarts,
- windows re-open in the same geometry,
- docking layout is restored (if the app uses docking),
- the last “route/scene” is restored (best-effort),
- app models may be restored only if the app opts in.

## Decisions (2026-02-15)

These choices are intentionally conservative and optimized for app authoring iteration speed.

- Acceptance carrier demos:
  - Baseline + model restore sample: `apps/fret-demo/src/bin/todo_demo.rs`
  - Docking restore sample: `apps/fret-demo/src/bin/docking_demo.rs`
- Scope ordering:
  - M1 window geometry
  - M2 docking layout
  - M3 opt-in app model restore
  - M4 multi-window restore (best-effort)
- Snapshot write policy:
  - write on important transitions (window move/resize end, docking commit, route change)
  - plus a 250–500ms debounce fallback to coalesce bursts
- “Start clean” escape hatch:
  - `fretboard` flag (preferred) + env var (scriptable); both supported

## Dev-state model (v1)

We treat dev-state as two layers:

1. **Runner-owned state** (framework-controlled; always on in dev)
   - windows: geometry + presentation hints
   - docking: layout snapshot (ecosystem docking)
   - navigation: last view/route key (if available)

2. **App-owned state** (opt-in; app-defined)
   - model snapshot(s) that are cheap and safe to deserialize
   - never required for a correct restart

Principles:

- Restore is **best-effort**: invalid/incompatible state must be ignored with a clear log line.
- Writes are **atomic**: write temp, then replace.
- File format is **versioned**.

### Storage location

Default location (workspace-relative):

- `.fret/dev_state.json`

Optional per-binary split (if needed later):

- `.fret/dev_state/<bin>.json`

## Integration sketch (where code should live)

This workstream should remain entirely in glue + ecosystem layers:

- `apps/fretboard` (supervisor; restart loop UX)
- `crates/fret-launch` (runner integration points; window lifecycle)
- `ecosystem/fret-bootstrap` (golden-path driver hooks; optional helpers)
- `ecosystem/fret-docking` (docking snapshot/restore contract)

Kernel crates must remain free of dev-only policy.

## API shape (v1 sketch)

The golden path should expose two optional hooks (function-pointer friendly):

- `dev_state_export(app, window, state) -> serde_json::Value`
- `dev_state_import(app, window, state, value) -> Result<(), DevStateError>`

Runner-owned state is handled automatically by the runner/driver.

Notes:

- Export/import should run at a **safe frame boundary** (not mid-event dispatch).
- Import must never assume the old code still exists (restart-only path is the baseline).

## Safety & failure modes

1. **Schema drift**
   - Version all files and support “ignore unknown fields”.
2. **Docking compatibility**
   - Docking layout restore should be guarded by a layout version and a stable “panel key” contract.
3. **Multi-window mapping**
   - Prefer “window role keys” (e.g. `main`, `secondary:<id>`) over transient runtime IDs.
4. **Crash loops**
   - Supervisor should detect repeated crashes and offer an easy “start clean” escape hatch:
     - ignore dev-state for one run (e.g. `FRET_DEV_STATE_RESET=1`)
     - or use `fretboard dev native --dev-state-reset` (clears the dev-state file).

## Deliverables

- A stateful restart path that is “good enough” for day-to-day app iteration:
  - window geometry restore
  - docking layout restore (if docking is used)
  - optional app state hooks (opt-in)
- A small set of evidence anchors (tests + a demo repro recipe).

Tracking:

- TODO list: `docs/workstreams/app-iteration-fast-restart-v1/app-iteration-fast-restart-v1-todo.md`
- Milestones: `docs/workstreams/app-iteration-fast-restart-v1/app-iteration-fast-restart-v1-milestones.md`
