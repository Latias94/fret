# Milestones

Status: Active
Last updated: 2026-06-21

## M0 - Baseline Read

Goal: separate real pointer-move cost from mixed interaction frames.

Exit evidence:

- Overlay pointer-move steady run passes the initial gate.
- Bundle stats show pointer-move dispatch and hit-test max values separately.
- Source audit confirms pointer move does not synchronously publish full command availability.

## M1 - Attribution Surface

Goal: make slow command availability publication explainable.

Exit evidence:

- Runtime records command, route, start/resolved node, start/resolved element, outcome, and elapsed
  time.
- Debug snapshots include `command_availability_hotspots`.
- `diag stats` human and JSON output surface the same attribution.
- Focused unit coverage proves the stats projection.

## M2 - Gate Closure

Goal: make the contract reusable by future refactors.

Exit evidence:

- Changed-package tests pass.
- Overlay pointer-move perf gate passes.
- Workstream evidence names the latest bundle and stats command.
- Any missing direct runtime test is either added or tracked as a follow-up with a named owner
  surface.
- Same-frame repeated `focus_traversal_snapshot` publication is coalesced by a frame-level cache and
  covered by a focused `fret-ui` test.

## M3 - Hardening Follow-Up

Goal: turn attribution into an explicit failure mode.

Exit evidence:

- A gate fails if command availability time is above 500us without hotspots.
- Route names are documented as diagnostics contract values.
- Any double-resolution overhead in diagnostic-only fallback root labeling is removed or proven
  negligible.
- The inspector direct-entry perf path keeps focus on the stable list root after row activation, and
  the latest bundle shows zero `subtree_no_focus_fallback` hotspots on that probe.
