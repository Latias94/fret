---
title: WebView integration (native wry) — v1
status: draft
date: 2026-02-11
scope: crates/fret-webview, crates/fret-webview-wry, WebPreview backend
---

# WebView integration (native wry) — v1

This workstream defines how we integrate an embedded WebView into Fret **without breaking crate
boundaries** or forcing heavy platform dependencies into UI/policy crates.

Primary consumer (initial): `ecosystem/fret-ui-ai` (`WebPreview`).

## Upstream reference notes (gpui-component)

We keep a local reference checkout under `F:/SourceCodes/Rust/fret/repo-ref/gpui-component`.

Relevant observations (as of the pinned snapshot in that folder):

- Uses `wry` via package rename `lb-wry` (crate name stays `wry`), version `0.53.3`.
- Embeds via `WebViewBuilder::build_as_child(&window_handle)` (requires a `raw_window_handle` window).
- Updates bounds in the UI frame lifecycle by calling `webview.set_bounds(Rect { position, size })`
  in **logical pixel** coordinates.
- WebView is a native child view that draws **on top** of the GPU-rendered UI; any elements behind
  the WebView bounds will be occluded.
  - Their README recommends using a separate window or a popup layer for safety.
- Windows note: their example sets `GPUI_DISABLE_DIRECT_COMPOSITION=true` to render WebView
  reliably. We may need an equivalent note/flag depending on our swapchain/compositor path.

## Why a dedicated crate pair

We want:

1. A **small, stable contract** crate usable by UI/policy layers (`fret-webview`).
2. A **native implementation** crate that can depend on heavy / platform-specific deps (`wry`,
   WebView2, etc.) behind features (`fret-webview-wry`).

This matches the repository's overall philosophy:

- `crates/fret-ui`: mechanisms only.
- `ecosystem/*`: policy/recipes can iterate faster and be feature-gated.

Note: the **runner** (`crates/fret-launch`) must not depend on `ecosystem/*` crates (layering rule
`kernel-no-ecosystem`), so the v1 WebView backend crates live under `crates/` even though they are
feature-gated and still evolving.

## Crate split (Plan B)

- `crates/fret-webview` (contract-only)
  - Types for intents/requests + events.
  - No dependency on `wry`/windowing backends.
  - Can be used by `fret-ui-ai` to express a WebView embedding target.
  - Provides an app-global `WebViewHost` queue + surface registry (via `GlobalsHost`) so UI and
    runner code can communicate without adding new effect variants.

- `crates/fret-webview-wry` (native impl)
  - Depends on `wry` (feature-gated).
  - Wires the contract into the actual platform WebView instance(s).
  - Owns the “how do we embed” details.
  - Provides `WryWebViewHost` (request executor).

## v1 contract sketch (subject to iteration)

**Intents / requests (UI → host/backend):**

- `Create { id, initial_url }`
- `Destroy { id }`
- `LoadUrl { id, url }`
- `GoBack { id }`
- `GoForward { id }`
- `Reload { id }`
- `SetDevtoolsOpen { id, open }` (optional)

**Events (backend → UI):**

- `TitleChanged { id, title }`
- `UrlChanged { id, url }`
- `NavigationState { id, can_back, can_forward, is_loading }`
- `ConsoleMessage { id, level, message, timestamp }` (optional)
- `LoadProgress { id, progress_0_1 }` (optional)

## Embedding strategy (v1 expectation)

We do **not** assume “render webview into a GPU texture” is available cross-platform.

v1 target:

- Native child view / overlay integration is backend-owned (wry).
- UI owns only layout/chrome and expresses “a rectangle wants a WebView here”.

### v1 seam (pragmatic)

Until we have a dedicated “native surface anchor” element in the UI contract, v1 uses a pragmatic
approach:

- UI marks the intended WebView surface with a stable `test_id`.
- The host/backend locates the bounds in the window `SemanticsSnapshot` and positions the native
  child WebView accordingly.

This is implemented as a contract-level helper in `fret-webview` so it remains backend-agnostic:

- `crates/fret-webview/src/lib.rs` (`best_bounds_for_test_id`, `placement_for_test_id`)

Runner integration (v1):

- `crates/fret-launch` feature: `webview-wry`
  - After each window render pass, compute placements from the semantics snapshot (when any webview
    surface is registered for that window).
  - Drain `WebViewHost` requests, execute them via `WryWebViewHost`, and requeue create/load
    requests that cannot be satisfied yet.

UI integration (v1):

- `ecosystem/fret-ui-ai` feature: `webview`
  - `WebPreviewBody` registers a `WebViewSurfaceRegistration` for placement using a stable
    `test_id`.
  - `WebPreview` pushes `Create` and `LoadUrl` requests via the app-global `WebViewHost`.

Risk notes:

- Z-order/input routing between the WebView child and Fret-rendered UI must be explicit.
- Multi-window + docking tear-offs may require reparenting or per-window WebView instances.

## Non-goals (v1)

- A web/wasm WebView backend.
- Sandboxed execution of arbitrary code (`sandbox.tsx` is a separate workstream).
- Full DevTools parity.

## Deliverables (v1)

- `fret-webview` contract crate (no heavy deps).
- `fret-webview-wry` backend crate (native).
- A `fret-ui-ai` feature-gated `WebPreview` backend integration that can:
  - commit URL on Enter,
  - reflect can_back/can_forward state,
  - optionally surface console messages.

See TODO tracker: `docs/workstreams/webview-wry-v1-todo.md`.

## Implementation notes (current)

- Navigation state + URL/title are emitted by the backend and stored as `WebViewRuntimeState` in the
  app-global `WebViewHost` (`crates/fret-webview`).
- `WebPreview` uses runtime state to auto-disable Back/Forward buttons when a backend is enabled.
- Layout note: the embedded native child WebView needs a non-zero height constraint. In UI Gallery
  we pin the demo height explicitly; in real apps prefer placing `WebPreview` in a flex container
  that provides height (docking panel, split, viewport).

## Lifecycle + GC policy (v1)

Problem: a native child WebView can outlive its UI surface in a retained tree (page change, tab
close, panel unmount). If we do not destroy it, it may:

- continue to occlude GPU-rendered UI,
- keep capturing input,
- leak OS/webview resources.

v1 policy: **surface-driven GC** owned by the runner/host.

- UI/policy code registers a `WebViewSurfaceRegistration` **every frame** while mounted.
- The host stores `last_registered_frame` for each surface.
- Runner periodically sweeps per window:
  - if `now_frame - last_registered_frame > grace_frames`, enqueue `Destroy { id }` and clear the
    surface/runtime state.

Notes:

- This does not require explicit “unmount hooks” in `fret-ui` and works for any retained authoring
  style as long as the surface registers each frame.
- Callers should prefer the tracked helper that records `frame_id` on register:
  - `webview_register_surface_tracked(...)` (records `last_registered_frame` from `TimeHost::frame_id()`).
- Runner/host uses `webview_gc_stale_surfaces(...)` to enqueue `Destroy` for stale registrations.

## Known gaps (v1)

- `can_go_back/can_go_forward` is derived from a backend-side best-effort history tracker and may
  diverge from the underlying engine in edge cases (redirect chains, cross-origin swaps).
- Console capture is best-effort:
  - We inject a JS bridge that forwards `console.log/warn/error` to the host via IPC.
  - The host stores a bounded per-webview ring buffer.
  - The ring buffer can be cleared from UI (v1: best-effort, no persistence).
  - We still need to decide on richer formatting (structured args, stack traces) and filtering.
