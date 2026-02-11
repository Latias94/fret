---
title: WebView integration (native wry) — v1
status: draft
date: 2026-02-11
scope: ecosystem/fret-webview, ecosystem/fret-webview-wry, WebPreview backend
---

# WebView integration (native wry) — v1

This workstream defines how we integrate an embedded WebView into Fret **without breaking crate
boundaries** or forcing heavy platform dependencies into UI/policy crates.

Primary consumer (initial): `ecosystem/fret-ui-ai` (`WebPreview`).

## Why a dedicated crate pair

We want:

1. A **small, stable contract** crate usable by UI/policy layers (`fret-webview`).
2. A **native implementation** crate that can depend on heavy / platform-specific deps (`wry`,
   WebView2, etc.) behind features (`fret-webview-wry`).

This matches the repository's overall philosophy:

- `crates/fret-ui`: mechanisms only.
- `ecosystem/*`: policy/recipes/backends can iterate faster and be feature-gated.

## Crate split (Plan B)

- `ecosystem/fret-webview` (contract-only)
  - Types for intents/requests + events.
  - No dependency on `wry`/windowing backends.
  - Can be used by `fret-ui-ai` to express a WebView embedding target.

- `ecosystem/fret-webview-wry` (native impl)
  - Depends on `wry` (feature-gated).
  - Wires the contract into the actual platform WebView instance(s).
  - Owns the “how do we embed” details.

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

- `ecosystem/fret-webview/src/lib.rs` (`best_bounds_for_test_id`, `placement_for_test_id`)

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
