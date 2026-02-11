Status: Active (workstream tracker)

This document tracks cross-cutting TODOs for:

- `docs/workstreams/image-support-v1.md`

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `IMG-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- file paths + key functions/tests
- and/or a `fretboard diag` script/suite name

## M0 — Lock decisions (no more “implicit stretch”)

- [x] IMG-adr-001 Draft and land an ADR for image `object-fit` semantics.
  - Must cover: fit enum vocabulary, default behavior, `ImageRegion` interaction rule, and backcompat.
  - References: `docs/adr/0119-streaming-images-and-video-surfaces.md`, `docs/adr/0004-resource-handles.md`.
  - Draft: `docs/adr/0237-image-object-fit-for-sceneop-image-v1.md`
  - Evidence:
    - `docs/adr/0237-image-object-fit-for-sceneop-image-v1.md`

- [x] IMG-guard-010 Add a minimal regression gate checklist for image work (fast vs full).
  - Fast (mechanism + renderer math):
    - `cargo fmt`
    - `cargo clippy -p fret-core -p fret-ui -p fret-render-wgpu --all-targets -- -D warnings`
    - `cargo nextest run -p fret-core image_object_fit`
    - `cargo nextest run -p fret-render-wgpu image_fit`
  - Full (user-visible shadcn outcomes):
    - `cargo nextest run -p fret-ui-shadcn media_image`
    - Optional: `cargo nextest run -p fret-ui-shadcn avatar_image_emits_cover_fit_scene_op`
    - Optional: `cargo nextest run -p fret-ui-shadcn --no-fail-fast` (may include known-non-image failures)

## M1 — Core fit semantics (mechanism layer)

- [x] IMG-core-100 Add fit semantics to the core image draw primitive (`SceneOp::Image`).
  - Evidence anchors:
    - `crates/fret-core/src/scene/mod.rs`
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/image.rs`
    - `crates/fret-core/src/scene/image_object_fit.rs` (`map_image_object_fit`)

- [x] IMG-core-101 Plumb fit through the declarative element surface (`ImageProps`) and paint path.
  - Evidence anchors:
    - `crates/fret-ui/src/element.rs`
    - `crates/fret-ui/src/declarative/host_widget/paint.rs`

- [x] IMG-test-120 Add conformance tests for fit mapping + UV crop math.
  - Prefer: pure math unit tests (no GPU required) + one renderer smoke test.
  - Must cover: clamp + monotonic UV rules and “no early rounding” guidance (ADR 0231).
  - Evidence:
    - `crates/fret-core/src/scene/image_object_fit.rs` (unit tests)
    - `crates/fret-render-wgpu/src/renderer/tests.rs` (`image_fit_*`)

## M2 — shadcn parity (user-visible win)

