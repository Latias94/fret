# Workstream: Font System Audit Closures (Zed / Parley / Xilem) v1

Status: Planned

This workstream turns the Feb 2026 font-system audit into a concrete refactor plan with milestones.

Scope (renderer-owned):

- **Font enumeration** (picker-facing catalog and metadata)
- **Caching** (family resolution, fallback lookup, instance identity)
- **Fallback chain** (script+locale, curated overrides, determinism)
- **Variable fonts** (axes metadata + per-span instance identity)

Primary audit input: `docs/audits/font-system-parley-zed-xilem-2026-02.md`.
Related ADRs: `docs/adr/0257-font-selection-fallback-and-variable-font-instances-v1.md`,
`docs/adr/0258-font-catalog-refresh-and-revisioning-v1.md`, `docs/adr/0259-system-font-rescan-and-injected-font-retention-v1.md`.

## Audit conclusions (summary)

1) **Treat fallback policy as a first-class “renderer policy object”**:
   - inputs must be explicit (locale, system-font availability, curated override tier, injection mode),
   - policy must produce a stable “fingerprint” that participates in `TextFontStackKey`,
   - policy must be exportable into diag bundles (snapshot + key).

2) **Keep picker UX stable by separating “catalog snapshot” from “selection mechanism”**:
   - runners can refresh/seed the picker catalog asynchronously on desktop,
   - revisioning rules must keep “refresh attempt” from looking like “data changed”.

3) **Cache expensive resolution at the right boundary**:
   - `family_name(lowercase) -> FamilyId`,
   - `(generic stack config, policy fingerprint) -> resolved family ids`,
   - (optional) `(script, locale, policy fingerprint) -> fallback family ids`.

4) **Variable fonts need two representations**:
   - normalized coords remain the authoritative instance identity input (cache keys + rasterization),
   - axis-tag + value lists should be recorded as debug metadata to make regressions explainable.

## Milestones

### M0: Mixed-script fallback conformance gate

Exit criteria:

- A `fretboard diag` script reproduces a mixed-script text case (Latin + CJK + emoji + RTL) in `fret-ui-gallery`.
- A gate asserts the fallback policy key is stable and that “missing glyphs/tofu” does not regress.

Evidence anchors:

- Diag scripts: `tools/diag-scripts/`
- Gates: `crates/fret-diag/src/stats.rs`
- Bundle fields: `UiResourceCachesV1.render_text_fallback_policy` and `render_text_font_trace`

### M1: Catalog snapshot + metadata expansion (axes ranges)

Exit criteria:

- `FontCatalogEntryMetadata` includes best-effort variable-axis info (tag + min/max/default) for the default face.
- Enumeration stays bounded: avoid forcing full table scans on the UI thread.

Evidence anchors:

- Renderer catalog extraction: `crates/fret-render-wgpu/src/text/parley_shaper.rs`
- Runtime catalog globals: `crates/fret-runtime/src/font_catalog.rs`

### M2: Policy composition API (renderer-internal)

Exit criteria:

- A single renderer-internal “fallback policy object” is responsible for composing:
  - requested family/generic,
  - script + locale fallback tier,
  - curated override tiers (wasm deterministic baseline),
  - common-fallback injection mode.
- The object provides:
  - a stable `fallback_policy_key`,
  - an “explain” snapshot for diagnostics.

Evidence anchors:

- `crates/fret-render-wgpu/src/text/mod.rs` (`fallback_policy_snapshot`, key derivation)

### M3: Performance hardening (budgets + async)

Exit criteria:

- Desktop catalog refresh and system font rescan remain async-by-default and request-coalesced.
- Injected font retention remains deduped and budgeted.
- Remaining “expensive” probes (e.g. monospace detection) are cached or moved off the UI thread.

Evidence anchors:

- Desktop runner async rescan: `crates/fret-launch/src/runner/desktop/mod.rs`
- Registered blob retention: `crates/fret-render-wgpu/src/text/parley_shaper.rs`

## Notes / upstream patterns

- Zed (cosmic-text) caches `FontId`s per-family and treats fallback as inherently dynamic; tracing “what was picked” is
  essential (`repo-ref/zed/crates/gpui/src/platform/linux/text_system.rs`).
- Parley’s design explicitly frames font selection/fallback as script+locale aware and “no tofu” oriented
  (`repo-ref/parley/doc/design.md`).
- Xilem demonstrates variable font usage via weight; a v1 API can keep most axis usage implicit while keeping instance
  identity correct (`repo-ref/xilem/xilem/examples/variable_clock.rs`).

