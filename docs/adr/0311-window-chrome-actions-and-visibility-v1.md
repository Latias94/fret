# ADR 0311: Window Chrome Actions and Visibility v1 (Drag / Resize / Show-Hide)

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed (GPUI): https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

Status: Proposed

## Context

Frameless utility windows (ADR 0139) shift responsibility for “chrome” interactions from the OS to
the app:

- moving the window (dragging a custom titlebar region),
- resizing (dragging edges/corners),
- showing/hiding for launcher-style UX (global hotkey toggles visibility; blur hides).

Today, Fret’s portable window contract focuses on *style facets* (always-on-top, activation, etc).
To make frameless windows practical without leaking backend types, we need portable requests for
common OS window actions.

## Goals

1. Provide a portable mechanism for initiating OS window move/resize from app-defined UI regions.
2. Provide a portable mechanism to show/hide windows without closing them.
3. Keep all surfaces capability-gated and best-effort (ADR 0054).
4. Keep backend/window-handle types out of portable crates (ADR 0090).

## Non-goals

- Per-pixel hit test regions or shaped windows (still out of scope; see ADR 0139).
- Custom non-client metrics parity across platforms (caption buttons, shadows, snap layouts).
- Global hotkeys or system tray integration (separate work; see launcher workstream).

## Decision

### 1) Add portable `WindowRequest` actions

Extend `WindowRequest` (portable) with:

- `SetVisible { window: AppWindowId, visible: bool }`
- `BeginDrag { window: AppWindowId }`
- `BeginResize { window: AppWindowId, direction: WindowResizeDirection }`

`BeginDrag` / `BeginResize` initiate an OS-native interactive move/resize operation if supported.
They are intended to be triggered from pointer handling in the app/policy layer (e.g. a custom
titlebar widget).

### 2) Define `WindowResizeDirection` v1

`WindowResizeDirection` is an 8-way edge/corner vocabulary:

- `N`, `NE`, `E`, `SE`, `S`, `SW`, `W`, `NW`

### 3) Capability keys (ADR 0054)

Add capability keys:

- `ui.window.set_visible`
- `ui.window.begin_drag`
- `ui.window.begin_resize`

Runners/backends must:

1. Advertise these at startup.
2. Clamp/ignore unsupported requests.
3. Preserve observability (log/diagnostics) when requests are ignored due to missing capabilities.

### 4) Semantics and degradation

Normative semantics:

- `SetVisible` toggles OS visibility without destroying window state, when supported.
- `BeginDrag` / `BeginResize` are **best-effort** and may be ignored.
- Requests must not panic/fail hard if unsupported; they degrade to no-op.

Recommended policy guidance (non-normative):

- For frameless windows, ecosystems should provide a standard “draggable region” recipe that maps
  pointer down to `BeginDrag`.
- Resize affordances should be explicit (thin edge handles) rather than relying on OS hit testing.

## Consequences

Pros:

- Makes frameless windows usable without native handle escape hatches.
- Keeps “mechanism vs policy” clean: the mechanism is `BeginDrag/BeginResize`, policy decides where.

Cons:

- Some platforms may have restrictions (Wayland, sandboxing); capability gating must be respected.
- Requires careful coordination with pointer capture and input routing policy in ecosystem layers.

## Alternatives considered

1) Rely on raw window handle escape hatch
- Rejected: makes frameless UX non-portable and forces platform forks.

2) Implement per-pixel hit testing and map edges to OS hit test regions
- Deferred: powerful but hard-to-change; keep v1 minimal.

## Implementation notes (non-normative)

Likely touch points:

- Portable contract:
  - `crates/fret-runtime/src/effect.rs` (`WindowRequest`)
  - `crates/fret-runtime/src/capabilities/keys.rs` (new keys)
  - `crates/fret-runtime/src/capabilities/platform.rs` (key mapping)
- Desktop runner:
  - winit supports initiating drags/resizes on supported platforms; map to best-effort calls.

Recommended validation:

- Add a small demo with a frameless window and a custom titlebar:
  - dragging the titlebar moves the window,
  - dragging a corner resizes it,
  - toggling visibility preserves in-memory state.

## References

- Window styles and utility windows: `docs/adr/0139-window-styles-and-utility-windows.md`
- Capability gating: `docs/adr/0054-platform-capabilities-and-portability-matrix.md`

