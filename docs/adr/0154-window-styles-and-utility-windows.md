# ADR 0154: Window Styles and Utility Windows (Transparent / Frameless / Always-on-top)

Status: Proposed

## Context

Fret already supports:

- multiple OS windows for docking tear-off (`CreateWindowKind::DockFloating`),
- per-window coordinate + DPI semantics (ADR 0017),
- multi-root overlay composition *within* a window (menus/popovers/tooltips) (ADR 0011, ADR 0064).

However, some desktop-class UI experiences require OS window styles that are not representable as
in-window overlays:

- transparent and frameless windows (desktop pets, HUD-style overlays),
- always-on-top z-level hints,
- skip taskbar / alt-tab visibility (tool windows),
- non-activating windows (do not steal keyboard focus),
- mouse passthrough (click-through overlays), where supported.

We want to support these without:

- leaking backend-specific types (`winit`, `HWND`, `NSWindow*`) into portable crates,
- forcing widgets to branch on platform details (ADR 0054),
- conflating OS windows with in-window overlay roots (ADR 0011).

## Goals

1. Add a **portable “window style request”** shape that can be issued from the app/runtime layer.
2. Keep a strict **runner/backends own OS window implementation** boundary (ADR 0091).
3. Make all features **capability-gated and degradable at runtime** (ADR 0054, ADR 0084).
4. Avoid conflicts with docking, viewports, and multi-root overlays.

## Non-goals

- System-wide selection, global hotkeys, cross-app integration (out of scope for this ADR).
- Per-pixel hit-test regions, shaped windows, or arbitrary compositor effects beyond “transparent”.
- A portable guarantee that all window style requests can be honored on all platforms.
- Full window state management (maximize/minimize/fullscreen, native tabbing groups, restore bounds)
  as a framework-level contract. These belong to the runner/app shell and may be added via separate
  ADRs if needed.

## Decision

### 1) Distinguish “in-window overlays” from “utility OS windows”

In-window overlays remain the default for:

- menus, context menus, popovers, tooltips, toasts, dialogs, command palette surfaces.

These must continue to use multi-root overlay composition (ADR 0011) and the anchored placement
solver (ADR 0064 / ADR 0104). They must not require creating additional OS windows.

Utility OS windows are reserved for:

- desktop HUD/pet-like surfaces,
- special tool windows that intentionally use OS-level z-ordering/visibility semantics,
- host-driven integration points that require a distinct OS surface.

### 2) Add a portable window style request to `CreateWindowRequest`

Extend `fret_runtime::CreateWindowRequest` with a portable style request value:

- `style: WindowStyleRequest` (default = normal app window behavior)

`WindowStyleRequest` is pure data and must not contain backend types.

In addition, `CreateWindowRequest` should carry a portable **window role** that affects host/runner
policy (but does not imply a backend type):

- `role: WindowRole` (default = `Auxiliary` unless otherwise specified by the caller)

`WindowRole` v1:

- `Main`: the primary app window; closing it may exit the runner depending on host config.
- `Auxiliary`: secondary windows (dock tear-off, tool windows, utility windows); closing them must
  never imply exiting the app instance.

Rationale:

- Utility windows should not accidentally inherit “main window” close/exit semantics (ADR 0094 and
  the runner’s `exit_on_main_window_close` policy).
- Docking tear-off windows are not “main” and should remain safe by default.

### 3) Define a minimal, stable `WindowStyleRequest` vocabulary

The request is intentionally small and “intent-oriented”:

- `decorations: Option<WindowDecorationsRequest>` — request a window decoration policy.
- `resizable: Option<bool>`
- `transparent: Option<bool>` — requests a transparent composited window background.
- `z_level: Option<WindowZLevel>` — `Normal`, `AlwaysOnTop`.
- `taskbar: Option<TaskbarVisibility>` — `Show`, `Hide`.
- `activation: Option<ActivationPolicy>` — `Activates` (default), `NonActivating`.
- `mouse: Option<MousePolicy>` — `Normal`, `Passthrough` (click-through).

Notes:

- `transparent` is not only a window flag; it is also a rendering/compositing contract (see §6).
- `NonActivating` defines focus semantics; it is allowed to still receive pointer events.
- `Passthrough` is defined at the window level (not per-pixel). Per-pixel hit regions are out of
  scope for v1.
- `decorations` refers to the window frame/titlebar/controls policy, not to alpha transparency.

`WindowDecorationsRequest` v1:

