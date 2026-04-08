# Font Bundle Release Boundary v1

Status: Active
Last updated: 2026-04-08

This workstream is a narrow follow-on to `font-mainline-fearless-refactor-v1`. It does not reopen
the whole font-owner-map refactor. It owns one smaller question: where the bundled-font boundary
should sit for a publishable crate line.

## Why this lane exists

The current `fret-fonts` crate has a sound runtime purpose but an unsound publication boundary:

- Fret still needs a bundled startup baseline because Web/WASM cannot rely on system font access.
- The framework baseline affects text measurement, fallback traces, diagnostics screenshots, and
  reproducible startup behavior.
- Cargo publishes the crate tarball, not "only the files selected by active features".
- Large optional assets (`emoji`, `bootstrap-full`) currently live in the same published package as
  the framework baseline.

That means the current package shape can be wrong even when the runtime feature model looks tidy.

## First-principles constraints

1. The framework default must guarantee a minimal portable text baseline.
2. Publication cost is owned by the package boundary, not by feature flags alone.
3. Optional language coverage and demo-grade convenience assets must not silently redefine the
   framework baseline.
4. Web/WASM must still have a bundled startup path after this split.

## Starting assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Cargo/crates.io publication cost is driven by the packaged crate contents, not by which features downstream consumers enable. | Confident | `cargo package -p fret-fonts`, `cargo package -p fret-fonts --list` | A feature-only split would be enough, and this lane would be narrower than expected. |
| `fret-launch` currently treats `fret-fonts` default features as part of the startup baseline. | Confident | `cargo tree -e features -p fret-launch`, `crates/fret-launch/Cargo.toml` | Launch wiring might already be explicit, reducing the migration slice. |
| `emoji` and `bootstrap-full` are not required for minimum framework correctness. | Confident | `crates/fret-fonts/Cargo.toml`, first-party web/demo feature flags, diagnostics notes | The lane would have to preserve them in the main package, which would materially change the target split. |
| `cjk-lite` is conceptually a first-party extension bundle rather than part of the framework baseline, even though the current launch defaults still pull it in. | Confident | `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`, `crates/fret-launch/Cargo.toml`, app web feature flags | M2 must split `cjk-lite` at the package boundary instead of preserving it in the main crate. |

## In scope

- Define which bundled faces are part of the framework-owned bootstrap baseline.
- Define which bundled faces are extension bundles and must move out of the main published package.
- Make launch/startup installation explicit instead of inheriting baseline meaning from crate
  defaults accidentally.
- Align first-party web/demo surfaces and release preflight with the new package boundary.

## Out of scope

- Rewriting renderer fallback policy or text shaping internals.
- Changing locale or platform fallback heuristics.
- Removing bundled fonts entirely.
- General font-catalog refresh or rescan design; those already belong to the main font lanes.

## Target shipped state

### Non-negotiable target

- `fret-fonts` remains the framework-owned bundled-font contract crate.
- The main published `fret-fonts` package only contains assets that are part of the real framework
  startup baseline.
- `emoji` and `bootstrap-full` no longer inflate that main published package.
- Release preflight explicitly checks package shape before publish waves resume.

### M1 baseline decision

Accepted on 2026-04-08:

- framework baseline: `bootstrap-subset`
- not framework baseline: `cjk-lite`, `emoji`, `bootstrap-full`

See:

- `docs/workstreams/font-bundle-release-boundary-v1/BASELINE_DECISION_2026-04-08.md`

This means the publication boundary must align with the conceptual model already present in ADR
0147 and in the first-party web feature surfaces: CJK coverage may still be enabled by default for
specific shipped apps, but it is an app/product decision, not a framework bootstrap invariant.

## Expected package split direction

- Core publish lane:
  - bootstrap sans/serif/mono baseline required for framework startup
- Extension publish lanes:
  - CJK coverage bundle
  - emoji coverage bundle
  - full-size debug/demo bundle

The implementation shape may be separate crates, or another equivalent package-level split, but the
publication boundary must become real at the crate/package level.

## Evidence anchors

- `crates/fret-fonts/Cargo.toml`
- `crates/fret-fonts/src/assets.rs`
- `crates/fret-fonts/src/profiles.rs`
- `crates/fret-launch/Cargo.toml`
- `crates/fret-launch/src/runner/font_catalog.rs`
- `docs/workstreams/font-mainline-fearless-refactor-v1/README.md`
- `docs/workstreams/font-bundle-release-boundary-v1/BASELINE_DECISION_2026-04-08.md`
