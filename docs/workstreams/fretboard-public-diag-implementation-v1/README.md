# Fretboard Public Diag Implementation v1

Status: Active
Last updated: 2026-04-09

Goal: implement the first shipped public `fretboard diag` surface as a project-facing diagnostics
core, starting from the target contract frozen in the closed taxonomy lane.

This lane is a narrow follow-on to:

- `docs/workstreams/fretboard-public-app-author-surface-v1/README.md`

It does not reopen the product split. The prior lane already froze:

- public diagnostics remain part of the installed `fretboard` product,
- repo-only `fretboard-dev diag` keeps suite/campaign/registry and other mono-repo inventory
  helpers,
- and hotpatch/theme-import remain separate follow-ons.

This lane owns implementation only:

- create a public-facing diagnostics core around the frozen verb subset,
- remove repo-product hardcoding from the diagnostics CLI surface,
- decide the smallest publishable dependency posture for that core,
- and update docs/ADR wording once the public surface is real.

## Assumptions-first snapshot

1. `Confident`: public `fretboard diag` must stay project-facing, not repo-facing.
   - Evidence:
     - `docs/workstreams/fretboard-public-app-author-surface-v1/DIAG_TARGET_INTERFACE_STATE.md`
     - `docs/workstreams/fretboard-public-app-author-surface-v1/FINAL_STATUS.md`
     - `apps/fretboard/src/diag.rs`
   - Consequence if wrong:
     - we would re-export suite/campaign/registry taxonomy that only makes sense inside this
       mono-repo.

2. `Confident`: the first blocker is not command implementation depth, but CLI contract/branding
   hardcoding inside `fret-diag`.
   - Evidence:
     - `crates/fret-diag/src/cli/contracts/mod.rs`
     - `crates/fret-diag/src/cli/contracts/commands/query.rs`
     - `crates/fret-diag/src/cli/contracts/commands/script.rs`
   - Consequence if wrong:
     - we may over-invest in help text while a deeper runtime dependency issue is actually the real
       critical path.

3. `Likely`: the public diagnostics core can reuse substantial `fret-diag` implementation, but it
   needs a mode/allowlist seam before `crates/fretboard` can call it safely.
   - Evidence:
     - `crates/fret-diag/src/lib.rs`
     - `crates/fret-diag/src/cli/cutover.rs`
     - `crates/fretboard/src/cli/contracts.rs`
   - Consequence if wrong:
     - we may need a harder split such as a dedicated public wrapper crate or a deeper command
       reorganization.

4. `Likely`: publishing `fret-diag` is an implementation prerequisite, not a second user-facing CLI
   contract.
   - Evidence:
     - `docs/workstreams/fretboard-public-app-author-surface-v1/DIAG_TARGET_INTERFACE_STATE.md`
     - `crates/fret-diag/Cargo.toml`
     - `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
   - Consequence if wrong:
     - dependency closure work may be mis-scoped and the public surface could drift into a new
       product name by accident.

## Current target

- Establish a mode-aware diagnostics CLI surface in `fret-diag` so help/usage/examples are no
  longer hardcoded to `fretboard-dev diag`.
- Freeze the exact first shipped public verb set around the taxonomy lane:
  - `run`
  - `perf`
  - `latest`
  - `resolve`
  - `meta`
  - `pack`
  - `screenshots`
  - `windows`
  - `ai-packet`
  - `query`
  - `slice`
  - `stats`
  - `compare`
- Keep public `diag` free of:
  - `suite`
  - `campaign`
  - `registry`
  - repo script catalogs as the default teaching surface
  - repo dashboard/doctor/list inventory helpers

## Primary lane docs

- `docs/workstreams/fretboard-public-diag-implementation-v1/README.md`
- `docs/workstreams/fretboard-public-diag-implementation-v1/DESIGN.md`
- `docs/workstreams/fretboard-public-diag-implementation-v1/TODO.md`
- `docs/workstreams/fretboard-public-diag-implementation-v1/MILESTONES.md`
- `docs/workstreams/fretboard-public-diag-implementation-v1/EVIDENCE_AND_GATES.md`
