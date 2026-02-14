# Hotpatch Devloop Alignment v1 — TODO Tracker

Status: Draft (workstream tracker)

This document tracks TODOs for:

- `docs/workstreams/hotpatch-devloop-alignment-v1.md`
- `docs/workstreams/hotpatch-devloop-alignment-v1-milestones.md`

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `HP-DL-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- file paths + key functions/tests
- and/or a `fretboard diag` script/suite name

## M0 — Baseline UX + status

- [ ] HP-DL-ux-001 Make `--hotpatch` the single recommended flag (document + examples).
  - Evidence: `apps/fretboard/src/dev.rs`, `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md`

- [ ] HP-DL-ux-002 Print a stable “Hotpatch Summary” block at startup.
  - Includes: mode, dx availability, ws endpoint (if any), build id policy, trigger path, view-call strategy.
  - Evidence: `apps/fretboard/src/dev.rs`

- [ ] HP-DL-obs-001 Standardize log locations and make them discoverable from `fretboard`.
  - Runner log: `.fret/hotpatch_runner.log`
  - Bootstrap/view log: `.fret/hotpatch_bootstrap.log`
  - Evidence: `crates/fret-launch/.../hotpatch.rs`, `ecosystem/fret-bootstrap/.../ui_app_driver.rs`

- [ ] HP-DL-obs-002 Add an optional `fretboard hotpatch status` command (read-only).
  - Minimal version: print paths + tail last N lines if present.
  - Evidence: `apps/fretboard/src/hotpatch.rs`

## M1 — Predictable fallback behavior

- [ ] HP-DL-fb-001 Define a single fallback ladder and make it explicit in logs.
  - Evidence: `docs/workstreams/hotpatch-devloop-alignment-v1.md` (section 4.3)

- [ ] HP-DL-win-001 Replace “memorize env var” with a supervised recommendation for Windows view crash cases.
  - Evidence: `apps/fretboard/src/dev.rs` startup summary, ADR 0105 known issue text.

- [ ] HP-DL-win-002 Add a fast “restart suggestion” UX when repeated crashes are detected.
  - Minimal: print actionable instructions with the exact command line.
  - Stretch: `fretboard` as a supervisor that relaunches automatically (time-boxed).

## M2 — No-compile: theme reload

- [ ] HP-DL-theme-001 Define a theme reload contract (data source, watcher, safe apply boundary).
  - Evidence: new ADR or workstream appendix.

- [ ] HP-DL-theme-002 Implement a minimal theme reload loop for one demo (proof of concept).
  - Evidence: demo path + before/after screenshot or diag script.

## M3 — No-compile: asset reload

- [ ] HP-DL-asset-001 Asset invalidation + redraw contract (svg/png/fonts).
- [ ] HP-DL-asset-002 Implement for one asset type (svg or png) and validate.

## M4 — No-compile: hot literals

- [ ] HP-DL-lit-001 Define a hot-literals file format and resolution precedence.
- [ ] HP-DL-lit-002 Implement for one demo and validate.

