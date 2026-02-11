---
title: WebView integration (native wry) — v1 TODO
status: active
date: 2026-02-11
scope: crates/fret-webview, crates/fret-webview-wry, WebPreview backend
---

# WebView integration (native wry) — v1 TODO

Workstream narrative: `docs/workstreams/webview-wry-v1.md`.

Tracking format:

- ID: `WEBVIEW-WRY{n}-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## M0 — Contract + crate skeletons

- [x] WEBVIEW-WRY0-contract-001 Create `crates/fret-webview` crate (contract-only).
- [x] WEBVIEW-WRY0-contract-002 Define v1 request + event enums (minimal set).
- [ ] WEBVIEW-WRY0-contract-003 Define stable ID + lifecycle rules (create/destroy; per-window constraints).
- [ ] WEBVIEW-WRY0-contract-004 Add a unit test that round-trips a request/event list (serde optional; only if needed).

- [x] WEBVIEW-WRY0-backend-010 Create `crates/fret-webview-wry` crate (native backend).
- [x] WEBVIEW-WRY0-backend-011 Add feature-gated `wry` dependency (native only).
- [x] WEBVIEW-WRY0-backend-012 Decide windowing glue strategy (winit/tao compatibility) and document it.

## M1 — Host integration seam

- [x] WEBVIEW-WRY1-host-001 Define how UI expresses “WebView wants this rect” (handle/adapter seam).
  - v1 plan: stable `test_id` + bounds lookup via `SemanticsSnapshot` (`fret-webview` helpers).
- [~] WEBVIEW-WRY1-host-002 Implement lifecycle: create/destroy per app window.
  - Done: switching `WebPreview` backend IDs destroys the previous instance.
  - Done: window close path destroys all hosted instances and clears surface registrations (best-effort).
  - TODO: unmount-driven destroy is not solved in v1 retained authoring (needs an explicit lifecycle hook or host-side GC policy).
- [x] WEBVIEW-WRY1-host-003 Route navigation intents to backend (load/go_back/go_forward/reload).
- [x] WEBVIEW-WRY1-host-004 Emit navigation state events back into a model/UI-consumable place.
  - Implemented: backend emits `WebViewEvent::NavigationStateChanged` + `UrlChanged` + `TitleChanged`.
  - Host derives `WebViewRuntimeState` and UI polls it via `webview_runtime_state(...)`.

## M2 — `WebPreview` end-to-end (feature-gated)

- [~] WEBVIEW-WRY2-ai-001 Add `fret-ui-ai` feature flags for webview backends (e.g. `webview`, `webview-wry`).
- [x] WEBVIEW-WRY2-ai-002 Wire `WebPreview` navigation buttons to backend intents (when enabled).
- [x] WEBVIEW-WRY2-ai-003 Reflect `can_back/can_forward/is_loading` into the chrome state.
  - Back/Forward auto-disable based on runtime state.
  - Reload is disabled while `is_loading` is true (v1 minimal signal).
- [x] WEBVIEW-WRY2-ai-004 UI Gallery demo: embedded WebView (or fallback to chrome-only with explicit label).
- [x] WEBVIEW-WRY2-ai-005 Add a diag script gate that:
  - enters a URL,
  - observes a “loaded” marker (backend-provided),
  - toggles console panel (if supported).
  - Implemented gate: `tools/diag-scripts/ui-gallery-ai-web-preview-demo-webview-wry-nav.json` (requires `--features webview-wry`).

## Blockers / risks

- [ ] WEBVIEW-WRYX-risk-001 Clarify if `wry` is compatible with our native runner stack without forking windowing.
  - gpui-component embeds as a native child view via `build_as_child` (raw-window-handle).
  - We still need to validate that this works with our `winit` runner and swapchain/compositor setup.
- [ ] WEBVIEW-WRYX-risk-002 Decide how we handle z-order + input capture for child WebViews inside docking panels.
