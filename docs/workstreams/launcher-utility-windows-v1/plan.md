# Launcher + Utility Windows v1 — Execution Plan

This plan is ordered by "hard-to-change first" (contracts + observability) and keeps interaction
policy in `ecosystem/*`.

## Phase 0 (done) — Contracts + MVP gate

- ADRs: 0139 + 0310 + 0311 are the current decision gates.
- MVP demo: `launcher_utility_window_demo`.
- MVP scripted gate: `tools/diag-scripts/launcher-utility-window-mvp.json`.

## Phase 1 (M3) — Background materials implementation (best-effort)

### Windows (DWM system backdrop)

1. Implement material mapping in the native/winit runner backend:
   - map requested variants to the platform API (and clamp when unsupported),
   - handle Windows version gating (Win11 where applicable),
   - define deterministic interaction rules for `transparent` + backdrop.
2. Capabilities:
   - advertise supported variants truthfully (no optimistic defaults),
   - keep a single source of truth for "what is supported on this machine".
3. Diagnostics:
   - record an "effective material" snapshot per window (post-clamp),
   - (optional) add clamp reasons once the mapping is stable.
4. Regression gates:
   - add a diag script that requests materials and asserts the effective snapshot,
   - capture a bundle for each supported/unsupported path.

### macOS (Vibrancy)

Repeat the same flow as Windows:

- minimal stable API surface first,
- clamp + capability-gate + diagnostics,
- scripted gates for supported/unsupported paths.

### Linux

Keep clamping explicit until we have a stable Wayland/X11 story.

## Phase 2 (M4) — Observability hardening

1. Snapshot evolution:
   - include requested/base facets vs effective facets,
   - add a structured "clamp reasons" vocabulary (best-effort; never relied on for correctness).
2. Human-facing inspection:
   - add a small inspection pane in `fretboard` (or UI gallery) to render the snapshot.
3. `fret-diag` improvements:
   - a "capability truth table" report for window facets (what is supported, and why),
   - keep fail-fast capability inference (prefer early, explicit errors over timeouts).

## Phase 3 — Ecosystem policy for launcher UX (out of core)

Implement these in `ecosystem/*` (not `crates/fret-ui`):

- drag region + resize handles recipes,
- dismissal rules (escape/outside-click), focus trap/restore,
- global hotkey / tray integration (requires follow-up ADRs),
- command palette and search UX.

## ADR candidates (follow-ups)

Only add new ADRs when we have concrete outcomes to lock:

- activation semantics (non-activating vs activating, focus/raise rules),
- taskbar/dock visibility + alt-tab inclusion,
- always-on-top and z-level interactions,
- global hotkeys and tray/menu bar integration.
