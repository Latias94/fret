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
- "Accidental" exports to consider removing:
  - role-specific byte helpers (`emoji_fonts`, `cjk_lite_fonts`) are convenient, but they also
    invite downstream code to reason about bundle composition at too low a level.
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
  - the entire public surface and feature matrix currently live in a single `src/lib.rs`
    (~884 LOC from the audit snapshot), which concentrates feature drift risk in one file.

Evidence anchors:

- `crates/fret-fonts/Cargo.toml`
- `crates/fret-fonts/src/lib.rs`
- `python tools/audit_crate.py --crate fret-fonts`

## 4) Module ownership map (internal seams)

- Bundled role/profile contract
  - Files: `crates/fret-fonts/src/lib.rs` (`BundledFontRole`, `BundledGenericFamily`,
    `BundledFontProfile`)
- Feature-matrix asset wiring
  - Files: `crates/fret-fonts/src/lib.rs` (`*_BYTES`, `*_FACE`, `*_PROFILE_NAME`,
    `BOOTSTRAP_PROFILE`, `DEFAULT_PROFILE`)
- Byte collection helpers
  - Files: `crates/fret-fonts/src/lib.rs` (`collect_font_bytes`, `default_fonts`,
    `bootstrap_fonts`, `emoji_fonts`, `cjk_lite_fonts`)
- Manifest / asset conformance tests
  - Files: `crates/fret-fonts/src/lib.rs` (`bundled_profiles_are_manifest_consistent`,
    `bundled_face_family_names_match_name_tables`, `default_fonts_total_size_is_reasonable`)

## 5) Refactor hazards (what can regress easily)

- Feature-matrix drift between declared profile metadata and actual bundled faces
  - Failure mode: profile names, provided roles, or expected family lists silently diverge from
    the bytes behind the manifest.
  - Existing gates: `bundled_profiles_are_manifest_consistent`,
    `bundled_face_family_names_match_name_tables`.
  - Missing gate to add: run a representative feature matrix in CI (`default`,
    `--no-default-features`, `--features bootstrap-full,emoji,cjk-lite`).
- Asset budget drift across feature combinations
  - Failure mode: WASM/bootstrap payload grows without an intentional contract update.
  - Existing gates: `default_fonts_total_size_is_reasonable`.
  - Missing gate to add: split out explicit budget checks for `bootstrap_fonts()` vs
    `default_fonts()` so bootstrap-only regressions cannot hide behind the larger default profile.
- Single-file maintenance pressure
  - Failure mode: future bundle additions touch unrelated profile, test, and asset constants in the
    same file, making review harder and ordering regressions easier.
  - Existing gates: manifest consistency tests catch some ordering issues.
  - Missing gate to add: none executable yet; the main fix is structural separation.
- Policy leakage into the asset crate
  - Failure mode: platform fallback defaults or startup policy get added here because the profile
    data is nearby.
  - Existing gates: dependency posture is currently clean.
  - Missing gate to add: keep `fret-fonts` in layering checks and review new deps aggressively.

## 6) Code quality findings (Rust best practices)

- The strongest part of the crate is its low dependency footprint and direct test coverage over the
  bundled manifest contract.
- The highest refactor risk is not algorithmic complexity but manual feature-matrix maintenance:
  dozens of `#[cfg(...)]` constants encode profile names, face sets, and provided-role lists.
- No `unsafe` usage, no UI-thread work, and no serialization hazards were found in the crate.

Evidence anchors:

- `crates/fret-fonts/src/lib.rs`
- `crates/fret-fonts/src/lib.rs` (`bundled_profile_matrix_covers_ui_and_monospace_contracts`,
  `bundled_profile_matrix_covers_emoji_fallback_contract`,
  `bundled_profile_matrix_covers_cjk_fallback_contract`)

## 7) Recommended refactor steps (small, gated)

1. Split `src/lib.rs` into `profile.rs`, `assets.rs`, and `tests.rs` (or generated manifest +
   helpers) — outcome: asset matrix changes become reviewable — gate: `cargo nextest run -p
   fret-fonts`.
2. Add representative feature-matrix gates for the crate — outcome: bundled profile drift is caught
   before integration — gate: `cargo check -p fret-fonts --no-default-features`, `cargo nextest
   run -p fret-fonts`, `cargo nextest run -p fret-fonts --features bootstrap-full,emoji,cjk-lite`.
3. Decide whether role-specific byte helper functions remain public — outcome: a narrower contract
   boundary that exposes profiles first and raw role slices only if justified — gate: docs-only in
   the first pass.

## 8) Open questions / decisions needed

- Should the bundled profile matrix stay handwritten Rust constants, or should it become a generated
  manifest derived from the subsetting scripts under `crates/fret-fonts/scripts/`?
- Do we want downstream crates to consume raw role-based byte helpers directly, or should
  `BundledFontProfile` become the only supported entrypoint?
