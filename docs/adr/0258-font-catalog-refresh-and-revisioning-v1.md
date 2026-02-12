# ADR 0258: Font Catalog Refresh + Revisioning (Runner/Renderer Boundary)

- Status: Proposed
- Date: 2026-02-11

## Context

Fret exposes a best-effort font catalog to settings UIs via runtime globals:

- `FontCatalog` (family list + revision)
- `FontCatalogMetadata` (entry list + revision, including best-effort axis ranges + monospace hints)
- `FontCatalogCache` (UI-friendly `Arc<[Arc<str>]>` derived from `FontCatalog`)

These globals are populated by runners from the renderer’s text backend (Parley/fontique).

Two failure modes show up quickly in editor-grade apps:

1) **Spurious invalidation**: bumping the catalog revision on every refresh attempt causes unnecessary UI churn and
   cache invalidation, even when the catalog is unchanged.
2) **Ambiguous semantics**: the revision currently means “refresh happened” instead of “effective catalog changed”,
   which makes it hard to reason about what invalidation boundaries should exist.

We want a contract where:

- **Catalog refresh is explicit** (startup, font injection, or an explicit “rescan” operation),
- **Revisions are stable** across no-op refreshes,
- **Catalog metadata remains best-effort** and platform-dependent.

## Goals

1) Make `FontCatalog.revision` and `FontCatalogMetadata.revision` represent **effective content changes**, not refresh
   attempts.
2) Reduce no-op invalidation and UI churn when runners refresh the catalog multiple times (common during bootstrap).
3) Keep the boundary portable and renderer-owned: the runtime stores snapshots; the renderer remains the source of
   truth for enumeration.

## Non-goals

- Guarantee stable, cross-machine font enumeration results.
- Require that apps expose a user-facing “system font rescan” UI for v1 (the runner may still provide an effect/command).
- Provide per-face/weight/style enumeration in the runtime contract (family-level is sufficient for v1 pickers).

## Decision

### 1) Only bump catalog revisions when the effective entry list changes

Runners update the globals via `fret-runtime::apply_font_catalog_update_with_metadata(...)`.

The update must:

- compare the incoming `Vec<FontCatalogEntry>` against the existing `FontCatalogMetadata.entries`,
- bump the revision **only if the entry list differs** (including metadata fields),
- leave the revision unchanged for a no-op refresh attempt.

This makes “revision” a reliable invalidation boundary for UIs and caches that depend on catalog content.

### 2) Treat `TextFontFamilyConfig` policy seeding as independent from catalog revisioning

Policy-driven defaults (e.g. wasm curated fallback seeding) may update `TextFontFamilyConfig` even when the catalog is
unchanged.

This must not force a catalog revision bump. The renderer already has its own invalidation keys for shaping/raster
behavior (`TextFontStackKey` and renderer-internal caches).

### 3) Refresh triggers stay explicit and bounded

Runners should refresh the catalog only on:

- initial renderer availability (startup/adopt),
- `Effect::TextAddFonts` when fonts were actually added,
- an explicit “rescan system fonts” operation (native-only; may be async).

No periodic scanning is implied by this ADR.

### 4) Catalog metadata probes must remain best-effort and budgetable

Some picker-facing metadata requires reading additional font tables (e.g. monospace hints). These probes:

- may be expensive on large font catalogs,
- must remain best-effort (safe to omit or default),
- should be controllable via debug/env knobs when diagnosing stalls.

Current knobs:

- `FRET_TEXT_FONT_CATALOG_MONOSPACE_PROBE=0` disables `post` table reads used to populate
  `FontCatalogEntry{is_monospace_candidate}`.

## Evidence: current refresh triggers (implementation)

- Desktop runner (startup): `crates/fret-launch/src/runner/desktop/app_handler.rs`
- Desktop runner (`Effect::TextAddFonts`): `crates/fret-launch/src/runner/desktop/mod.rs`
- Desktop runner (`Effect::TextRescanSystemFonts`): `crates/fret-launch/src/runner/desktop/mod.rs`
- Web runner (adopt gfx): `crates/fret-launch/src/runner/web/gfx_init.rs`
- Web runner (`Effect::TextAddFonts`): `crates/fret-launch/src/runner/web/effects.rs`

## Consequences

- Settings pickers can treat `FontCatalogMetadata.revision` as “data changed”.
- Runner bootstrap can refresh defensively without causing cascading invalidations.
- Renderer-side caching (enumeration + family id lookup) remains the primary performance lever; runtime revisioning
  ensures the snapshot boundary stays cheap and predictable.

## Implementation anchors

- Runtime no-op revisioning + policy seeding split:
  - `crates/fret-runtime/src/font_bootstrap.rs` (`apply_font_catalog_update_with_metadata`)
  - `crates/fret-runtime/src/font_catalog.rs` (revision semantics docs)
- Renderer enumeration caching (reduces refresh cost):
  - `crates/fret-render-wgpu/src/text/parley_shaper.rs` (`all_font_names_cache`, `all_font_catalog_entries_cache`)

## Workstream tracking

See `docs/workstreams/font-catalog-refresh-policy-v1.md`.
