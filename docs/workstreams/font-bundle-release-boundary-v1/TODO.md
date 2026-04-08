# Font Bundle Release Boundary v1 — TODO

Status: Active

## Baseline evidence

- [x] FBRB-001 Record the current packaged-crate fact pattern:
  - `cargo package -p fret-fonts` produces a publish artifact that is too large for a comfortable
    first release.
  - `cargo package -p fret-fonts --list` includes all large bundled assets because they live in the
    crate, even when they are feature-gated at compile time.
- [x] FBRB-002 Confirm the current launch wiring:
  - `fret-launch` currently pulls `fret-fonts` default features, so the runtime baseline and crate
    defaults are coupled today.

## Contract decisions

- [x] FBRB-010 Decide the framework bootstrap baseline:
  - accepted verdict: `bootstrap-subset` only
  - evidence: `docs/workstreams/font-bundle-release-boundary-v1/BASELINE_DECISION_2026-04-08.md`
- [x] FBRB-011 Write the explicit "not baseline" verdict for:
  - `cjk-lite`
  - `emoji`
  - `bootstrap-full`
  - evidence: `docs/workstreams/font-bundle-release-boundary-v1/BASELINE_DECISION_2026-04-08.md`
- [x] FBRB-012 Decide whether `cjk-lite` stays in the core publish lane or moves to a first-party
  extension package.
  - accepted verdict: move `cjk-lite` out of the framework core publish lane
  - evidence: `docs/workstreams/font-bundle-release-boundary-v1/BASELINE_DECISION_2026-04-08.md`

## Package boundary changes

- [x] FBRB-020 Restructure the published package boundary so the main `fret-fonts` tarball no
  longer contains non-baseline assets:
  - `cjk-lite`
  - `emoji`
  - `bootstrap-full`
- [x] FBRB-021 Preserve the bundled profile / asset identity contract, or leave a narrow migration
  note if package names or profile names must change.
- [x] FBRB-022 Add a publish-facing package preflight gate so tarball regressions are caught before
  crates.io upload attempts.

## Consumer migration

- [x] FBRB-030 Make `fret-launch` install the chosen baseline explicitly instead of inheriting it
  indirectly from crate defaults.
- [x] FBRB-031 Move first-party web/demo/gallery surfaces to explicit opt-ins for any non-baseline
  bundles they still require, including `cjk-lite`.
- [x] FBRB-032 Refresh README and release docs so the bundled-font story matches the shipped package
  split.

## Release closure

- [x] FBRB-040 Re-run `cargo package -p fret-fonts` and confirm the main tarball fits the intended
  publication boundary.
- [x] FBRB-041 Re-run release closure / publish-order checks after the package split.
- [ ] FBRB-042 Resume crate publication only after `fret-fonts` no longer blocks downstream publish
  waves.
- [x] FBRB-043 Refresh release wave dry-run artifacts against the 51-crate publish graph before the
  next real publish attempt.

Notes on completion:

- FBRB-020 / FBRB-040 are proven by `python3 tools/check_fret_fonts_package_boundary.py` and
  `cargo package --allow-dirty --locked --list -p fret-fonts{,-cjk,-emoji}`.
- FBRB-021 is satisfied by preserving the bundled face/profile contract in `fret-fonts` baseline
  and moving extension asset identities into `fret-fonts-cjk` / `fret-fonts-emoji` with explicit
  bundle names.
- FBRB-022 is satisfied by the new `tools/check_fret_fonts_package_boundary.py` gate, which now
  runs inside `tools/check_fret_fonts_feature_matrix.py`.
- FBRB-041 is satisfied by regenerating `docs/release/v0.1.0-publish-order.txt` and
  `docs/release/v0.1.0-publish-waves.txt` from the updated `release-plz.toml` closure.
- FBRB-043 is satisfied by regenerating `docs/release/v0.1.0-wave-1-dry-run.txt` through
  `docs/release/v0.1.0-wave-12-dry-run.txt` with the new 51-crate wave graph. Wave 1 and Wave 2
  now pass locally; Wave 3 through Wave 12 are packaging-clean and fail only on expected
  crates.io visibility gaps.
- FBRB-042 is partially complete as of 2026-04-08:
  - Wave 2 has been published successfully (`fret-a11y-accesskit`, `fret-diag-protocol`,
    `fret-dnd`, `fret-fonts`, `fret-i18n-fluent`, `fret-perf`, `fret-platform`,
    `fret-ui-headless`, `fret-viewport-tooling`; `delinea` was already visible).
  - Wave 3 actual publish has started, but crates.io returned a new-crate rate-limit window while
    attempting `fret-diag-ws`. Resume after the server-provided retry time.
