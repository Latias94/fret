# `fret-fonts`

Bundled font bytes for bootstrapping `fret` apps, primarily targeting Web/WASM where system fonts
are not available.

Status note:

- The shipped `bootstrap_profile()` and `default_profile()` now guarantee `sans`, `serif`, and
  `monospace` whenever `bootstrap-subset` or `bootstrap-full` is enabled.
- The intentionally minimal mono-only build still avoids promising `sans` / `serif`.

## Contents

- Inter (roman + italic) — OFL 1.1 (`assets/Inter-OFL.txt`)
- Roboto Slab Variable — Apache 2.0 (`assets/RobotoSlab-LICENSE.txt`)
- JetBrains Mono (roman + italic) — OFL 1.1 (`assets/JetBrainsMono-OFL.txt`)
- Fira Mono (subset) — OFL 1.1 (`assets/FiraMono-LICENSE`)
- Noto Color Emoji — OFL 1.1 (`assets/NotoEmoji-LICENSE.txt`) (optional; `emoji` feature)

The canonical API is:

- `fret_fonts::default_profile()` / `fret_fonts::bootstrap_profile()` — manifest-backed bundled
  profile metadata (roles, expected family names, generic guarantees, fallback families).
- `fret_fonts::bundled_asset_bundle()` — package-scoped logical bundle id for the shipped font
  assets (`pkg:fret-fonts`).
- `fret_fonts::default_profile().asset_entries()` /
  `fret_fonts::bootstrap_profile().asset_entries()` — `StaticAssetEntry` iterators that publish the
  bundled faces on the shared asset contract.
- `fret_fonts::default_profile().faces` / `fret_fonts::bootstrap_profile().faces` — ordered
  bundled face specs for callers that explicitly need raw bytes or face-level metadata.
- `fret_fonts::default_profile().faces_for_role(...)` — role-scoped bundled face iteration for
  callers that need role-aware byte/metadata access without leaving the face contract.
- `BundledFontFaceSpec::asset_locator()` / `asset_request()` — face-level logical asset identity for
  resolvers, installers, or diagnostics that need to reference one bundled face explicitly.
- `fret_fonts::build_imported_font_asset_batch(...)` — prepare user-selected local font files as
  stable memory `AssetRequest`s plus resolved bytes for the runtime asset lane.
- `fret_fonts::ImportedFontAssetResolver` — mutable memory resolver for app/session-local imported
  fonts staged through the shared asset contract.
- `fret_fonts::test_support::face_blobs(...)` (feature `test-support`) — test-only helper that
  converts a chosen face iterator into owned `Vec<u8>` blobs for deterministic conformance gates.

The profile surfaces are the contract source for bundled roles, guarantees, and logical asset
identity. Framework-owned startup baselines should publish those asset entries into the shared
runtime asset resolver and resolve startup bytes from that identity before renderer injection.
Session-local imported fonts can now follow that same identity model through memory locators rather
than bypassing the resolver contract. Byte-oriented callers should stay anchored on concrete
bundled faces rather than reintroducing a separate runtime raw-byte lane. The `test_support`
module is intentionally scoped to tests and golden/conformance harnesses.

## Size strategy (WASM)

The default feature set uses **subset fonts** to reduce WASM payload size:

- `bootstrap-subset` (default): uses `*-subset.ttf` for Inter/JetBrains Mono.
- `cjk-lite` (default): adds a small subset of `Noto Sans CJK SC` for basic CJK coverage.
- `bootstrap-full`: uses the full font files (much larger).

Emoji:

- `emoji`: includes `assets/NotoColorEmoji.ttf` (large; intended as an explicit opt-in).

## CJK lite (WASM bootstrap)

- `cjk-lite`: includes a subset of `Noto Sans CJK SC` as
  `assets/NotoSansCJKsc-Regular-cjk-lite-subset.otf` (generated via `fonttools`/`pyftsubset`).
  Intended to cover a practical baseline of CJK glyphs for bootstrap UI on Web/WASM without pulling
  in the full font payload.

## Recommended bundles

For a general-purpose app shell:

- Web/WASM: keep `bootstrap-subset` + `cjk-lite` on by default, and gate `emoji` behind an explicit
  feature or user setting (WASM size impact is significant).
- Native: either rely on system UI fonts (plus explicitly user-imported local fonts) or use `bootstrap-full` for a
  deterministic demo experience.

For future expansion, prefer feature-gated bundles (e.g. CJK subsets) rather than growing the
shipped default profile unconditionally.
