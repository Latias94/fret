# Workstream: Font Catalog Refresh Policy v1

Status: In progress

This workstream focuses on the runner/runtime boundary contract for **font catalog refresh**, **revisioning**, and the
minimal caching required to keep settings pickers stable in editor-grade apps.

Primary text pipeline tracker remains: `docs/workstreams/text-system-v2-parley.md`.

## Why a dedicated workstream?

Font selection/fallback correctness (ADR 0257) and picker metadata (catalog entries) are related, but the “refresh and
revisioning” policy is an easy place for accidental churn to leak in. Keeping this as a focused tracker helps:

- make invalidation boundaries explicit,
- avoid “refresh happened” being treated as “data changed”,
- stage a future “system font rescan” without threading it through unrelated text work.

## Milestones

### M0 (landed): no-op refresh does not bump revision

Exit criteria:

- `FontCatalogMetadata.revision` only changes when `entries` changes.
- Policy seeding of `TextFontFamilyConfig` can happen without bumping the catalog revision.

Evidence:

- `crates/fret-runtime/src/font_bootstrap.rs` (`apply_font_catalog_update_with_metadata`)
- test: `crates/fret-runtime/src/font_bootstrap.rs` (`apply_update_with_metadata_does_not_bump_revision_when_entries_unchanged`)

### M1 (next): make refresh triggers explicit in user-facing docs

Goals:

- Document when runners refresh the catalog (startup/adopt + `TextAddFonts`).
- Ensure the UI gallery / diagnostics tooling does not accidentally depend on “refresh attempt count”.

Evidence (current triggers):

- Desktop runner (startup): `crates/fret-launch/src/runner/desktop/app_handler.rs`
- Desktop runner (`Effect::TextAddFonts`): `crates/fret-launch/src/runner/desktop/mod.rs`
- Web runner (adopt gfx): `crates/fret-launch/src/runner/web/gfx_init.rs`
- Web runner (`Effect::TextAddFonts`): `crates/fret-launch/src/runner/web/effects.rs`

### M2 (optional): explicit system font rescan surface (native-only)

Candidate shape:

- new runtime effect/command (runner-owned) that triggers a platform font rescan and re-applies metadata if changed.

Status:

- Landed: `Effect::TextRescanSystemFonts` is handled by the desktop runner and ignored on web.

Evidence:

- Effect contract: `crates/fret-runtime/src/effect.rs` (`TextRescanSystemFonts`)
- Desktop wiring: `crates/fret-launch/src/runner/desktop/mod.rs`
- Web wiring: `crates/fret-launch/src/runner/web/effects.rs`
- Renderer rescan: `crates/fret-render-wgpu/src/text/mod.rs` (`TextSystem::rescan_system_fonts`)

Open questions:

- Whether rescans should be blocking, async, or “eventually consistent” (likely async).
- How to keep WASM deterministic (likely “no system rescan”, only `add_fonts`).

## Related docs

- ADR: `docs/adr/0258-font-catalog-refresh-and-revisioning-v1.md`
- Font-system contract: `docs/adr/0257-font-selection-fallback-and-variable-font-instances-v1.md`
- Audit: `docs/audits/font-system-parley-zed-xilem-2026-02.md`
- Workstream: `docs/workstreams/font-system-v1.md`
