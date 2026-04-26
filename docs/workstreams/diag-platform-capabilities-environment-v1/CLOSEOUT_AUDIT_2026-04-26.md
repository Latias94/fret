# Closeout Audit - 2026-04-26

Status: Closed

## Result

This follow-on adds the second admitted diagnostics environment source without changing the
first-source contract into a generic expression system.

The shipped source is:

- `source_id: "platform.capabilities"`
- payload file: `environment.source.platform.capabilities.json`
- predicate kind: `platform_capabilities`

The predicate is intentionally exact-match only. It supports the platform and the few UI capability
facets needed to schedule the Wayland docking acceptance campaign honestly.

## Why This Is Closed

The motivating consumer is `imui-p3-wayland-real-host`. That campaign can now skip non-Wayland
hosts through campaign environment admission and keep the direct script focused on the actual
degradation proof.

Remaining real-host acceptance is still owned by
`docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
This follow-on only provides the admission mechanism.

## Non-Goals

- No generic boolean expression grammar.
- No attempt to promote debug snapshots into preflight sources.
- No claim that Wayland compositor acceptance has passed.
- No widening of `requires_capabilities`; diagnostics capabilities remain separate from host
  environment admission.
