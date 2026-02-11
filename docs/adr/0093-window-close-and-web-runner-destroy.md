# ADR 0093: Window Close Semantics and Web Runner Destroy

## Context

Fret supports multiple platform backends (native via `winit` and web via `winit` + WebGPU). The UI
layer also supports docking and floating/overlay surfaces (menus, popovers, tooltips) that are not
OS-managed windows.

We need a clear, portable meaning for “close” to avoid conflating:

- Closing the **application window** (or exiting the app instance on single-window platforms).
- Closing a **dock tab/panel** inside the app.
- Dismissing a **floating overlay** (menu/popup/tooltip) rendered inside the same surface.

On the web, browsers do not reliably allow applications to close the tab/window. The correct
semantics is to stop the runner instance (stop rendering, stop processing events, and unsubscribe
DOM listeners).

Related ADRs:

- ADR 0011: Overlays and Multi-Root UI Composition
- ADR 0013: Docking Operations, Stable Panel Identity, and Layout Persistence
- ADR 0083: Multi-Window Degradation Policy
- ADR 0090: Platform Backends (Native + Web)

## Decision

### 1) `WindowRequest::Close` means “close the app window / exit the app instance”

`fret_runtime::WindowRequest::Close(AppWindowId)` is reserved for OS/window-runner level close.

- On **native** backends: it maps to closing the corresponding OS window (or exiting if it is the
  last/main window).
- On **web** backends: it maps to **exiting the runner instance**, not attempting to close the
  browser tab/window.

### 2) Docking and overlays do not use `WindowRequest::Close`

Docking operations (close tab/panel, split/merge, tear-off when supported) must be expressed as
docking model operations (e.g. `DockOp::*`), not as window close.

Overlay dismissal (menus/popovers/tooltips) must be expressed as overlay/interaction policy state
updates (dismiss reasons, outside press, focus changes), not as window close.

### 3) Web hosts get an explicit “destroy” API

In addition to app-driven `WindowRequest::Close`, the web backend exposes a host-controlled handle
to stop the runner instance:

- `fret_launch::WebRunnerHandle::destroy()`

This is intended for SPA/embedded scenarios (restarting the demo in the same page, hot reload,
tearing down the UI when navigating within the app shell). Destroy is implemented by requesting an
exit and waking the event loop; the runner exits at a well-defined hook and drops resources, which
unsubscribes any DOM listeners owned by the runner.

## Consequences

- `WindowRequest::Close` becomes a stable, portable “quit/close window” primitive.
- Docking/overlay “close” stays purely in the UI model layer and remains platform-independent.
- Web backends do not pretend they can close the tab; they stop the runner instance instead.
- Hosts embedding Fret on the web have an explicit teardown mechanism (`destroy`) that prevents
  event listener leaks and enables re-running the same demo in one page session.

