# IMUI Demo Metrics Debug Action Metadata v1

Status: Active
Last updated: 2026-05-30

## Problem

`demo-metrics-debug` is now a permanent first-open route across CLI, DevTools GUI, and MCP, but
the route's executable actions are still mostly text: a label and a command string. That is enough
for copy/paste, but it is not enough for richer GUI execution controls or a future shared command
palette because consumers would have to infer whether an action launches a demo, requires a
diagnostics bundle, or is a handoff gate by parsing human-facing strings.

## Target State

- `cargo run -p fretboard-dev -- list tool-apps --json` exposes stable action metadata for
  `demo-metrics-debug` route actions.
- DevTools GUI and MCP first-open text show the same metadata without changing the copyable command
  bundle shape.
- The route names this follow-on as the action metadata owner while keeping the original
  `imui-demo-metrics-debug-devtools-v1` route owner closed.
- Diagnostics gates reject drift between CLI JSON, DevTools GUI, MCP, and source markers.

## Scope

- In scope: `apps/fretboard/src/demos.rs`, `apps/fret-devtools/src/native.rs`,
  `apps/fret-devtools-mcp/src/native.rs`, and the existing first-open/product-chain gate scripts.
- In scope: route action fields `id`, `category`, `primary`, and `requires_bundle`.
- Out of scope: launching actions from a new command palette, a generalized DevTools command
  registry, new runtime APIs, and any `fret-imui` API widening.

## Boundary Rules

- `fret-imui` remains thin and does not own diagnostics UI.
- `fret-ui-kit::imui` remains the immediate-mode policy owner, not the DevTools route owner.
- DevTools and diagnostics own productization around evidence, bundles, and first-open routes.
- Docking real-host Wayland acceptance stays in `docking-multiwindow-imgui-parity`; local policy
  skips cannot replace that acceptance evidence.

## Assumptions

- The old route owner lane is closed and should not be reopened for richer execution controls.
- Action metadata can be additive in the existing `fretboard_tool_apps` JSON schema because it does
  not remove or rename existing fields.
- `requires_bundle` is the first useful split for GUI controls because metrics/debug drill-downs
  need a selected bundle while demo/product-gate actions do not.
- A future command palette should consume stable action IDs rather than derive IDs from labels.
