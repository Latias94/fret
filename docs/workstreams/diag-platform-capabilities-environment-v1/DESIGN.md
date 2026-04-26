# Diagnostics Platform Capabilities Environment Admission

Status: closed narrow follow-on

## Purpose

The docking multi-window parity lane needs a Wayland-only real-host campaign. Running that script on
Windows or X11 should not produce a timeout that looks like a docking failure. It should produce a
policy skip before the script runs.

The existing diagnostics environment contract already separates `requires_environment` from
diagnostics transport capabilities. This follow-on adds a second source-specific admission shape:

- `source_id: "platform.capabilities"`
- `predicate.kind: "platform_capabilities"`

## Boundary

This is not a generic environment expression language.

The source is launch-time runner platform posture, published by `fret-bootstrap` from the existing
`PlatformCapabilities` global. The predicate supports only exact expectations needed by current
campaign scheduling:

- `platform_is`
- `ui_multi_window_is`
- `ui_window_tear_off_is`
- `ui_window_hover_detection_is`
- `ui_window_z_level_is`

That keeps the grammar source-scoped and avoids reopening the closed first-source lane.

## Consumer

`tools/diag-campaigns/imui-p3-wayland-real-host.json` uses this source to admit only Linux hosts
whose runner reports the Wayland-safe docking posture:

- `ui.multi_window=true`
- `ui.window_tear_off=false`
- `ui.window_hover_detection=none`
- `ui.window_z_level=none`

The direct script remains the proof surface; the campaign wrapper owns host admission.
