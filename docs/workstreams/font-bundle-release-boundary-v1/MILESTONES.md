# Font Bundle Release Boundary v1 — Milestones

Status: Active

## M0: Evidence freeze and scope lock

Exit criteria:

- The current packaged-crate facts are recorded:
  - tarball size
  - tarball file list
  - launch feature propagation
- The lane explicitly states:
  - what it owns,
  - what it does not own,
  - and why it exists as a narrow follow-on instead of reopening the whole font mainline lane.

Primary commands:

- `cargo package -p fret-fonts`
- `cargo package -p fret-fonts --list`
- `cargo tree -e features -p fret-launch | rg "fret-fonts|bootstrap-subset|cjk-lite|emoji"`

## M1: Baseline contract decision

Exit criteria:

- The framework-owned bootstrap baseline is explicit.
- `cjk-lite`, `emoji`, and `bootstrap-full` are explicitly classified as non-baseline assets.
- The lane records the `cjk-lite` verdict as a first-party extension bundle.

Primary evidence:

- `docs/workstreams/font-bundle-release-boundary-v1/DESIGN.md`
- `docs/workstreams/font-bundle-release-boundary-v1/TODO.md`
- `docs/workstreams/font-bundle-release-boundary-v1/BASELINE_DECISION_2026-04-08.md`

Status:

- Completed on 2026-04-08.

## M2: Package split and consumer migration

Exit criteria:

- The main published `fret-fonts` package only contains assets that belong to the agreed framework
  baseline.
- `fret-launch` installs the baseline explicitly.
- First-party web/demo surfaces opt into extra bundles explicitly, including `cjk-lite`.
- Tests and docs reflect the new package split.

Primary gates:

- `cargo nextest run -p fret-fonts`
- `python3 tools/check_fret_fonts_feature_matrix.py`
- `python3 tools/check_fret_fonts_package_boundary.py`
- `cargo nextest run -p fret-render-wgpu --lib`
- `cargo package --allow-dirty --locked --list -p fret-fonts`

Status:

- Completed on 2026-04-08.
- Landed outcomes:
  - `fret-fonts` is baseline-only at the published package boundary.
  - `fret-fonts-cjk` and `fret-fonts-emoji` carry extension assets explicitly.
  - `fret-launch` and first-party web/gallery wiring seed extension common-fallback families
    explicitly instead of inheriting them from `fret-fonts`.
  - renderer bundled-only tests now mirror the real web startup contract instead of relying on the
    historical implicit fallback set.

## M3: Release closure unblock

Exit criteria:

- `fret-fonts` package preflight is green for the new publication boundary.
- Release closure checks are re-run after the split.
- The crate publish wave can continue without `fret-fonts` being the blocker.

Primary gates:

- `python3 tools/check_fret_fonts_package_boundary.py`
- `python3 tools/release_closure_check.py --config release-plz.toml --print-publish-commands`

Status:

- In progress on 2026-04-08.
- Structural exit criteria are satisfied on 2026-04-08:
  - closure/order is green for a 51-crate release graph,
  - wave 1 and wave 2 pass local `cargo publish --dry-run --allow-dirty`,
  - wave 3 through wave 12 show registry-gap-only failures.
- Remaining work is operational rather than structural:
  - continue actual publish attempts when crates.io propagation/rate limits allow.
- Operational progress on 2026-04-08:
  - Wave 2 is fully visible on crates.io after successful publish.
  - Wave 3 is dry-run clean and ready, but the first real publish attempt hit crates.io new-crate
    throttling.
