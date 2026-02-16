# A11y + AccessKit xplat bridge v1 — TODO

Status: Draft

Tracking doc: `docs/workstreams/a11y-accesskit-xplat-bridge-v1.md`

## 0) Audit follow-ups (excluding runner glue)

- [ ] Add additional semantics validation (portable, cheap):
  - [ ] `pos_in_set/set_size` invariants (when both present)
  - [ ] relation edges reference existing nodes
  - [ ] `active_descendant` references an existing node
- [ ] Document “large text value” policy for semantics snapshots (link to code editor workstream).

## 1) AccessKit upgrade (workspace)

- [ ] Bump `accesskit` workspace dep from `0.22` to `0.24.0`.
- [ ] Fix compilation fallout in:
  - [ ] `crates/fret-a11y-accesskit`
  - [ ] `crates/fret-runner-winit`
  - [ ] any downstream crates touching AccessKit types
- [ ] Run gates:
  - [ ] `cargo fmt`
  - [ ] `cargo nextest run -p fret-a11y-accesskit`

## 2) Implement internal xplat adapter (runner glue)

- [ ] Decide placement:
  - [ ] Implement inside `crates/fret-runner-winit` (recommended for v1).
- [ ] Add platform deps and wiring:
  - [ ] Windows: `accesskit_windows`
  - [ ] macOS: `accesskit_macos`
  - [ ] Linux/Unix: `accesskit_unix` (+ executor feature choice)
  - [ ] Android: `accesskit_android` (if we want parity now; otherwise explicitly defer)
  - [ ] `raw-window-handle` (if needed explicitly)
- [ ] Replace the current no-op implementation in `crates/fret-runner-winit/src/accessibility.rs` with a real one:
  - [ ] `take_activation_request` toggles on initial tree request
  - [ ] `is_active` tracks accessibility activation state
  - [ ] `process_event` forwards focus/bounds changes
  - [ ] `update_if_active` forwards `TreeUpdate` to AccessKit adapter
  - [ ] `drain_actions` yields `ActionRequest`s
- [ ] Keep the existing “invisible until adapter created” window lifecycle behavior in `fret-launch`.

## 3) Tests and manual acceptance

- [ ] Add at least one unit test in `fret-runner-winit` that:
  - [ ] exercises activation flow + action queue (without requiring OS adapters, if possible)
- [ ] Ensure existing AccessKit mapping tests still pass:
  - [ ] `cargo nextest run -p fret-a11y-accesskit`
- [ ] Manual acceptance checklist:
  - [ ] Run `docs/a11y-acceptance-checklist.md` on Windows
  - [ ] Run on one of macOS/Linux

## 4) Cleanup and hardening

- [ ] Decide fate of `crates/fret-runner-winit/src/accessibility_accesskit_winit.rs`:
  - [ ] keep as fallback, or
  - [ ] delete after xplat is verified
- [ ] Add a debug kill-switch:
  - [ ] env var to disable bridge (useful for triage)
- [ ] Update any docs referencing “disabled while on winit beta” if no longer true.

## 5) Gates (must stay green)

- [ ] `python3 tools/check_layering.py`
- [ ] `cargo nextest run -p fret-runner-winit`
- [ ] `cargo nextest run -p fret-a11y-accesskit`

