# App Iteration Fast Restart + State Restore (v1) — TODO Tracker

Status: Draft (workstream tracker)

This TODO list is ordered by **cost-effectiveness** for app authoring iteration speed.

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `FR-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- file paths + key functions/tests
- and/or a `fretboard-dev diag` script/suite name

## P0 — High impact, low risk (default path)

- [x] FR-contract-001 Define `.fret/dev_state.json` file format and versioning rules.
  - Must be forward compatible: ignore unknown fields.
  - Must be crash-safe: atomic write + replace.
  - Suggested validation target: `apps/fret-demo/src/bin/todo_demo.rs`
  - Evidence:
    - `crates/fret-launch/src/runner/desktop/runner/dev_state.rs`

- [x] FR-runner-001 Capture and restore window geometry (single-window baseline).
  - Evidence: a demo restarts into the same window size/position.
  - Suggested carrier: `apps/fret-demo/src/bin/todo_demo.rs`
  - Evidence:
    - `crates/fret-launch/src/runner/desktop/runner/dev_state.rs`
    - `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`

- [x] FR-runner-002 Add a stable “window role key” concept for restore mapping.
  - Example: `main`, `aux:<name>`.
  - Must degrade gracefully when roles don’t match.
  - Evidence:
    - `crates/fret-launch/src/runner/desktop/runner/dev_state.rs` (per-key map)
    - `crates/fret-launch/src/dev_state.rs` (`DevStateWindowKeyRegistry`)
    - `apps/fret-examples/src/docking_arbitration_demo.rs` (registers `floating-N`)

- [x] FR-supervisor-001 Make “restored dev state” visible in dev output.
  - One line at startup: path + version + restore outcome (ok/partial/ignored).
  - Evidence:
    - `crates/fret-launch/src/runner/desktop/runner/dev_state.rs` (stderr summary)

- [x] FR-escape-001 Add a “start clean” escape hatch.
  - Both supported:
    - CLI: `fretboard-dev dev native ... --dev-state-reset`
    - Env: `FRET_DEV_STATE_RESET=1`
  - Evidence:
    - `apps/fretboard/src/dev.rs`
    - `crates/fret-launch/src/runner/desktop/runner/dev_state.rs`

## P1 — Medium cost, high leverage for editor-grade apps

- [x] FR-docking-001 Define docking layout snapshot/restore contract (panel key stability).
  - Must live in ecosystem docking layer (policy), not `crates/fret-ui`.
  - Evidence:
    - `crates/fret-core/src/dock/persistence.rs` (layout export/import primitives)
    - `apps/fret-examples/src/docking_demo.rs`

- [x] FR-docking-002 Implement docking layout restore for one docking-heavy demo.
  - Evidence: restart retains split layout and active tab.
  - Suggested carrier: `apps/fret-demo/src/bin/docking_demo.rs`
  - Evidence:
    - `apps/fret-examples/src/docking_demo.rs`
    - `apps/fret-examples/src/docking_arbitration_demo.rs`

- [x] FR-app-001 Add opt-in app-owned state hooks (export/import).
  - Default: off.
  - Import must be fallible and never block startup.
  - Evidence:
    - `crates/fret-launch/src/dev_state.rs` (`DevStateHook`, `DevStateHooks`, `DevStateExport`)

- [x] FR-app-002 Provide a sample: retain a small model across restarts.
  - Example: a todo list demo retains items and selection.
  - Suggested carrier: `apps/fret-demo/src/bin/todo_demo.rs`
  - Evidence:
    - `apps/fret-examples/src/todo_demo.rs`

## P2 — Multi-window & robustness

- [x] FR-mw-001 Restore multi-window sets (best-effort).
  - Restore order is not guaranteed; roles must drive mapping.
  - Evidence:
    - `apps/fret-examples/src/docking_arbitration_demo.rs` (creates windows from layout)
    - `crates/fret-launch/src/runner/desktop/runner/dev_state.rs` (per-window geometry by key)

- [ ] FR-obs-001 Standardize a lightweight log path for dev-state restore.
  - Example: `.fret/dev_state.log` (or reuse an existing devloop log).

- [x] FR-perf-001 Debounce snapshots (avoid writing on every frame).
  - Define a clear “when we flush” policy (timer-based + on important transitions).
  - Evidence:
    - `crates/fret-launch/src/runner/desktop/runner/dev_state.rs` (poll + debounce + atomic flush)
    - `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` (flush on close)
    - `apps/fretboard/src/dev.rs` (watch-mode forces debounce=0)

## P3 — Long-term accelerators (optional; large change)

- [ ] FR-build-001 Explore “dynamic linking for faster relink” guidance (Bevy-style).
  - Target: reduce rebuild+restart latency for large apps.
  - Must remain optional and well-documented.

- [ ] FR-plugin-001 Evaluate a dev-only plugin boundary (C ABI / wasm module) for policy-layer iteration.
  - Explicitly not required for v1 success.
  - Must come with a strict “stable host API” story.
