# fret-bootstrap

Opinionated bootstrap utilities for Fret applications.

`fret-bootstrap` is an ecosystem-level crate that composes `fret-launch`, `fret-app`, and related
runtime services into a practical desktop-first startup story.

Related workstream: `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/`

## Choosing an entry path

- App authors: `ui_app(...)` / `ui_app_with_hooks(...)`
- Advanced low-level integration with bootstrap defaults: `BootstrapBuilder::new_fn(...)`
- Compatibility/generic driver integration: `BootstrapBuilder::new(...)`

Recommended mental model:

- `ui_app(...)` is the primary author-facing path.
- `BootstrapBuilder::new_fn(...)` is the recommended advanced escape hatch.
- `BootstrapBuilder::new(...)` is for existing `WinitAppDriver` integrations or callers that
  already hold a fully built driver value.

## Surface map

- `ui_app(...)` / `ui_app_with_hooks(...)`
  - wraps `ui_app_driver::UiAppDriver`
  - converts it into `fret_launch::FnDriver`
  - applies bootstrap conveniences through `BootstrapBuilder`
- `BootstrapBuilder::new_fn(...)`
  - builds `fret_launch::FnDriver` from function pointers
  - keeps the hotpatch-friendly advanced path explicit
- `BootstrapBuilder::new(...)`
  - accepts any low-level driver implementing `fret_launch::WinitAppDriver`
  - keeps compatibility/generic integration available without making it the default story

## Relationship to `fret`

- `fret` is the batteries-included author-facing facade.
- `fret-bootstrap` is the recommended manual-assembly layer when callers still want bootstrap
  defaults but need more control over driver wiring.
- `fret-launch` remains the lower-level runner facade for advanced integration details.

## Command palette boundary

- `fret-bootstrap/ui-app-command-palette` owns the driver-level capability:
  - command dispatch handling,
  - per-window open/query models,
  - command gating snapshots.
- `fret-bootstrap/ui-app-command-palette-shadcn` adds the default shadcn `CommandDialog` overlay
  on top of that capability.
- Custom design systems should stay on `ui-app-command-palette` and supply an app-owned overlay
  renderer.

## Diagnostics boundary

- `fret-bootstrap/diagnostics` owns the diagnostics service and bundle/script orchestration.
- retained canvas cache exporters are opt-in on `fret-bootstrap/diagnostics-canvas`.
- `fret-bootstrap/diagnostics-ws` is the explicit opt-in lane for the devtools WebSocket
  transport.

## Minimal examples

- General UI app: see `ecosystem/fret/src/lib.rs`
- Advanced `FnDriver` escape hatch: `ecosystem/fret-bootstrap/examples/fn_driver_escape_hatch.rs`
