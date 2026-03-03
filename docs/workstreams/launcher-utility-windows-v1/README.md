# Launcher + Utility Windows v1

Status: In progress (M3)

## Context

We want to support an "app launcher"-class UX (Vicinae / uTools-like):

- a frameless utility window (custom chrome) that can be shown/hidden quickly,
- optional transparent / OS-material backgrounds (Mica/Acrylic/Vibrancy), capability-gated,
- predictable activation semantics (non-activating vs activating),
- always-on-top and skip-taskbar postures,
- no backend type leakage into portable crates.

This is a contract-heavy surface: getting it wrong tends to cause large rewrites. Therefore this
workstream follows the repo's ADR-driven approach.

## Current status (as of 2026-03-03)

- Contracts + portable plumbing are landed (M1).
- Desktop runner supports:
  - create-time facets: `decorations/resizable/transparent`,
  - runtime actions: `set_visible/begin_drag/begin_resize` (best-effort; capability-gated),
  - diagnostics predicates for effective window style/material, with fail-fast capability inference.
- MVP demo + scripted gate are landed (M2):
  - Demo: `launcher_utility_window_demo` (frameless main window, drag region, resize handle, blink).
  - Script: `tools/diag-scripts/launcher-utility-window-mvp.json`.
- OS background materials are still "request + clamp + diagnostics" only (M3 is not implemented).

## Related ADRs (decision gates)

- Window styles + utility windows: `docs/adr/0139-window-styles-and-utility-windows.md`
- Window background materials: `docs/adr/0310-window-background-materials-v1.md`
- Window chrome actions + visibility: `docs/adr/0311-window-chrome-actions-and-visibility-v1.md`

## Diagnostics strategy (why this is feasible)

Fret already has a strong diagnostics + scripted automation pipeline (`fret-bootstrap` + `fret-diag`
tooling + `fret-diag-protocol` schemas). For this workstream, we treat "effective/clamped window
style/material" as a first-class diagnostics surface so we can add non-pixel regression gates.

The guiding rule is: make missing support fail fast via explicit `diag.*` capabilities (no "hang
until timeout").

## Documents

- TODO list: `docs/workstreams/launcher-utility-windows-v1/todo.md`
- Milestones: `docs/workstreams/launcher-utility-windows-v1/milestones.md`
- Execution plan: `docs/workstreams/launcher-utility-windows-v1/plan.md`

## Goals

1. Make utility-window contracts complete and internally consistent (ADRs + capability keys).
2. Land a minimal, testable end-to-end path on desktop runner(s) (initially Windows/macOS best-effort).
3. Provide observability: effective/clamped style and material results are inspectable (ADR 0036).
4. Keep policy (where to drag/resize, dismissal rules, focus traps) in `ecosystem/*`.

## Non-goals (v1)

- Global hotkeys and tray integration (tracked as follow-up once window contracts are stable).
- Shaped windows / per-pixel hit testing.
- Pixel-perfect parity of OS materials across versions/themes.

## Acceptance criteria

- Portable contract exists for:
  - `WindowStyleRequest` including `decorations/resizable/transparent` (ADR 0139),
  - background materials request (ADR 0310),
  - chrome actions and show/hide (ADR 0311).
- Capability gating exists for each facet; ecosystem can gate via `when` without `cfg(target_os)`.
- Runner exposes effective/clamped style/material results to diagnostics/inspection.
  - Note: v1 currently exposes "effective (post-clamp)" snapshots; requested/base snapshots are a
    follow-up once the platform implementations are stable.
