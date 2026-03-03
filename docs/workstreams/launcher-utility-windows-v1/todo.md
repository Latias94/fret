# Launcher + Utility Windows v1 — TODO

Status: Draft

## Docs alignment (contract first)

- [ ] Review ADR 0139 and ensure it matches implementation intent (create-time vs runtime patchability).
- [ ] Lock background material vocabulary + capability keys (ADR 0310).
- [ ] Lock chrome action vocabulary + capability keys (ADR 0311).

## Portable contract plumbing

- [ ] Extend `fret_runtime::WindowStyleRequest` with missing facets:
  - [ ] `decorations`
  - [ ] `resizable`
  - [ ] `transparent`
  - [ ] `background_material` (ADR 0310)
- [ ] Extend `PlatformCapabilities` key set (ADR 0054):
  - [ ] `ui.window.*` style facet keys (ADR 0139)
  - [ ] `ui.window.background_material.*` (ADR 0310)
  - [ ] `ui.window.begin_drag` / `ui.window.begin_resize` / `ui.window.set_visible` (ADR 0311)
- [ ] Add a stable diagnostics surface for “effective/clamped window style” (ADR 0139, ADR 0036).

## Diagnostics + scripted gates (evidence-first)

- [ ] Add explicit diagnostics capabilities (fail-fast gating) in `capabilities.json`:
  - [ ] `diag.window_style_snapshot` (requested/base + effective/clamped, per window)
  - [ ] `diag.window_background_material_snapshot` (requested + effective/clamped, per window)
- [ ] Add script predicates for window-style assertions (capability-gated):
  - [ ] `window_style_effective_*` (decorations/resizable/transparent/taskbar/activation/z_level/mouse/opacity)
  - [ ] `window_background_material_effective_is`
- [ ] Add at least one small diag script (schema v2) that gates:
  - [ ] frameless utility window opens with expected effective style,
  - [ ] show/hide preserves state (no close/recreate),
  - [ ] background material request clamps deterministically when unsupported.

## Desktop runner wiring (best-effort, capability-gated)

- [ ] Create-time style application:
  - [ ] decorations / resizable / transparent
  - [ ] background materials (OS-backed)
- [ ] Runtime patching (`WindowRequest::SetStyle`) for patchable facets.
- [ ] Window chrome actions:
  - [ ] `BeginDrag`
  - [ ] `BeginResize`
  - [ ] `SetVisible`

## Validation and gates

- [ ] Add a demo (or scripted diag) that:
  - [ ] opens a frameless utility window,
  - [ ] allows drag/resize via custom chrome,
  - [ ] toggles show/hide without losing state,
  - [ ] reports effective/clamped style/material results.

## Follow-ups (post-v1 decision gates)

- [ ] ADR: global hotkeys contract (register/unregister, conflict handling, security constraints).
- [ ] ADR: system tray/menu bar integration contract.
- [ ] Runner “agent mode” / keepalive policy when zero windows remain.
