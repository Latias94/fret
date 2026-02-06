---
title: "ADR 0065: Icon System and Asset Packaging (SVG-First, Semantic Keys)"
---

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

### 2) Icon registry: source is data, not rendering

`fret-icons` also provides an `IconRegistry` mapping `IconId -> IconSource`.

`IconSource` is renderer-agnostic and supports:

- SVG bytes (static/owned).
- Glyph fallback (for bootstrap/minimal builds).
- Aliases (for migration and compatibility).

The registry supports layering (app overrides component defaults) and safe fallbacks:

- If an icon is missing, components must render a deterministic fallback (e.g. a “missing icon” glyph) rather than
  failing silently.

### 3) Asset packaging: icon sets are separate crates (and/or features)

Large icon sets (e.g. Lucide/Radix) are **not embedded by default** in core component crates.

Instead, provide dedicated icon-pack crates (examples):

- `fret-icons-editor` (small editor-focused set, curated).
- `fret-icons-lucide` (Lucide SVGs, curated in-repo).
- `fret-icons-radix` (Radix icons, curated in-repo).

In this repository, Lucide is maintained as a full vendored set generated from an upstream-pinned submodule source
to keep updates deterministic and maintenance low. Other icon packs may still choose curated subsets where that is
the more practical trade-off.

Each icon-pack crate exports a single registration entrypoint:

- `pub fn register_icons(reg: &mut IconRegistry)`

Applications choose dependencies (or features) and call `register_icons` during initialization.

Icon packs may also register a vendor-prefixed `IconId` namespace that mirrors upstream naming (e.g. `lucide.x`,
`radix.gear`) to make it easy for application code to use upstream icon names directly. Core component code should
continue to rely on semantic IDs (e.g. `ui.close`) for portability.

### Rendering contract (components layer)

Component-layer helpers (e.g. `fret-ui-kit` `Icon` / `IconButton`) render an `IconId` by resolving
`IconSource`:

- Preferred: SVG bytes -> `fret_ui::SvgSource` + declarative `SvgIcon` element (`ElementContext::svg_icon`), emitting `SceneOp::SvgMaskIcon`.
- Optional: for multi-color assets -> render as an `ImageId` (renderer-owned rasterization/caching).
- Fallback: glyph -> existing text pipeline.

No component code holds `ImageId` or manages raster caches directly.

### Optional optimization: preload SVG ids

To avoid per-frame SVG registration when rendering icons, apps may preload icon SVG bytes into
stable `SvgId`s at the renderer boundary (for example during `WinitAppBuilder::on_gpu_ready`).

In-tree helper:

- `fret_ui_kit::declarative::icon::preload_icon_svgs(app, renderer_as_ui_services)` (feature `icons`)

This stores `IconId -> SvgId` in an `IconSvgRegistry` global so `fret-ui-kit` icon helpers can
emit `SvgSource::Id` directly.

## Consequences

- Icon usage in components/app code becomes stable and comparable to “frontend semantics-first” practices.
- Icon sets become swappable without changing app/component call sites.
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
- Sync vendored SVGs from upstream sources:
  - Windows: `pwsh tools/sync_icons.ps1 -Pack lucide -Clean`
  - macOS/Linux: `python3 tools/sync_icons.py --pack lucide --clean`
- Verify vendor icon references resolve to vendored assets:
  - Windows/macOS/Linux: `python3 tools/verify_icons.py --strict`
- CI consistency gate:
  - `pwsh -NoProfile -File tools/check_lucide_generation.ps1`
