# UI + Launch Modularization v1 — TODO

Status: Draft

Checklist for Track B modularization refactor (no crate boundary changes).

## Baseline

- [ ] Record current “top large files” (for comparison) in:
  - `crates/fret-ui/src/tree/*`
  - `crates/fret-launch/src/runner/*`
- [ ] Confirm minimal gates runnable locally:
  - `python tools/check_layering.py`
  - `cargo fmt`
  - `cargo nextest run -p fret-ui`

## `fret-ui` (tree modularization)

- [ ] Create `tree/*` module skeleton (no logic changes yet).
- [ ] Move storage/state types into `tree/state/*` (mechanical moves).
- [ ] Move mount/diff/build logic into `tree/mount/*`.
- [ ] Move dispatch/routing code into `tree/dispatch/*`.
- [ ] Move layout/invalidation code into `tree/layout/*`.
- [ ] Keep `tree/mod.rs` thin: re-exports + a small number of entry points.
- [ ] Tighten visibility:
  - [ ] prefer `pub(crate)` where possible
  - [ ] keep `fret-ui` public exports deliberate
- [ ] Gates:
  - [ ] `cargo fmt`
  - [ ] `python tools/check_layering.py`
  - [ ] `cargo nextest run -p fret-ui`

## `fret-launch` (runner modularization)

- [ ] Create `runner/*` module skeleton and move non-hot-path utilities into `runner/common/*`.
- [ ] Split `runner/desktop/*` by responsibility:
  - [ ] window lifecycle + event loop
  - [ ] render/present + surface management
  - [ ] effects draining and platform IO wiring
  - [ ] diagnostics-only utilities (screenshots, bundles)
- [ ] Split `runner/web/*` similarly (RAF loop vs effects vs streaming assets).
- [ ] Gates:
  - [ ] `cargo fmt`
  - [ ] `python tools/check_layering.py`
  - [ ] `cargo nextest run -p fret-launch`

## Documentation alignment

- [ ] Verify ADR statements about `fret-ui` dependencies reflect reality (avoid misleading guidance).
- [ ] Record any “new contract needed” discoveries and decide: ADR update vs ecosystem implementation.

## Closeout

- [ ] Sanity check representative demos:
  - [ ] `apps/fret-ui-gallery`
  - [ ] `apps/fret-demo`
- [ ] Decide whether Track C is justified (compile time, dependency isolation, web/desktop divergence).

