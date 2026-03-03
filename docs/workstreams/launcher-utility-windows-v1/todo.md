# Launcher + Utility Windows v1 — TODO

Status: In progress (M3)

## Completed (M0/M1)

- [x] ADR 0139 aligned (create-time vs runtime patchability).
- [x] Background material vocabulary + capability keys (ADR 0310).
- [x] Chrome actions + visibility vocabulary + capability keys (ADR 0311).
- [x] `fret_runtime::WindowStyleRequest` extended with v1 facets (`decorations/resizable/transparent/background_material`).
- [x] `PlatformCapabilities` extended with `ui.window.*` facet keys and chrome-action keys.
- [x] Diagnostics surface for effective/clamped window style + material:
  - [x] runner records an effective snapshot per window (post-clamp),
  - [x] diag predicates exist for scripted gating,
  - [x] `fret-diag` infers required capabilities from these predicates (fail-fast).

## Completed (M2) — Desktop runner MVP (frameless utility window)

- [x] Land a minimal "utility window" demo:
  - [x] open a frameless window via `WindowStyleRequest { decorations: None, .. }`,
  - [x] implement a draggable region (policy in `ecosystem/*`) that calls `BeginDrag`,
  - [x] implement resize handles that call `BeginResize { direction }`,
  - [x] toggle `SetVisible` without closing/recreating the window.
- [x] Land a diag script (schema v2) that gates the demo:
  - [x] asserts effective decorations/resizable/transparent,
  - [x] asserts non-destructive show/hide (window count stable + style snapshot still present),
  - [x] captures a diagnostics bundle for future triage.

Evidence anchors:

- Demo: `apps/fret-examples/src/launcher_utility_window_demo.rs`
- Script: `tools/diag-scripts/launcher-utility-window-mvp.json`

## Next (M3) — OS materials (Windows/macOS best-effort)

- [ ] Windows: implement DWM system backdrop mapping (Mica/Acrylic):
  - [ ] define backend mapping and version gating (Win11+ where applicable),
  - [ ] update capabilities to truthfully advertise supported variants,
  - [ ] ensure transparency + backdrop interaction is deterministic and recorded.
- [ ] macOS: implement Vibrancy mapping:
  - [ ] pick the minimal stable API surface (titlebar/toolbar interactions),
  - [ ] capability gating + diagnostics evidence.
- [ ] Linux: explicitly clamp to `None/SystemDefault` until we have a stable story (Wayland/X11).

## Next (M4) — Observability hardening

- [ ] Upgrade window-style/material diagnostics snapshot to include:
  - [ ] requested/base style facets,
  - [ ] effective/clamped results,
  - [ ] explicit clamp reasons (optional, best-effort).
- [ ] Add an inspection pane in `fretboard` or UI gallery to render the snapshot (non-scripted).
- [ ] Add a "capability truth table" report in `fret-diag` for window style/material facets.

## Follow-ups (post-v1 decision gates)

- [ ] ADR: global hotkeys contract (register/unregister, conflict handling, security constraints).
- [ ] ADR: system tray/menu bar integration contract.
- [ ] Runner "agent mode" / keepalive policy when zero windows remain.
