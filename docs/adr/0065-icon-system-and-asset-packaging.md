---
title: "ADR 0065: Icon System and Asset Packaging (SVG-First, Semantic Keys)"
---

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- gpui-component: https://github.com/longbridge/gpui-component
- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

# ADR 0065: Icon System and Asset Packaging (SVG-First, Semantic Keys)

## Status

Accepted.

## Context

Fret targets editor-grade UI (Unity/Unreal/Godot-like): docking, multi-window tear-off, multiple viewports, and
high-density surfaces (toolbars, inspector rows, trees, tables).

Icons are pervasive in such UIs. We need an icon solution that is:

- Theme-friendly (tintable monochrome icons are the common case).
- Portable (Windows/macOS/Linux now; future wasm).
- Performance-aware (avoid per-frame CPU-heavy work; allow caching/budgets).
- Maintainable at scale (hundreds/thousands of icons; multiple icon sets).
- Binary-size conscious (do not force embedding large icon sets by default).

Fret already supports SVG rendering in a GPUI-aligned way:

- `SceneOp::SvgMaskIcon` (alpha-mask + tint) and `SceneOp::SvgImage` (RGBA image) in `fret-core`.
- Renderer-managed SVG raster caches/atlases/budgets in `fret-render`.

Separately, the components layer needs a stable “what icon is this?” vocabulary that does not leak upstream asset
names or rendering representation.

GPUI Component provides a strong precedent: `IconName`/`Icon` exist in the UI crate, while SVG assets are split into a
separate optional assets crate (not bundled by default).

## Decision

We define an **SVG-first** icon system for Fret components with three strict separations:

1) **Icon identity is semantic and renderer-agnostic** (component vocabulary).
2) **Icon assets are optional and packaged separately** (binary-size control).
3) **Icon rendering uses existing SVG primitives** (renderer owns caching/budgets).

### 1) Icon identity: semantic keys

`fret-icons` provides an `IconId` that is a stable semantic key (string-like), not tied to a specific icon
set or upstream filename.

Naming guidelines:

- Use a dot-separated namespace: `domain.action` or `surface.role`.
- Prefer meaning over shape: `panel.close`, `toolbar.search`, `status.warning`.
- Use shape keys only for truly generic glyphs: `chevron.down`, `chevron.right`.
- Do not encode vendor names in app/component code (avoid `lucide.close`).

Ergonomics:

- Prefer using shared constants for common IDs (e.g. `fret_components_icons::ids::*`) to avoid typos and to make
  cross-crate refactors safer.

### 2) Icon registry: definition is data, not rendering

`fret-icons` also provides an `IconRegistry` mapping `IconId -> IconDefinition`.

`IconDefinition` is renderer-agnostic and groups:

- `IconSource`
  - SVG bytes (static/owned),
  - aliases (for migration and compatibility),
- optional fallback data (for example glyph fallback in bootstrap/minimal builds),
- `IconPresentation`
  - including the preferred render mode (`Mask` vs `OriginalColors`).

Rule:

- `IconSource` stays source data only; it must not become a rendering-policy bag.
- Aliases resolve to the full target icon definition, not only to raw bytes.

The registry supports layering (app overrides component defaults) and safe fallbacks:

- If an icon is missing, components must render a deterministic fallback (e.g. a “missing icon” glyph) rather than
  failing silently.
- Vendor icon IDs remain explicit namespaces (for example `lucide.*`, `radix.*`) and do not conflict with each other.
- Semantic aliases (for example `ui.search`) should be registered with
  `IconRegistry::alias_if_missing(...)`, which makes the first successfully registered provider the
  stable default.
- App/bootstrap code may intentionally override a semantic alias afterwards with
  `IconRegistry::alias(...)`, `IconRegistry::register(...)`, or `IconRegistry::register_icon(...)`;
  this is the explicit escape hatch for product-specific icon choices.