- `System` (default): compositor/system decorations (platform default).
- `None`: request a frameless window (client-drawn).
- `Server`: request server-side decorations (Wayland only; best-effort).
- `Client`: request client-side decorations (Wayland only; best-effort).

Future extensions (explicitly out of scope for v1):

- `AlwaysOnBottom` / “desktop level” semantics.
- Per-pixel hit testing / shaped windows.
- Runtime style mutation (`WindowRequest::SetStyle`), beyond runner-internal temporary overrides.
- OS-provided background materials (blur/mica/vibrancy) as portable style facets. These typically
  require renderer + platform coordination and should be capability-gated behind a separate ADR.

### 4) Runtime capability gating for each style facet (ADR 0054)

Extend `PlatformCapabilities.ui` with booleans for the style facets, and add matching string keys:

- `ui.window.transparent`
- `ui.window.always_on_top`
- `ui.window.skip_taskbar`
- `ui.window.non_activating`
- `ui.window.mouse_passthrough`
- `ui.native_window_handle` (escape hatch; backend-specific)

The runner/backend must:

1. Advertise available capabilities at startup.
2. Clamp requested window styles to what is available on the current platform/backend.

Widgets and commands must not hard-fork on target OS; they gate behavior via `when` using these
capabilities (ADR 0054).

In addition to capability gating, this contract requires **observability**:

- The runner must expose the “effective/clamped” window style per window to diagnostics and
  inspection tools (ADR 0036), so that users can debug why a style request was ignored/clamped.
- The runner may choose the mechanism (log line on create, debug HUD entry, or a window-scoped
  runtime service), but the information must be visible without requiring native handle access.

Style precedence and runner overrides:

- The requested `WindowStyleRequest` is the *base policy* for the window.
- The runner may apply temporary overrides for correctness/UX (e.g. docking tear-off “keep moving
  window on top” during drag), but must restore the base policy when the temporary condition ends.
- If an override conflicts with the requested base policy, the override wins for the duration of
  the temporary condition.

Input and focus semantics (v1):

- `ActivationPolicy::NonActivating` means the runner must not programmatically focus/activate the
  window as a side-effect of creation or pointer interaction, where supported.
- `MousePolicy::Passthrough` means the window must not receive pointer events where supported; when
  enabled, the UI runtime must not assume it will see pointer events for that window.
- Neither policy implies global shortcuts or text input; utility windows should be treated as
  “pointer-only unless focused” by default.

Accessibility (A11y) default for utility windows:

- Utility windows should default to “no special A11y behavior” (they participate like any other
  window), but the runner must be able to *disable* platform accessibility exposure per window for
  HUD/pet-style overlays if required by platform conventions.
- This ADR does not lock a portable A11y policy field in `WindowStyleRequest` yet; it only requires
  that the backend can implement such a policy without leaking backend types into portable crates.

### 5) Single-window platforms (ADR 0084)

When `PlatformCapabilities.ui.multi_window == false`:

- Creating additional OS windows is unavailable by definition.
- Apps should gate “utility window” features off via capabilities/`when`.

Behavior when a request is still issued is intentionally *best-effort* and runner-defined:

- backends may ignore the request,
- or map it to an in-window virtual surface (policy-layer), if the host chooses.

Recommended default behavior (v1):

- Ignore the request (no additional OS window), log a rate-limited warning, and rely on capability
  gating to prevent repeated requests.

This ADR does not require a portable “window creation failure completion” mechanism.

### 6) Transparent window implies a renderer + surface alpha contract

If `transparent == true` is requested and supported:

- the runner must configure the platform window for transparency (backend-specific),
- the renderer/surface configuration must select an alpha mode that supports compositing (when
  available) and must clear with alpha = 0 where appropriate.

If the backend cannot provide a composited alpha surface, it must clamp `transparent` to false and
report `ui.window.transparent == false` in capabilities.

Note (Zed/GPUI reference, non-normative):

- GPUI models “window background appearance” separately from content theme, including `Opaque`,
  `Transparent`, `Blurred`, and Windows 11 `Mica*` variants (`repo-ref/zed/crates/gpui/src/platform.rs`).
  Fret’s `transparent` facet in v1 is intentionally narrower; richer OS background materials should
  be treated as a follow-up contract (likely tied to renderer effect semantics and capability gates).
