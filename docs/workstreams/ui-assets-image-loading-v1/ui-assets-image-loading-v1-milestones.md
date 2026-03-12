# UI Assets image loading v1 — Milestones

## M0 — Observability (done)

- ViewCache-safe invalidation via per-request signal model.
- Event-driven GPU-ready bump (no continuous-frame hacks).
- Minimal runtime error surfaces (warn logs + optional UI debug badge).

## M1 — UI Gallery parity

- Card "event cover" renders `assets/textures/test.jpg` reliably on native.
- Visual alignment review against shadcn Card docs:
  - Cover cropping (`object-fit: cover`)
  - Overlay layering and clipping
  - Aspect ratio and container sizing

## M2 — Regression gates

- Unit tests for:
  - Decode completion bumps signal model.
  - GPU-ready (`ImageRegistered`) bumps signal model.
- Optional diag script (only after stable selectors/visuals):
  - Navigate to Card preview → assert event cover image node exists → capture bundle.

## M3 — Optional query ergonomics

- `fret-query` integration behind a feature flag, without changing the base image loading contract.
- Document the recommended patterns:
  - no-query: `ImageSource::from_path/from_bytes` + `cx.use_image_source_state`
  - with-query: query provides the `ImageSource` / path resolution, UI stays the same

