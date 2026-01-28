# ADR 0084: Multi-Window Degradation Policy (Logical Windows on Single-Window Platforms)

Status: Accepted

## Context

Fret’s docking model and persistence format support **multiple logical windows** (ADR 0013), and the
UI coordinate model treats multi-window + per-monitor DPI as first-class constraints (ADR 0017).

However, some target platforms cannot (or should not) create multiple OS windows:

- wasm (single canvas),
- mobile (single activity/surface),
- some sandboxed environments.

We already have a runtime capability model (ADR 0054) that can express this as:

- `ui.multi_window == false`

Without an explicit, stable degradation policy, docking + tear-off is likely to drift into
platform-specific forks and late-stage rewrites.

## Decision

When `PlatformCapabilities.ui.multi_window == false`, the host/runtime must adopt a single, explicit
behavior:

### 1) One OS window, many logical “window roots”

- The dock graph and persistence remain unchanged: layouts may contain multiple `DockLayoutWindow`
  entries (`logical_window_id`).
- The platform creates **only one OS window**.
- All non-main `DockLayoutWindow` roots are rendered inside the main OS window as **virtual window
  surfaces** (policy-layer composition), not as OS windows.

This keeps persistence portable and keeps the data model stable while allowing wasm/mobile to
execute the same logical layout.

### 2) Tear-off becomes in-window floating (policy), not `CreateWindow`

On single-window platforms:

- “tear-off to new window” UX is represented as **in-window floating panels** (or floating dock
  groups) within the same OS window.
- The platform must not create new OS windows.

Implementation guidance:

- Docking UI (`ecosystem/fret-docking`) should consult capabilities and/or settings to avoid
  emitting `CreateWindowKind::DockFloating` when `ui.multi_window == false`.
- If a tear-off request still occurs (e.g. loading a persisted multi-window layout), the host must
  render the additional logical window roots as in-window virtual windows instead of failing.
  - Contract helper: `DockGraph::import_layout_for_windows_with_fallback_floatings` can import
    unmapped logical windows into a fallback window as floating containers.

### 3) Input, focus, and modality remain window-absolute

On single-window platforms, all routing is still scoped to the single OS window:

- Pointer/keyboard events are delivered to the one `AppWindowId` for that OS window.
- Modal barriers and overlay roots remain correct within that OS window (ADR 0011 / ADR 0067 / ADR
  0069).

Virtual window surfaces must obey the same rules as any other UI subtree:

- no “special” bypasses for focus/capture,
- no hidden platform forks in widgets (ADR 0054).

### 4) Viewport input disambiguation is panel-scoped, not window-scoped

Because a single-window platform has only one `AppWindowId`:

- `ViewportInputEvent.window` is not sufficient to identify which viewport is targeted.

The intended disambiguation mechanism remains:

- docking panel identity (`PanelKey`) and viewport mapping (`ViewportMapping`) at the policy layer
  (see ADR 0025 / ADR 0049).

### 5) Persistence: keep extra logical windows, but treat placement metadata as best-effort

Persisted layouts may include:

- multiple logical windows,
- optional placement metadata (`DockLayoutWindow.placement`) (ADR 0017).

On single-window platforms:

- placement metadata is ignored (or treated as hints for in-window virtual window positions),
- but the logical window roots remain present in the data model.

## Consequences

Pros:

- wasm/mobile can execute multi-window docking layouts without changing core contracts.
- no runtime virtualization of `AppWindowId` is required.
- avoids late-stage rewrites by making “what happens when multi-window is unavailable” explicit.

Cons:

- virtual window surfaces must be implemented as policy-layer UI composition (likely in
  `fret-docking`), including:
  - z-ordering,
  - resizing/moving affordances,
  - focus restoration rules.

## Alternatives Considered

### A) Virtualize `AppWindowId` and run multiple `UiTree`s inside one OS window

Pros:

- closer to “real” multi-window semantics.

Cons:

- high complexity: effects, cursor, IME, accessibility, and viewport input would need a virtualization
  layer to map between logical windows and the single OS window.
- risks ballooning the runtime contract surface (ADR 0066).

### B) Forbid multi-window layouts on single-window platforms (drop extra roots)

Pros:

- simplest implementation.

Cons:

- breaks persistence portability and makes layout files platform-dependent.
- forces editor UX to diverge between desktop and wasm/mobile.

## Implementation Notes (Non-Normative)

- Capability gate: use `PlatformCapabilities.ui.multi_window` (ADR 0054) plus user settings (e.g.
  “tear-off enabled”) to control whether tear-off is available.
- Windowing quality signals (ADR 0054) may still require policy-level degradation on multi-window
  platforms (e.g. disable follow-mode or avoid z-level nudges when reliability is `best_effort`).
- Rendering model: treat each additional `DockLayoutWindow` root as an in-window “floating dock host”
  whose chrome and interaction policy live in `fret-docking`.
- Testing strategy:
  - add a demo harness that runs docking with `ui.multi_window=false` and validates:
    - tear-off becomes in-window float,
    - drag arbitration (ADR 0072) still holds.
  - Suggested harness: `cargo run -p fret-demo --bin docking_arbitration_demo` with `FRET_SINGLE_WINDOW=1` (see `docs/docking-arbitration-checklist.md`).

## Implementation Status

This policy is implemented in the current workspace:

- Dock layout import fallback (unmapped logical windows -> in-window floatings): `crates/fret-core/src/dock.rs`
- Tear-off request degradation (no OS window creation when `ui.multi_window == false`): `ecosystem/fret-docking/src/runtime.rs`

## References

- ADR 0013: `docs/adr/0013-docking-ops-and-persistence.md`
- ADR 0017: `docs/adr/0017-multi-window-display-and-dpi.md`
- ADR 0054: `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- ADR 0025: `docs/adr/0025-viewport-input-forwarding.md`
- ADR 0072: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
