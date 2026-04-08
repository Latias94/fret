# Font Bundle Release Boundary v1 — Evidence and Gates

Status: Active

## Smallest current repro

Use this sequence before changing code:

```bash
python3 tools/check_fret_fonts_package_boundary.py
python3 tools/check_fret_fonts_feature_matrix.py
cargo nextest run -p fret-render-wgpu --lib
python3 tools/release_closure_check.py --config release-plz.toml --print-publish-commands
```

What this proves:

- the main `fret-fonts` package no longer ships CJK/emoji/full-font assets,
- the extension crates each own their explicit publish boundary,
- bundled-only renderer fallback still works once extensions are seeded through explicit
  `common_fallback` configuration,
- and the release wave remains structurally publishable once the package split lands.

## Current evidence at lane open

- Local packaging preflight on 2026-04-08 produced a `fret-fonts` publish artifact around `15 MiB`
  compressed.
- The packaged crate list includes:
  - `NotoColorEmoji.ttf`
  - `NotoSansCJKsc-Regular-cjk-lite-subset.otf`
  - full Inter assets
  - full JetBrains Mono assets
- `fret-launch` currently resolves `fret-fonts` default features, so `bootstrap-subset + cjk-lite`
  are effectively part of the launch baseline today.
- Early crate publication already proved that new-crate rate limiting exists, so package retries are
  expensive; `fret-fonts` must stop failing at the publication boundary.
- ADR 0147 and first-party web harnesses already model `cjk-lite` / `emoji` as optional tiers, so
  the package split should align the publication boundary with that existing conceptual model.

## Accepted M1 decision

Accepted on 2026-04-08:

- framework baseline: `bootstrap-subset`
- non-baseline bundles: `cjk-lite`, `emoji`, `bootstrap-full`

Decision note:

- `docs/workstreams/font-bundle-release-boundary-v1/BASELINE_DECISION_2026-04-08.md`

## Current evidence after M2 implementation

Validated on 2026-04-08:

- `cargo nextest run -p fret-render-wgpu --lib` is green after updating bundled-only fallback tests
  to mirror the real web startup contract.
- `python3 tools/check_fret_fonts_package_boundary.py` proves:
  - `fret-fonts` ships only baseline subset assets,
  - `fret-fonts-cjk` owns the CJK subset and its license/materialization text,
  - `fret-fonts-emoji` owns the emoji bundle and its license.
- `python3 tools/check_fret_fonts_feature_matrix.py` now includes the package-boundary gate in
  addition to baseline / extension test coverage.
- `python3 tools/release_closure_check.py --config release-plz.toml --write-order ... --write-waves ...`
  regenerated release artifacts for a 51-crate graph with:
  - `fret-fonts-cjk`
  - `fret-fonts-emoji`
  inserted before downstream crates that depend on `fret-runtime` / text surfaces.
- `python3 tools/release_wave_dry_run.py --wave <N> --allow-registry-gap` has been rerun for all 12
  waves using `cargo publish --dry-run --allow-dirty`:
  - wave 1 and wave 2 pass locally,
  - wave 3 through wave 12 fail only because earlier waves are not yet visible on crates.io.
- `python3 tools/release_wave_registry_status.py --wave 2` confirms Wave 2 is now fully visible on
  crates.io after publish.
- Real publish progress on 2026-04-08:
  - Wave 2 crates were published successfully.
  - Wave 3 publish began at `fret-diag-ws` but was deferred by crates.io's new-crate rate limit.

## Gate set

### Core crate gates

```bash
cargo nextest run -p fret-fonts
python3 tools/check_fret_fonts_feature_matrix.py
```

### Package boundary gates

```bash
python3 tools/check_fret_fonts_package_boundary.py
cargo package --allow-dirty --locked --list -p fret-fonts
cargo package --allow-dirty --locked --list -p fret-fonts-cjk
cargo package --allow-dirty --locked --list -p fret-fonts-emoji
```

### Consumer / renderer gates

```bash
cargo nextest run -p fret-launch --features wasm-cjk-lite-fonts,wasm-emoji-fonts
cargo nextest run -p fret-render-wgpu --lib
```

### Release closure gate

```bash
python3 tools/release_closure_check.py --config release-plz.toml --write-order docs/release/v0.1.0-publish-order.txt --write-waves docs/release/v0.1.0-publish-waves.txt --print-publish-commands --print-publish-waves
python3 tools/release_wave_dry_run.py --wave 1 --allow-registry-gap
python3 tools/release_wave_dry_run.py --wave 2 --allow-registry-gap
python3 tools/release_wave_dry_run.py --wave 3 --allow-registry-gap
```

## Evidence anchors

- `crates/fret-fonts/Cargo.toml`
- `crates/fret-fonts/src/assets.rs`
- `crates/fret-fonts/src/profiles.rs`
- `crates/fret-fonts/src/tests.rs`
- `crates/fret-fonts-cjk/Cargo.toml`
- `crates/fret-fonts-cjk/src/lib.rs`
- `crates/fret-fonts-emoji/Cargo.toml`
- `crates/fret-fonts-emoji/src/lib.rs`
- `crates/fret-launch/Cargo.toml`
- `crates/fret-launch/src/runner/font_catalog.rs`
- `crates/fret-render-wgpu/src/text/tests.rs`
- `tools/check_fret_fonts_feature_matrix.py`
- `tools/check_fret_fonts_package_boundary.py`
- `tools/release_wave_dry_run.py`
- `tools/release_wave_registry_status.py`
- `docs/workstreams/font-mainline-fearless-refactor-v1/README.md`
- `docs/workstreams/font-bundle-release-boundary-v1/BASELINE_DECISION_2026-04-08.md`
- `docs/release/v0.1.0-publish-order.txt`
- `docs/release/v0.1.0-publish-waves.txt`
- `docs/release/v0.1.0-wave-1-dry-run.txt`
- `docs/release/v0.1.0-wave-2-dry-run.txt`
- `docs/release/v0.1.0-wave-3-dry-run.txt`
- `docs/release/v0.1.0-release-checklist.md`