- [x] IMG-shadcn-200 Make `AvatarImage` default to cover semantics (no stretching).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/avatar.rs`
    - `ecosystem/fret-ui-shadcn/tests/*` (web_vs_fret snapshots)
    - `ecosystem/fret-ui-shadcn/src/avatar.rs` (`avatar_image_emits_cover_fit_scene_op`)

- [x] IMG-shadcn-210 Add a small shadcn `Image` recipe/component for cards/media rows.
  - Must support: `fit`, `loading` skeleton slot, and `fallback` slot (policy lives here).
  - Evidence:
    - `ecosystem/fret-ui-shadcn/src/media_image.rs` (`MediaImage`)
    - `ecosystem/fret-ui-shadcn/src/media_image.rs` (`media_image_emits_fit_scene_op_when_image_present`)

## M3 — Metadata query seam (optional but likely needed)

- [x] IMG-meta-300 Provide a policy-owned intrinsic metadata seam (no `UiServices` expansion).
  - Decision: do **not** add an `ImageService` to `UiServices` for v1.
  - Apps/components can record and read intrinsic dimensions via an app-global store:
    - `ecosystem/fret-ui-kit/src/image_metadata.rs` (`ImageMetadataStore`, `with_image_metadata_store_mut`)
  - Ecosystem ergonomics:
    - `ecosystem/fret-ui-shadcn/src/media_image.rs` (`intrinsic_aspect_ratio_from_metadata`)

## M4 — Ecosystem `img(source)` (deferred until M1/M2 are stable)

- [x] IMG-eco-400 Add an ecosystem crate (or module) for `ImageSource` + decode/load + cache integration.
  - Non-goal: framework-owned media engine.
  - Must integrate with: `fret-ui-assets` / `fret-asset-cache` budgets.
  - Evidence anchors:
    - `ecosystem/fret-ui-assets/src/image_source.rs` (`ImageSource`, `use_image_source_state`)
    - `ecosystem/fret-ui-assets/Cargo.toml` (`image-decode`, `image-metadata` features)
    - `apps/fret-ui-gallery/src/driver.rs` (`UiGalleryImageSourceDemoAssets`)
    - `apps/fret-ui-gallery/src/ui.rs` (`Ecosystem ImageSource (bytes decode)` section)

- [x] IMG-eco-410 Add a demo + at least one diag script for:
  - scrolling thumbnails (virtual list),
  - video-like streaming updates (existing demos ok; add UI composition around them).
  - Evidence anchors:
    - `apps/fret-ui-gallery/src/spec.rs` (`PAGE_IMAGE_OBJECT_FIT`)
    - `apps/fret-ui-gallery/src/ui.rs` (`preview_image_object_fit`)
    - `apps/fret-ui-gallery/src/driver.rs` (demo image register + streaming `ImageUpdateRgba8`)
    - `tools/diag-scripts/ui-gallery-image-object-fit-screenshots.json`

## M5 — Video fast paths (capability-gated)

- [x] IMG-video-500 Audit streaming update perf on desktop + wasm and record a baseline.
  - References: ADR 0119/0122/0124.
  - Desktop (Windows-local):
    - `tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json`
    - `docs/workstreams/perf-baselines/ui-gallery-image-object-fit.windows-local.v1.json`
    - `docs/workstreams/perf-baselines/policies/ui-gallery-image-object-fit.v1.json`
    - Command:
      - `cargo run -p fretboard -- diag perf tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json --repeat 5 --warmup-frames 5 --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-image-object-fit.windows-local.v1.json --perf-baseline-headroom-pct 20 --dir target/fret-diag-perf/ui-gallery-image-object-fit.windows-local.v1 --launch -- cargo run -p fret-ui-gallery --release`
  - WASM (web-local):
    - Export bundles:
      - `tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json`
      - `.fret/diag/exports/<exported_unix_ms>/bundle.json`
    - Baseline:
      - `docs/workstreams/perf-baselines/ui-gallery-image-object-fit.web-local.v1.json`
  - Note: `--perf-baseline-seed` / `--perf-baseline-seed-preset` are currently rejected by `fret-diag` (even though `fretboard help` mentions them), so this baseline uses the default seeding behavior.
  - WASM progress:
    - Web build compiles: `cargo build -p fret-ui-gallery-web --target wasm32-unknown-unknown`
      - Evidence: `crates/fret-platform-web/src/wasm/ime.rs` (visibility fixes for `WebImeBridge` / debug state; `Effect::OpenUrl { url, .. }` pattern)
    - Baseline workflow (web runner): run the script via `apps/fret-devtools` (or any workflow that produces exported bundles under `.fret/diag/exports/`), then generate a baseline from bundle paths:
      - `cargo run -p fretboard -- diag perf-baseline-from-bundles tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json .fret/diag/exports/<exported_unix_ms> --perf-baseline-out docs/workstreams/perf-baselines/ui-gallery-image-object-fit.web-local.v1.json`
  - Evidence anchors (desktop):
    - `tools/diag-scripts/ui-gallery-image-object-fit-perf-steady.json`
    - `docs/workstreams/perf-baselines/ui-gallery-image-object-fit.windows-local.v1.json`
    - `docs/workstreams/perf-baselines/policies/ui-gallery-image-object-fit.v1.json`
  - Evidence anchors (WASM):
    - `docs/workstreams/perf-baselines/ui-gallery-image-object-fit.web-local.v1.json`

- [ ] IMG-video-510 Decide the next capability-gated step for “external texture import”.
  - Must not leak backend handles into `fret-ui`.
