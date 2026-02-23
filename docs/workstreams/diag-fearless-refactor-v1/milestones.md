---
title: Diagnostics Fearless Refactor v1 (Milestones)
status: draft
date: 2026-02-23
scope: diagnostics, automation, tooling, refactor
---

# Diagnostics Fearless Refactor v1 (Milestones)

## Milestone 1: Mechanical modularization is landed

Exit criteria:

- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` no longer contains the full per-frame script driver implementation.
- Script engine code (driver + helpers) lives in `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`.
- Script runner state types live in `ecosystem/fret-bootstrap/src/ui_diagnostics/script_types.rs`.
- `cargo check -p fret-ui-gallery` is green.

## Milestone 1b: CLI stats modularization is underway

Exit criteria:

- UI gallery checks are no longer all implemented inside `crates/fret-diag/src/stats.rs`.
- Extracted modules exist for large check families:
  - `crates/fret-diag/src/stats/ui_gallery_markdown_editor.rs`
  - `crates/fret-diag/src/stats/ui_gallery_code_editor.rs`
  - `crates/fret-diag/src/stats/debug_stats_gates.rs`
  - `crates/fret-diag/src/stats/interaction_gates.rs`
  - `crates/fret-diag/src/stats/gc_gates.rs`
  - `crates/fret-diag/src/stats/notify_gates.rs`
  - `crates/fret-diag/src/stats/overlay_gates.rs`
  - `crates/fret-diag/src/stats/retained_vlist_gates.rs`
  - `crates/fret-diag/src/stats/view_cache_gates.rs`
- The `diag perf` subcommand handler is no longer implemented inline in `crates/fret-diag/src/lib.rs`.
  - Evidence: `crates/fret-diag/src/diag_perf.rs`
- The `diag compare` subcommand handler is no longer implemented inline in `crates/fret-diag/src/lib.rs`.
  - Evidence: `crates/fret-diag/src/diag_compare.rs`
- The `diag stats` subcommand handler is no longer implemented inline in `crates/fret-diag/src/lib.rs`.
  - Evidence: `crates/fret-diag/src/diag_stats.rs`
- The `diag matrix` subcommand handler is no longer implemented inline in `crates/fret-diag/src/lib.rs`.
  - Evidence: `crates/fret-diag/src/diag_matrix.rs`
- The `diag repeat` subcommand handler is no longer implemented inline in `crates/fret-diag/src/lib.rs`.
  - Evidence: `crates/fret-diag/src/diag_repeat.rs`
- The `diag repro` subcommand handler is no longer implemented inline in `crates/fret-diag/src/lib.rs`.
  - Evidence: `crates/fret-diag/src/diag_repro.rs`
- `cargo check -p fret-diag` is green.

## Milestone 2: Sidecar indexes reduce day-to-day pain

Exit criteria:

- Tools can locate relevant snapshots without opening full `bundle.json` in memory.
- Sidecars are documented and versioned.
- Missing sidecars degrade gracefully (no “hang until timeout”).
- A lightweight per-frame index exists for agentic triage:
  - `frames.index.json` (generated via `fretboard diag frames-index` and included in `diag doctor --fix-sidecars` / `diag ai-packet`).

## Milestone 3: Agent-friendly triage loop

Exit criteria:

- A maintainer can run a scripted repro, collect evidence, and generate a small “triage bundle” quickly.
- Huge-bundle first-pass triage does not require loading `bundle.json` into memory:
  - `fretboard diag triage --lite ...` works from `frames.index.json`.
  - `fretboard diag hotspots --lite ...` reports slow frames from `frames.index.json`.
- `fretboard diag ai-packet ...` includes lite reports (`triage.lite.json`, `hotspots.lite.json`) for agent-first workflows.
- A documented migration plan exists and stays in sync with implementation:
  - `docs/workstreams/diag-fearless-refactor-v1/migration-plan.md`
- Evidence anchors in docs stay in sync with the implementation.

## Milestone 4: Debt is removed (no redundant code paths)

Exit criteria:

- No “temporary forwarders” remain solely due to historical file layout (moved code stays moved).
- Monolith risk is reduced:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` is not the primary home for script engine logic.
  - `crates/fret-diag/src/stats.rs` stays small; large check families live in `crates/fret-diag/src/stats/*.rs`.
- Semantics parsing is centralized:
  - gates use `crate::json_bundle::SemanticsResolver` and shared helpers (no repeated JSON path digging).
- `cargo check -p fret-ui-gallery` and `cargo check -p fret-diag` are green.
