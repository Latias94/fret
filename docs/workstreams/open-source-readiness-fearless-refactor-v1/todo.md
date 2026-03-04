# Open Source Readiness (Fearless Refactor v1) — TODO

Status legend:

- `[ ]` Not started
- `[~]` In progress
- `[x]` Done
- `[?]` Needs triage / unclear ownership

## M0 — Lock the public story (docs-first)

- [x] Ensure `README.md` links only canonical entry points (no scattered run commands).
- [x] Add a short pointer from `docs/README.md` to this workstream.
- [x] Add a short pointer from `docs/examples/README.md` to the cookbook “recommended order”.

## M1 — Cookbook curation (small, teachable, fast)

- [x] Keep cookbook deps minimal (avoid enabling “everything” by default).
- [x] Add “Official vs Lab” labels for cookbook examples.
- [x] Gate the highest-ceiling interop examples behind explicit Cargo features (optional).
- [x] Add/curate diag scripts for the recommended 5–8 examples (start with `hello` + `simple_todo`).
  - Evidence: suites exist for the onboarding ladder:
    - [`tools/diag-scripts/suites/cookbook-hello/suite.json`](../../../tools/diag-scripts/suites/cookbook-hello/suite.json)
    - [`tools/diag-scripts/suites/cookbook-simple-todo/suite.json`](../../../tools/diag-scripts/suites/cookbook-simple-todo/suite.json)
    - [`tools/diag-scripts/suites/cookbook-overlay-basics/suite.json`](../../../tools/diag-scripts/suites/cookbook-overlay-basics/suite.json)
    - [`tools/diag-scripts/suites/cookbook-text-input-basics/suite.json`](../../../tools/diag-scripts/suites/cookbook-text-input-basics/suite.json)
    - [`tools/diag-scripts/suites/cookbook-commands-keymap-basics/suite.json`](../../../tools/diag-scripts/suites/cookbook-commands-keymap-basics/suite.json)
    - [`tools/diag-scripts/suites/cookbook-virtual-list-basics/suite.json`](../../../tools/diag-scripts/suites/cookbook-virtual-list-basics/suite.json)
    - [`tools/diag-scripts/suites/cookbook-effects-layer-basics/suite.json`](../../../tools/diag-scripts/suites/cookbook-effects-layer-basics/suite.json)
    - [`tools/diag-scripts/suites/cookbook-theme-switching-basics/suite.json`](../../../tools/diag-scripts/suites/cookbook-theme-switching-basics/suite.json)

## M2 — `fret` feature profiles

- [x] Make selector/query helpers optional (feature-gated) so `default-features = false` is actually small.
- [x] Make `diagnostics` opt-in by default (`app` excludes it; `batteries` includes it).
- [x] Document recommended profiles in `docs/crate-usage-guide.md`:
  - minimal app,
  - recommended app,
  - batteries-included.

## M3 — UI Gallery “lite” mode

- [x] Decide gating approach (runtime vs compile-time vs separate bin).
- [x] Implement lite mode (compile-time feature gating + optional deps) and update UI Gallery README.
- [x] Ensure lite mode has deterministic smoke gates (first frame + basic navigation).

## M4 — Move lesson-shaped demos out of `fret-demo`

- [x] Identify 8–15 lesson-shaped `apps/fret-demo/src/bin/*` candidates.
  - See: [docs/workstreams/open-source-readiness-fearless-refactor-v1/M4_CANDIDATES.md](./M4_CANDIDATES.md)
- [ ] Migrate into `apps/fret-cookbook/examples/*` with stable `test_id`s.
- [x] Keep `fret-demo` as maintainer/labs; ensure `fretboard list native-demos` stays intentionally small.