### 3) Asset packaging: icon sets are separate crates (and/or features)

Large icon sets (e.g. Lucide/Radix) are **not embedded by default** in core component crates.

Instead, provide dedicated icon-pack crates (examples):

- `fret-icons-editor` (small editor-focused set, curated).
- `fret-icons-lucide` (Lucide SVGs, curated in-repo).
- `fret-icons-radix` (Radix icons, curated in-repo).

In this repository, Lucide and Radix are maintained as full vendored sets generated from upstream-pinned submodule
sources to keep updates deterministic and maintenance low. Other icon packs may still choose curated subsets where
that is the more practical trade-off.

Each icon-pack crate exports a single registration entrypoint:

- `pub fn register_icons(reg: &mut IconRegistry)`

Applications choose dependencies (or features) and call `register_icons` during initialization.

Icon packs may also register a vendor-prefixed `IconId` namespace that mirrors upstream naming (e.g. `lucide.x`,
`radix.gear`) to make it easy for application code to use upstream icon names directly. Core component code should
continue to rely on semantic IDs (e.g. `ui.close`) for portability.

App-facing convenience wrappers may exist above this low-level registration seam (for example
`crate::app::install(...)` helpers or named `InstallIntoApp` bundles). Reusable crates should keep
those wrappers explicit and grep-friendly rather than requiring app authors to manually replay
internal icon/asset registration steps.

Pack crates should also keep provenance explicit in code rather than only in README prose:

- export `PACK_METADATA: IconPackMetadata`,
- export one or more data-first registration values (`PACK`, `VENDOR_PACK`, and optional
  semantic-alias registration values),
- and let app/bootstrap layers record installed-pack metadata separately from the renderer-agnostic
  `IconRegistry`.

### 3.5) Relationship to the general asset contract

The framework currently uses a deliberate hybrid model:

- icon semantics and vendor naming resolve through `IconRegistry`,
- generic shipped bytes (images, SVG illustrations, fonts, JSON, etc.) resolve through the general
  asset contract (`AssetBundleId`, `AssetLocator`, resolver layers, generated asset modules).

Rules:

- component code consumes `IconId`; it must not hard-code raw icon file paths or bundle keys,
- reusable crates that ship non-icon bytes alongside icons should publish those bytes as
  package-owned assets (`AssetBundleId::package(...)`) and expose a named installer/bundle on the
  app surface instead of asking apps to reproduce the crate's internal mounts,
- icon-pack crates remain free to vendor SVG bytes internally; those bytes are not a public
  app-facing locator contract unless the crate explicitly chooses to publish them as package assets,
- future unification between icon-pack bytes and the general asset contract is allowed, but it must
  preserve the semantic/vendor `IconId` layer as the component-facing API.

### Rendering contract (components layer)

Component-layer helpers (e.g. `fret-ui-kit` `Icon` / `IconButton`) render an `IconId` by resolving
`ResolvedIcon`:

- Themed icon path: SVG bytes + `IconRenderMode::Mask` ->
  `fret_ui::SvgSource` + declarative `SvgIcon` element (`ElementContext::svg_icon`), emitting
  `SceneOp::SvgMaskIcon`.
- Original-color path: SVG bytes + `IconRenderMode::OriginalColors` ->
  declarative `SvgImage` element (`ElementContext::svg_image`), emitting `SceneOp::SvgImage`.
- Fallback: glyph -> existing text pipeline.

No component code holds `ImageId` or manages raster caches directly.

### Optional optimization: preload SVG ids

To avoid per-frame SVG registration when rendering icons, apps may preload icon SVG bytes into
stable `SvgId`s at the renderer boundary (for example during `WinitAppBuilder::on_gpu_ready`).

In-tree helper:

- `fret_ui_kit::declarative::icon::preload_icon_svgs(app, renderer_as_ui_services)` (feature `icons`)

