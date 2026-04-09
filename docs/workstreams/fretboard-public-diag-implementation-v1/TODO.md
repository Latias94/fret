# Fretboard Public Diag Implementation v1 TODO

Status: Active
Last updated: 2026-04-09

- [x] FDIAGPUB-100 Create the dedicated follow-on lane and record the implementation slice.
- [x] FDIAGPUB-200 Add a mode-aware diagnostics CLI contract seam in `fret-diag` so public help and
      repo help can diverge cleanly.
- [x] FDIAGPUB-210 Freeze and enforce the exact public-core verb allowlist.
- [x] FDIAGPUB-220 Wire the public-core diagnostics entrypoint into `crates/fretboard`.
- [x] FDIAGPUB-230 Audit `fret-diag` publication blockers and dependency closure.
  - `release-plz.toml` now includes `fret-diag`.
  - `python3 tools/release_closure_check.py --config release-plz.toml --print-publish-commands`
    reports 53 crates, 0 internal dependency issues, and places `fret-diag` before `fretboard`.
  - `cargo publish --dry-run --allow-dirty -p fret-diag` now succeeds.
- [ ] FDIAGPUB-240 Publish `fret-diag` before validating or publishing `fretboard`.
  - `cargo publish --dry-run --allow-dirty -p fretboard` still fails until crates.io can resolve the
    newly added `fret-diag` dependency from the index.
- [ ] FDIAGPUB-300 Update public help/docs/ADR wording for the shipped diagnostics surface.
- [x] FDIAGPUB-400 Run help/tests/smoke gates for the new public diagnostics core.
