# Image Source + ViewCache Correctness v1 (Milestones)

Last updated: 2026-02-13

This file defines coarse milestones for:

- `docs/workstreams/image-source-view-cache-v1.md`

## M0 — Naming + boundaries locked

Exit criteria:

- `fret-ui-assets` feature flags follow repo conventions:
  - `ui` for `ElementContext` sugar
  - `query-integration` for optional `fret-query` integration
- No new crate is introduced for this workstream.

## M1 — ViewCache-safe baseline (no query required)

Exit criteria:

- Image load completion triggers a rerender even when the subtree is under a `ViewCache` boundary.
- No “continuous frames” requirement to complete loads.
- A UI-facing API exists behind `fret-ui-assets/ui` that:
  - observes a model dependency every frame,
  - returns a data-only state including `Option<ImageId>`.

## M2 — GPU-ready correctness (no implicit polling)

Exit criteria:

- GPU completion events (`ImageRegistered` / `ImageRegisterFailed`) trigger invalidation under
  `ViewCache` without requiring continuous frames.
- Invalidation is per-request (signal models), not a global `ImageAssetCache` observe.

## M3 — UI Gallery parity + regression gate

Exit criteria:

- UI Gallery Card “Event cover” demo loads a real file image (e.g. `assets/textures/test.jpg`) reliably.
- A diag screenshot script exists for the Card cover case and targets stable `test_id`s.

## M4 — Optional `fret-query` integration (ergonomics)

Exit criteria:

- With `fret-ui-assets/query-integration` enabled, authors can opt into query-driven decoding/fetching.
- Baseline `fret-ui-assets/ui` remains fully usable without `fret-query`.
