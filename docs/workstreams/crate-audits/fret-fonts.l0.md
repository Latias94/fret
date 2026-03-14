# Crate audit (L0) — `fret-fonts`

## Crate

- Name: `fret-fonts`
- Path: `crates/fret-fonts`
- Owners / adjacent crates: `fret-render-text`, `fret-launch`, `fret-runtime`
- Current layer: bundled font asset + manifest contract

## 1) Purpose (what this crate *is*)

- A portable, dependency-light crate that packages bundled font bytes and exposes a small profile
  manifest (`bootstrap_profile`, `default_profile`) for the rest of the text stack.
- The crate should own bundled asset composition and role metadata only; it should not absorb
  locale policy, platform fallback heuristics, or runner-specific startup behavior.
- Its primary contract is "which families/roles/generic guarantees a bundled profile provides",
  plus the raw bytes that can be injected into the renderer.

Evidence anchors:

- `crates/fret-fonts/Cargo.toml`
- `crates/fret-fonts/src/lib.rs`

## 2) Public contract surface

- Key exports / stable types:
  - `BundledFontRole`
  - `BundledGenericFamily`
  - `BundledFontFaceSpec`
  - `BundledFontProfile`
  - `bootstrap_profile`, `default_profile`, `bootstrap_fonts`, `default_fonts`
- Narrowed contract guidance:
  - role-scoped byte access should go through `BundledFontProfile::font_bytes_for_role(...)`, so
    downstream code stays anchored on the manifest/profile contract instead of top-level helper
    slices.
- Feature flags and intent:
  - default = `bootstrap-subset + cjk-lite`
  - optional expansion flags: `bootstrap-full`, `emoji`, `cjk-lite`
  - the feature matrix is effectively part of the asset contract and needs explicit gating.

Evidence anchors:

- `crates/fret-fonts/src/lib.rs`
- `crates/fret-fonts/Cargo.toml`

## 3) Dependency posture

- Backend coupling risks:
  - none in normal builds; the crate has no non-dev dependencies.
- Layering policy compliance:
  - excellent; this crate is fully portable and asset-focused.
- Compile-time / maintenance hotspots:
  - the crate now has explicit `assets / profiles / tests` seams, but the feature matrix is still
    handwritten Rust constants rather than a generated manifest.

Evidence anchors:

- `crates/fret-fonts/Cargo.toml`
- `crates/fret-fonts/src/lib.rs`
- `crates/fret-fonts/src/{assets,profiles,tests}.rs`
- `python tools/audit_crate.py --crate fret-fonts`

## 4) Module ownership map (internal seams)

- Bundled role/profile contract
  - Files: `crates/fret-fonts/src/lib.rs` (`BundledFontRole`, `BundledGenericFamily`,
    `BundledFontProfile`)
- Feature-matrix asset wiring
  - Files: `crates/fret-fonts/src/assets.rs` (`*_BYTES`, `*_FACE`, `BOOTSTRAP_FACES`,
    `DEFAULT_FACES`), `crates/fret-fonts/src/profiles.rs` (`*_PROFILE_NAME`,
    `BOOTSTRAP_PROFILE`, `DEFAULT_PROFILE`)
- Byte collection helpers
  - Files: `crates/fret-fonts/src/profiles.rs` (`collect_font_bytes`, `default_fonts`,
    `bootstrap_fonts`), `crates/fret-fonts/src/lib.rs` (`BundledFontProfile::font_bytes_for_role`)
- Manifest / asset conformance tests
  - Files: `crates/fret-fonts/src/tests.rs` (`bundled_profiles_are_manifest_consistent`,
    `bundled_face_family_names_match_name_tables`, `default_fonts_total_size_is_reasonable`)

## 5) Refactor hazards (what can regress easily)

- Feature-matrix drift between declared profile metadata and actual bundled faces
  - Failure mode: profile names, provided roles, or expected family lists silently diverge from
    the bytes behind the manifest.
  - Existing gates: `bundled_profiles_are_manifest_consistent`,
    `bundled_face_family_names_match_name_tables`,
    `python tools/check_fret_fonts_feature_matrix.py`.
  - Remaining follow-up: wire the feature-matrix script into CI once the broader font-mainline
    workstream is ready for promotion.
- Asset budget drift across feature combinations
  - Failure mode: WASM/bootstrap payload grows without an intentional contract update.
  - Existing gates: `default_fonts_total_size_is_reasonable`.
  - Missing gate to add: split out explicit budget checks for `bootstrap_fonts()` vs
    `default_fonts()` so bootstrap-only regressions cannot hide behind the larger default profile.
- Handwritten feature-matrix pressure
  - Failure mode: future bundle additions still require touching multiple handwritten constant sets
    across `assets.rs` and `profiles.rs`, making omissions possible even after the module split.
  - Existing gates: manifest consistency tests catch some ordering and role drift.
  - Remaining follow-up: a generated manifest would still reduce future hand-maintained cfg drift.
- Policy leakage into the asset crate
  - Failure mode: platform fallback defaults or startup policy get added here because the profile
    data is nearby.
  - Existing gates: dependency posture is currently clean.
  - Missing gate to add: keep `fret-fonts` in layering checks and review new deps aggressively.

## 6) Code quality findings (Rust best practices)

- The strongest part of the crate is its low dependency footprint and direct test coverage over the
  bundled manifest contract.
- The first ownership split is now landed: `lib.rs` is a facade and the crate has explicit
  `assets / profiles / tests` seams.
- The highest remaining refactor risk is not algorithmic complexity but manual feature-matrix
  maintenance: dozens of `#[cfg(...)]` constants still encode profile names, face sets, and
  provided-role lists.
- No `unsafe` usage, no UI-thread work, and no serialization hazards were found in the crate.

Evidence anchors:

- `crates/fret-fonts/src/lib.rs`
- `crates/fret-fonts/src/tests.rs` (`bundled_profile_matrix_covers_ui_and_monospace_contracts`,
  `bundled_profile_matrix_covers_emoji_fallback_contract`,
  `bundled_profile_matrix_covers_cjk_fallback_contract`)

## 7) Recommended refactor steps (small, gated)

1. Split `src/lib.rs` into `profiles.rs`, `assets.rs`, and `tests.rs` — outcome: asset matrix
   changes become reviewable — status: landed — gate: `cargo nextest run -p fret-fonts`,
   `cargo check -p fret-fonts --no-default-features`,
   `cargo check -p fret-fonts --features bootstrap-full,emoji,cjk-lite`.
2. (Done) Add a representative feature-matrix gate for the crate — outcome: bundled profile drift
   is caught before integration — gate: `python tools/check_fret_fonts_feature_matrix.py`.
3. (Done) Narrow role-scoped byte access behind `BundledFontProfile::font_bytes_for_role(...)` —
   outcome: profiles remain the primary public contract, and raw role slices no longer define the
   crate surface — gate: `cargo nextest run -p fret-fonts`, dependent text-test crates compile
   against the profile-based accessor.

## 8) Open questions / decisions needed

- Should the bundled profile matrix stay handwritten Rust constants, or should it become a generated
  manifest derived from the subsetting scripts under `crates/fret-fonts/scripts/`?
- `BundledFontProfile` is now the supported entrypoint for role-scoped bundled byte access.
