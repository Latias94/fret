# ADR 0112: UiAssets Facade and Golden-Path Wiring (Images / SVGs)

Status: Accepted

## Context

Fret follows a handle-based resource boundary (ADR 0004):

- UI code references stable IDs (`ImageId`, `SvgId`, ...).
- The runner/renderer owns the actual GPU resources.
- Registration happens via effects at flush points (images) or via `UiServices` (SVGs).

In practice, new users want a GPUI-style experience:

- “use_asset” convenience APIs that dedupe keys and handle caching,
- a single place to configure budgets / eviction policy,
- minimal boilerplate to keep caches driven by the event pipeline,
- no drift into an editor-grade asset database/import pipeline (ADR 0026 remains out of scope).

We already have cache primitives in `ecosystem/fret-asset-cache` (images/SVGs), and a re-export surface
`ecosystem/fret-ui-assets` (ADR 0106), but wiring is still easy to get wrong:

- `ImageAssetCache` requires `handle_event` to observe `Event::ImageRegistered` / `Event::ImageRegisterFailed`.
- SVG caching uses `UiServices::svg()` and does not require event driving, but still needs budgets/stats access.

## Goals

- Provide a tiny, memorable *single entry point* for UI render asset caches: images + SVGs.
- Make “golden path” apps work without writing event-boilerplate to drive image cache state.
- Keep layering intact (ADR 0092 / ADR 0106):
  - kernel crates do not depend on ecosystem,
  - `fret-bootstrap` may integrate optional conveniences via feature flags,
  - component crates remain portable (no `winit`/`wgpu` coupling).

## Non-goals

- A project/editor asset pipeline (GUIDs, import graphs, derived artifacts) (ADR 0026).
- A global “everything crate” that pulls in all defaults (ADR 0106 rejects this).
- Forcing a single application architecture (sync/async/multi-thread).

## Decision

### 1) Introduce `fret-ui-assets::UiAssets` as a facade

`ecosystem/fret-ui-assets` provides a small facade type:

- `UiAssets::configure(host, budgets)` (see `UiAssetsBudgets`):
  - ensures the global `ImageAssetCache` / `SvgAssetCache` exist,
  - applies budgets/max-entries.
- `UiAssets::handle_event(host, window, event)`:
  - drives the image cache state machine from the event pipeline.
- `UiAssets::{image_stats, svg_stats}`:
  - provide read-only stats for overlays/debug UI.

This facade is intentionally thin and does not change ADR 0004’s resource boundary.

### 2) Golden-path driver drives `UiAssets` when enabled

When `fret-bootstrap` is built with the `ui-assets` feature:

- `UiAppDriver` drives `UiAssets::handle_event(...)` from the event pipeline by default.
- Apps can opt out via `UiAppDriver::drive_ui_assets(false)` (advanced/manual wiring).

This makes `ImageAssetCache` “just work” in small apps without additional boilerplate.

### 3) Usage examples are part of the contract

We provide a runnable demo:

- Native demo: `apps/fret-demo/src/bin/assets_demo.rs`

This demo exists to validate the “golden path” experience end-to-end.

## Consequences

### Benefits

- Users get a single place to look for “UI render asset caches”.
- Golden-path apps do not need to remember to forward events into the image cache.
- Third-party ecosystem crates can depend on `fret-ui-assets` without depending on runner glue (`fret-bootstrap`).

### Costs

- One more public surface to keep stable (facade + budgets struct).
- Feature gating adds some complexity in `fret-bootstrap`, but avoids forcing extra dependencies on all users.

## References

- Resource handles boundary: `docs/adr/0004-resource-handles.md`
- Asset pipeline out-of-scope: `docs/adr/0026-asset-database-and-import-pipeline.md`
- Crate layering: `docs/adr/0092-crate-structure-core-backends-apps.md`
- Bootstrap + ui-assets story: `docs/adr/0106-ecosystem-bootstrap-ui-assets-and-dev-tools.md`
- Golden-path driver/pipelines: `docs/adr/0110-golden-path-ui-app-driver-and-pipelines.md`
- Zed/GPUI asset convenience layer (non-normative):
  - async asset fetching + caching entry points:
    `repo-ref/zed/crates/gpui/src/app.rs` (`fetch_asset`, `remove_asset`)
  - asset abstractions and IDs:
    `repo-ref/zed/crates/gpui/src/asset_cache.rs`, `repo-ref/zed/crates/gpui/src/assets.rs`
