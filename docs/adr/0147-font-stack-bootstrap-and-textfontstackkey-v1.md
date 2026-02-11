# ADR 0147: Font Stack Bootstrap and `TextFontStackKey` (v1)

- Status: Proposed
- Date: 2026-01-13

## Context

Fret’s stable text boundary is locked (`TextBlobId` + `TextMetrics` + geometry queries; ADR 0006 / ADR 0045 / ADR 0046).
The renderer text system v2 direction is also locked (Parley shaping + wrapper-owned wrap/ellipsis + attributed spans; ADR 0142).

What is not yet locked is the **font bootstrap + invalidation contract** across platforms:

- Web/WASM cannot access system fonts; the app must inject font bytes.
- Native platforms can enumerate system fonts, but fallback behavior and “system UI font” choice is platform-dependent.
- The renderer caches shaped layouts and glyph rasters; stale caches after font mutations are correctness bugs (missing glyphs, wrong metrics, wrong caret mapping).

The repository already has globals intended for this boundary:

- `FontCatalog` (settings UI / family picker surface).
- `TextFontFamilyConfig` (user-configurable family candidates).
- `TextFontStackKey` (stable key that participates in renderer text cache keys).

Runner wiring must remain consistent (e.g. web runner and desktop runner should publish the same globals through the same helper),
and Web/WASM needs a deterministic bootstrap story because system font discovery is not available.

GPUI/Zed provides a useful reference: it has an explicit `fallback_font_stack` and a platform text system boundary, ensuring that
font resolution failures degrade predictably and that fallback remains deterministic (see `repo-ref/zed/crates/gpui/src/text_system.rs`).

## Goals

1) Make font selection predictable and portable (native + wasm), especially under IME provisional glyphs.
2) Lock the `TextFontStackKey` contract so cache invalidation is always correct.
3) Standardize runner responsibilities (one canonical bootstrap/update path).
4) Define a minimal Web/WASM font bootstrap story that supports UI text out of the box.
5) Keep room for future extensions: user font loading, per-script fallbacks, emoji policy, variable fonts.

## Decision

### 1) Define the meaning of the globals

`FontCatalog`:

- Best-effort list of family names available to the renderer backend on the current platform.
- Used for settings UIs and diagnostics only (not authoritative for shaping).

`TextFontFamilyConfig`:

- User-facing *preference lists* for the three generic families:
  - `ui_sans`
  - `ui_serif`
  - `ui_mono`
- The renderer resolves each list to an effective family name by selecting the first installed/available family.
- An empty list means “use the platform/system default mapping” (implementation-defined).
- `common_fallback` is an ordered list of extra family candidates appended to the framework fallback
  chain (used for mixed-script “no tofu” behavior without per-span font selection).

`TextFontStackKey`:

- A renderer-provided, stable key representing the **current effective font stack and fallback configuration**.
- This key participates in text cache keys (`TextBlobKey` / layout caches / raster caches) so changing fonts or fallbacks cannot reuse stale shaping results.

### 2) Runner responsibilities (canonical path)

Runners must propagate font changes into globals, consistently across native and web:

1. When the renderer backend becomes available (bootstrap), and whenever fonts/configuration change:
   - Refresh `FontCatalog` from the renderer (`renderer.all_font_names()`).
   - Publish the chosen/updated `TextFontFamilyConfig`.
   - Publish `TextFontStackKey(renderer.text_font_stack_key())`.

2. Prefer using the shared helper for catalog/config publication where possible:
   - `fret_runtime::apply_font_catalog_update(...)` is the canonical place for future bootstrap policy (see Open Questions).

Notes:

- The runner must treat `TextFontStackKey` as authoritative. The renderer is the only component that knows which knobs affect shaping/metrics.
- It is valid for the renderer to bump its internal revision even when an upstream key does not change (e.g. Parley/fontique generic mapping changes).

### 3) Web/WASM font bootstrap

Baseline contract for Web/WASM:

- The runner must inject at least one bundled font set on startup via `Effect::TextAddFonts`.
- The `fret-fonts` crate provides **role-based bundles** (not “complete coverage”).

Minimum requirement for `fret-fonts::default_fonts()` (v1):

- one **UI sans** font (or subset) suitable for labels and UI controls,
- one **monospace** font (or subset) for developer tooling / code-like surfaces,
- optional: an **emoji/color** fallback font for deterministic emoji rendering,
- optional: a **CJK** fallback font for "no tofu" baseline in East Asian UIs.

This ADR does not mandate which exact font families are bundled; it mandates the roles and the bootstrap wiring.

#### Bundled font tiers (recommended)

To keep WASM payload size controllable while still supporting real apps, `fret-fonts` is expected to expose bundles as
separate feature flags:

- **Bootstrap**: small UI sans + monospace baseline (recommended default for web demos and starter templates).
- **Emoji**: a color emoji font bundle (opt-in; can be large).
- **CJK lite**: a small subset to cover common simplified Chinese UI strings (opt-in; can be medium).

The runner should allow toggling these tiers via crate features (e.g. `fret-launch` / `fret-demo-web`) or app settings.

### 4) `TextFontStackKey` invalidation contract

`TextFontStackKey` must change whenever the effective shaping/metrics output can change due to font configuration mutations, including:

- user-loaded font bytes added/removed,
- changes to the resolved generic family names (after applying `TextFontFamilyConfig`),
- changes to fallback policy inputs (locale, platform fallback lists, or framework fallback stacks),
- backend font registry changes that affect font face identity mapping (e.g. Parley fontique family remapping),
- renderer-defined policy knobs that affect glyph rasterization selection (when they are tied to font selection).

`TextFontStackKey` must **not** change for theme-only paint changes (color/decorations).

## Consequences

- Text caches become correct under font mutations (no reuse of stale shaping/layout).
- Web/WASM gets a deterministic bootstrap story rather than relying on system fonts.
- The runner/renderer contract becomes explicit and auditable.

## Open Questions / Follow-ups

1) Bootstrap policy in `apply_font_catalog_update`:
   - The current `FillIfEmpty` policy is not a good default for user-visible configs (it can explode configs to “all families”).
   - Decide a better policy vocabulary (e.g. “use platform defaults” vs “seed with curated candidates”).

2) Unifying font resolution sources:
   - Today the renderer may bridge multiple libraries (e.g. `cosmic-text` fontdb + Parley/fontique).
   - Long term we should converge on a single source of truth for generic families + fallback ordering to avoid “key vs behavior” drift.

3) Emoji policy:
   - Baseline pipeline and cache-key rules are tracked by ADR 0152.
   - Keep conformance coverage for variation selectors, ZWJ sequences, flags, and keycaps.

4) CJK baseline:
   - Decide whether we want additional region/script bundles beyond "cjk-lite" (JP/KR/TC, Arabic, Devanagari, etc.),
     and keep them opt-in to avoid unbounded wasm payload growth.