This stores `IconId -> (SvgId, IconPresentation)` in an `IconSvgRegistry` global so SVG-bearing
icon helpers can emit `SvgSource::Id` directly while still preserving authored render mode.

## Consequences

- Icon usage in components/app code becomes stable and comparable to “frontend semantics-first” practices.
- Icon sets become swappable without changing app/component call sites.
- Multiple icon packs can coexist deterministically:
  - vendor IDs stay namespaced,
  - semantic `ui.*` aliases default to first-writer-wins unless app/bootstrap code explicitly
    overrides them.
- Renderer retains authority over SVG performance (atlas/page budgeting, caching keys, eviction policies).
- Binary size is controlled by dependencies (and/or features), not by pulling in a huge default icon set.

## Migration Plan (Non-breaking)

- Existing ad-hoc keys (e.g. `close`, `search`) can be preserved as aliases to semantic keys (e.g. `close` ->
  `panel.close`) inside `IconRegistry`.
- Migrate call sites incrementally toward semantic keys; keep aliases until downstream codebases converge.

## References

- Fret SVG icon MVP notes: `docs/archive/mvp-svg-icons.md`
- Vector graphics roadmap: `docs/archive/mvp-vector-graphics-roadmap.md`
- Shape/SDF semantics (renderer implementation detail policy): `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
- GPUI Component icon + assets split: `repo-ref/gpui-component/crates/ui/src/icon.rs`,
  `repo-ref/gpui-component/crates/assets/src/lib.rs`
- Zed icon theme registry and schema (non-normative):
  - `repo-ref/zed/crates/theme/src/icon_theme.rs`, `repo-ref/zed/crates/theme/src/icon_theme_schema.rs`

## In-repo maintenance (Lucide)

- Source-of-truth upstream: `third_party/lucide` (git submodule)
- Generated list: `ecosystem/fret-icons-lucide/icon-list.txt`
- Generated constants: `ecosystem/fret-icons-lucide/src/generated_ids.rs`
- Generate list/constants from submodule:
  - Windows/macOS/Linux: `python3 tools/gen_lucide.py`
  - Unified entrypoint: `python3 tools/gen_icons.py --pack lucide`
- Sync vendored SVGs from upstream sources:
  - `python3 tools/sync_icons.py --pack lucide --clean`
  - macOS/Linux: `python3 tools/sync_icons.py --pack lucide --clean`
- Verify vendor icon references resolve to vendored assets:
  - Windows/macOS/Linux: `python3 tools/verify_icons.py --strict`
- CI consistency gate:
  - `python3 tools/check_icons_generation.py --pack lucide`

## In-repo maintenance (Radix)

- Source-of-truth upstream: `third_party/radix-icons` (git submodule)
- Generated list: `ecosystem/fret-icons-radix/icon-list.txt`
- Generated constants: `ecosystem/fret-icons-radix/src/generated_ids.rs`
- Generate list/constants from submodule:
  - Windows/macOS/Linux: `python3 tools/gen_radix.py`
  - Unified entrypoint: `python3 tools/gen_icons.py --pack radix`
- Sync vendored SVGs from upstream sources:
  - `python3 tools/sync_icons.py --pack radix --clean`
  - macOS/Linux: `python3 tools/sync_icons.py --pack radix --clean`
- Verify vendor icon references resolve to vendored assets:
  - Windows/macOS/Linux: `python3 tools/verify_icons.py --strict`
- CI consistency gate:
  - `python3 tools/check_icons_generation.py --pack radix`

## Vendor update quick flow (pre-release)

- Update submodules:
  - `git submodule update --init --recursive`
  - `git submodule update --remote third_party/lucide third_party/radix-icons`
- Regenerate both packs:
  - `python3 tools/gen_icons.py --pack all`
- Run unified consistency check:
  - `python3 tools/check_icons_generation.py --pack all`
- Optional release gate:
  - `python3 tools/pre_release_icons.py`
