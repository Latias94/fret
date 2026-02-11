# ADR 0173: Window Command Enabled Service (Per-Command Overrides)

Status: Proposed

## Context

OS menubars need a synchronous, best-effort enable/disable signal for menu items (e.g. macOS
`validateMenuItem:` and Windows `WM_INITMENUPOPUP` refresh).

Fret already has:

- `WindowInputContextService` (UI runtime publishes a window-scoped `InputContext` snapshot for
  runner gating, including focus/modal and input arbitration state).
- `WindowCommandAvailabilityService` (v1 seam for Undo/Redo availability, because it is not always
  derivable from focus/modal alone).

As we build editor-grade workflows (docking, multi-window, multiple viewports, tool overlays),
more commands have enablement that is:

- window-scoped,
- dynamic,
- not expressible as a pure `when` expression over focus/modal state, and
- must be consistent across surfaces (OS menubar, in-window menubar, command palette, shortcuts).

We want a data-only seam that:

- does not depend on `fret-ui-kit` or app model types,
- does not inflate the `InputContext` contract surface with app-specific fields, and
- allows apps to publish a minimal snapshot that runners and UI surfaces can consult.

## Decision

Introduce `WindowCommandEnabledService` in `fret-runtime`:

- Keyed by `(AppWindowId, CommandId)`.
- Value is `bool` (enabled/disabled).
- The service is optional; absence implies "no override".

Usage:

- OS menubar gating MAY treat `Some(false)` as disabled even if `when` would otherwise evaluate to
  true.
- Shortcut dispatch MUST respect `Some(false)` so keyboard behavior matches menu enablement.
- App effect enqueue MUST drop `Effect::Command { window: Some(_), .. }` when the override is
  `Some(false)` as a final guardrail against surface inconsistencies.

This service is window-scoped (not viewport-scoped). Docking and multiple viewports do not change
the seam: viewports are app/UI details; the OS menubar is a window-level integration surface.

## Consequences

### Positive

- Adds a minimal, data-only seam for "hard-to-infer" enablement.
- Keeps `InputContext` free of app-specific fields.
- Keeps `fret-ui` as a mechanism/contract layer (no shadcn policy leakage).
- Makes OS menubar and shortcuts consistent by default.

### Trade-offs

- Apps must publish snapshots; there is no automatic discovery.
- `Some(true)` is not a "force enable" guarantee; other gating (e.g. `when`) can still disable.

## Evidence anchors (implementation)

- Service: `crates/fret-runtime/src/window_command_enabled.rs`
- App-level guardrail: `crates/fret-app/src/app.rs`
- Shortcut gating: `crates/fret-ui/src/tree/shortcuts.rs`
- Windows OS menubar gating: `crates/fret-launch/src/runner/desktop/windows_menu.rs`
- macOS OS menubar gating: `crates/fret-launch/src/runner/desktop/macos_menu.rs`
- In-window menubar gating: `ecosystem/fret-kit/src/workspace_menu.rs`
- Command palette gating: `ecosystem/fret-ui-shadcn/src/command.rs`
- Tests: `crates/fret-ui/src/tree/tests/command_enabled_service.rs`

## References

- ADR 0168: `docs/adr/0168-os-menubar-effect-setmenubar.md`
- ADR 0022: `docs/adr/0022-when-expressions.md`
- ADR 0066: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
