# Jellyflow Package Split v1 - TODO

## Guardrails

- [x] Do not split Jellyflow into a separate repository before crate boundaries are proven inside
      the Fret monorepo.
- [x] Keep `fret-node` as the Fret adapter and compatibility facade.
- [x] Keep `jellyflow-core` free of `fret-ui`, `fret-runtime`, `fret-canvas`, `wgpu`, and `winit`.
- [ ] Do not move runtime, geometry, or UI modules until the previous package boundary has focused
      compile and compatibility gates.

## Tasks

- [x] JF-001 Create the first headless Jellyflow core crate.
  - Scope:
    - create `ecosystem/jellyflow-core`
    - move `core`, `types`, and `interaction` from `fret-node` into `jellyflow-core`
    - add compatibility wrapper modules in `fret-node`
    - add a manifest source-policy test that keeps `jellyflow-core` off UI/render/platform deps
    - record ADR/workstream evidence for the new package direction
  - Validation:
    - `cargo check -p jellyflow-core`
    - `cargo nextest run -p jellyflow-core`
    - `cargo check -p fret-node --all-features --tests`
    - `cargo nextest run -p fret-node --no-default-features`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
  - Exit note: first compile proof lands the brand/package boundary without moving runtime or UI.
  - Fresh gates:
    - `cargo check -p jellyflow-core`: passed.
    - `cargo nextest run -p jellyflow-core`: passed with 14 tests.
    - `cargo clippy -p jellyflow-core --all-targets -- -D warnings`: passed.
    - `cargo check -p fret-node --all-features --tests`: passed.
    - `cargo nextest run -p fret-node --no-default-features`: passed with 124 tests.
    - `cargo fmt --check`: passed.
    - `jq empty docs/workstreams/jellyflow-package-split-v1/WORKSTREAM.json`: passed.
    - `git diff --check`: passed.
    - `python3 tools/check_layering.py`: passed.

- [x] JF-010 Decide whether `ops` moves to `jellyflow-core` or waits for `jellyflow-runtime`.
  - Scope:
    - dependency audit of `ops`, `runtime`, and public re-export compatibility
    - move `GraphOp`, `GraphTransaction`, `GraphHistory`, fragment/diff/normalize helpers, and
      transaction sanity checks into `jellyflow-core`
    - keep the XyFlow-style node/edge change projection helper in `fret-node`
  - Validation:
    - `cargo check -p fret-node --all-features --tests`
    - `cargo nextest run -p fret-node --no-default-features`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
  - Exit note: second compile proof lands the transaction/history boundary without moving runtime
    or UI.
  - Fresh gates:
    - `cargo check -p fret-node --all-features --tests`: passed.
    - `cargo nextest run -p fret-node --no-default-features`: passed with 90 tests.
    - `cargo fmt --check`: passed.
    - `python3 tools/check_layering.py`: passed.

- [ ] JF-020 Extract the first runtime boundary only after JF-010.
  - Scope: decide whether runtime store/callback helpers and any remaining geometry seams belong in
    `jellyflow-runtime` or stay in the Fret adapter.
  - Validation: `cargo nextest run -p fret-node --no-default-features runtime` plus new
    Jellyflow runtime gates.
