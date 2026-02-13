# Image Source + ViewCache Correctness v1 (Workstream)

Status: Draft (implementation notes; ADRs remain the source of truth)

Related workstreams:

- `docs/workstreams/image-support-v1.md` (broader image surface: fit semantics, metadata, caching)
- `docs/workstreams/query-lifecycle-v1.md` (no implicit polling; model-driven invalidation)
- `docs/workstreams/state-management-v1.md` (model/selector/query layering)

## Why this exists

Fret already has the “GPU resource handle” story for images:

- decode/load is ecosystem policy,
- GPU registration is driven through flush-point effects (ADR 0004),
- rendering consumes stable `ImageId` (mechanism-only; `SceneOp::Image`).

However, `ViewCache` subtree reuse introduces a correctness trap for asynchronous resources:

- a cached subtree can be **repainted** without being **re-rendered**,
- “hook-style” helpers that rely on being called again (to advance a state machine) may never run,
- `request_redraw` alone is insufficient when the cached subtree is reused.

This workstream defines a **ViewCache-safe** image loading contract for `fret-ui-assets` that:

1. works without `fret-query` (UI can load images out of the box), and
2. becomes more ergonomic when `fret-query` is available (optional integration), without creating a new crate.

## Goals (v1)

1. **ViewCache correctness:** image load completion must trigger a rerender when it affects visible UI.
2. **No implicit polling:** do not require “continuous frames” to make image loads complete.
3. **Layering discipline:**
   - keep `crates/fret-ui` mechanism-only,
   - keep image decode/network/media engines out of the kernel,
   - keep the “resource handle at flush point” contract intact (ADR 0004).
4. **Ergonomics:**
   - provide `ElementContext` sugar behind a feature flag (consistent with other ecosystem crates),
   - provide optional `fret-query` integration behind a feature flag (single-crate approach).
5. **Debuggability:** leave evidence anchors and (optional) diag scripts so regressions are cheap to catch.

## Non-goals (v1)

- A new global reactive graph (derived state remains explicit).
- Framework-owned network/media stack (no “download manager” in the kernel).
- Making “intrinsic image size” implicitly drive layout (must remain opt-in policy; see ADR 0124).

## Feature flag naming (consistency)

Existing conventions in this repo:

- `ui` — opt-in `ElementContext` helpers (e.g. `fret-query`, `fret-selector`, `fret-canvas`).
- `query-integration` — opt-in integration with `fret-query` without owning a `QueryClient` (e.g. `fret-router`).
- `app-integration` — optional helpers to wire a crate into `fret-app` (e.g. `fret-ui-assets`, `fret-ui-shadcn`).

Recommendation for `ecosystem/fret-ui-assets`:

- add `ui` for `ElementContext` extensions (ViewCache-safe observation),
- add `query-integration` for optional `fret-query`-powered fetch/decode paths,
- keep existing `app-integration`, `image-decode`, `image-metadata` as-is.

## Proposed architecture

### 1) Make image loading observable (the key ViewCache fix)

The core rule for ViewCache safety:

> If an async completion can change what a cached subtree renders, the subtree must have observed a
> model/global that changes when the completion happens.

Concretely, `fret-ui-assets` should expose image loading state via a `Model<...>` that UI observes
every frame through `ElementContext::observe_model` (or equivalent `watch_model` sugar).

Recommended shape (v1 baseline):

- keep the existing `ImageSourceRuntime` state machine unchanged (it owns `Decoded`/`Ready` etc).
- introduce a per-request *signal* model:
  - `ImageSourceUiSignal { epoch: u64 }`
  - mapped by `ImageSourceRequestKey` and kept alive by an app-global loader/client.

Implementation sketch:

- `ImageSourceLoader` stored as an untracked app global (ecosystem-level).
  - maps `ImageSourceRequestKey` → `Model<ImageSourceUiSignal>` (plus `last_used_frame` for GC)
  - stores `ImageSourceRequestKey` → `ModelId` in the runtime so the inbox drainer can bump it.
- Background decode completes via the inbox drain boundary and increments `epoch`.

This makes the dependency explicit and compatible with:

- `ViewCache` reuse (observed models are “touched” on reuse),
- `fret-selector` (`DepsBuilder.model_rev(...)` becomes a natural dependency signature),
- diagnostics (you can snapshot models and correlate state).

### 2) Separate “decode done” vs “GPU ready”

The UI-visible output is `ImageId`, which becomes `Some` only after GPU registration completes.
That readiness is driven by the event pipeline (`UiAssets::handle_event` → `ImageAssetCache::handle_event`).

To avoid implicit polling, v1 relies on two observed dependencies:

- **decode completion**: bumps `Model<ImageSourceUiSignal>` (per-request), and
- **GPU readiness**: UI observes the `ImageAssetCache` global, so `ImageRegistered` / `Failed`
  naturally invalidates view-cached subtrees (the cache is mutated through tracked globals).

### 3) UI sugar behind `fret-ui-assets/ui`

Add `ui` feature in `fret-ui-assets` (depends on `fret-ui`) that provides:

- `ImageSourceElementContextExt::use_image_source_state(...) -> ImageSourceState`
  - ensures the per-request signal model is observed (`Invalidation::Paint`)
  - observes `ImageAssetCache` global (`Invalidation::Paint`)
  - delegates to the existing `use_image_source_state_with_options` hook for the state machine

This is the “no query required” baseline. It should fully solve ViewCache correctness.

### 4) Optional `fret-query` integration behind `query-integration`

With `query-integration` enabled (depends on `fret-query`):

- provide helpers that build `QueryKey<DecodedRgba8>` (or bytes) from `ImageSourceId`/path/url.
- the query model becomes an additional observable dependency that drives decode completion
  without polling.

Important: the query integration should remain an optional fast path / ergonomics layer. The
baseline `use_image_source` must not require `fret-query`.

## Validation / regression gates

Minimum:

- a UI Gallery demo that loads a real file image (`assets/textures/test.jpg`) into a Card cover,
  and still completes when view-cache is enabled.

Recommended:

- a diag screenshot script for the UI Gallery Card image cover case (stable `test_id` targets),
  kept in `tools/diag-scripts/`.

## Migration targets (first wave)

- UI Gallery Card “Event cover” demo (shadcn Card image example).
- Any other shadcn recipes that currently rely on “hook-style polling” without an observed model.