- GPUI exposes a richer “window kind” concept (`WindowKind::{Normal, PopUp, Floating, Dialog, ...}`),
  including Wayland `LayerShell` windows on Linux (`repo-ref/zed/crates/gpui/src/platform.rs`).
  Fret intentionally treats menus/popovers as *in-window overlays* (ADR 0011 / ADR 0064) rather than
  relying on OS popup windows, to keep single-window platforms viable (ADR 0084).
- GPUI’s `WindowDecorations` request is a concrete reason to keep `decorations` extensible (Wayland
  client vs server side decorations) (`repo-ref/zed/crates/gpui/src/window.rs`,
  `repo-ref/zed/crates/gpui/src/platform.rs`).
- On Windows, GPUI maps `WindowKind::PopUp` to a tool window style (`WS_EX_TOOLWINDOW`), which avoids
  taskbar/Alt-Tab presence by design; Fret’s `TaskbarVisibility::Hide` is intended to capture the
  same portable intent without adopting GPUI’s window taxonomy (`repo-ref/zed/crates/gpui/src/platform/windows/window.rs`).

### 7) Optional backend escape hatch: raw window handle access (non-portable)

Fret may provide a **backend-only** API to access native window handles for ecosystem integrations:

- exposed from runner/backend crates (e.g. `fret-launch` / `fret-runner-winit`),
- never from portable crates (`fret-core`, `fret-runtime`, `fret-ui`).

This escape hatch must be explicitly documented as:

- non-portable (native-only, backend-specific; not available on single-window platforms by default),
- capability-gated (`ui.native_window_handle == true`) and/or feature-gated (e.g. `native-window-handle`),
- not required for core framework features (all framework-owned functionality must work without it).

API guardrails:

- Prefer returning `raw_window_handle` types (or providing access via `HasWindowHandle`/`HasDisplayHandle`)
  over exposing `winit::window::Window` directly, to avoid “winit becomes the portable contract” drift.
- Prefer callback-style access (`with_native_window_handle(|h| ...)`) so the handle is not assumed to be
  `'static` or safely storable across frames; the runner retains lifetime + thread-affinity control.
- The handle access point must be on the runner thread and only valid while the OS window is alive.
  Persisting the handle and using it after window close is explicitly unsupported.

## Consequences

Pros:

- Enables “desktop pet / HUD / tool window” classes without violating crate layering (ADR 0091).
- Keeps the portable contract small, intent-based, and runtime-degradable (ADR 0054).
- Avoids conflating in-window overlay roots (menus/popovers) with OS windows (ADR 0011).

Cons:

- Some facets are not uniformly available (Wayland/sandboxing), so capability gating is required.
- Transparent window support requires renderer/surface integration work, not just window flags.

## Alternatives Considered

1) Implement everything as in-window overlays (ADR 0011 only)
- Rejected: cannot represent desktop-level z-order/visibility semantics or “on desktop” behavior.

2) Expose native window handles in portable crates
- Rejected: violates backend/portable layering (ADR 0091) and forces platform forks in ecosystem code.

3) Require apps to directly use winit/native APIs
- Rejected: causes fragmentation and makes cross-backend support and portability matrix enforcement
  much harder (ADR 0054).

## Implementation Notes (Non-normative)

Likely touch points:

- Portable types:
  - `crates/fret-runtime/src/effect.rs` (`CreateWindowRequest`)
  - `crates/fret-runtime/src/capabilities.rs` (capability keys + struct extension)
- Runner:
  - `crates/fret-launch/src/runner/common.rs` (`WindowCreateSpec` may need style fields)
  - `crates/fret-launch/src/runner/desktop/mod.rs` (apply style to winit window)
- Renderer/surface:
  - `crates/fret-render-wgpu/src/surface.rs` (alpha mode selection + clear alpha policy)

Recommended validation:

- Add a desktop demo that opens a frameless transparent always-on-top window and reports the
  effective clamped style + capabilities in logs/inspector (ADR 0036).

## References

- Platform backends and layering: `docs/adr/0091-platform-backends-native-web.md`
- Runtime capability model: `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Multi-window/DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
- Multi-window degradation policy: `docs/adr/0084-multi-window-degradation-policy.md`
- In-window overlays and multi-root composition: `docs/adr/0011-overlays-and-multi-root.md`
- Anchored overlay placement: `docs/adr/0064-overlay-placement-contract.md`, `docs/adr/0104-layout-driven-anchored-overlays.md`
- Window close semantics: `docs/adr/0094-window-close-and-web-runner-destroy.md`
