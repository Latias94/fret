# Fretboard Public App-Author Surface v1 — TODO

Status: In progress
Last updated: 2026-04-09

Tracking legend:

- `[ ]` open
- `[~]` in progress
- `[x]` done
- `[>]` moved elsewhere
- `[!]` blocked

ID format:

- `FBPUB-{area}-{nnn}`

## M0 — Freeze the product taxonomy

- [x] FBPUB-docs-010 Create a dedicated lane for the public app-author CLI surface.
- [x] FBPUB-docs-020 Record the current split between public `fretboard` and repo-only
      `fretboard-dev`.
- [x] FBPUB-docs-030 Align the obvious metadata/root-doc drift so public `fretboard` now clearly
      includes `new`.
  - Evidence:
    - `crates/fretboard/Cargo.toml`
    - `docs/README.md`
    - `docs/setup.md`

## M1 — Fix external-onboarding doc drift

- [ ] FBPUB-docs-100 Audit first-contact docs that still teach `fretboard-dev new` where the
      installed public CLI should be the product spelling.
  - Primary targets:
    - `docs/first-hour.md`
    - `docs/examples/README.md`
    - `docs/examples/todo-app-golden-path.md`

- [ ] FBPUB-docs-110 Establish a consistent docs rule for command spelling:
  - installed/external product wording: `fretboard ...`
  - in-repo maintainer wording: `cargo run -p fretboard-dev -- ...`

## M2 — Define the public `dev` contract

- [x] FBPUB-dev-200 Write the target public `dev native` / `dev web` contract around project-local
      inputs (`--manifest-path`, package/bin/example selection) instead of repo demo IDs.
  - Evidence:
    - `docs/workstreams/fretboard-public-app-author-surface-v1/TARGET_INTERFACE_STATE.md`
    - `apps/fretboard/src/dev/contracts.rs`
    - `apps/fretboard/src/dev/native.rs`
    - `apps/fretboard/src/dev/web.rs`

- [~] FBPUB-dev-210 Separate repo convenience selection (`--demo`, cookbook/demo registries, gallery
      shortcuts) from the future public `dev` contract.

- [ ] FBPUB-dev-220 Decide the smallest publishable dependency posture for the public `dev` lane.

## M3 — Define a public diagnostics core

- [ ] FBPUB-diag-300 Partition the current `diag` tree into:
  - public app-author diagnostics core
  - repo maintainer extensions

- [ ] FBPUB-diag-310 Decide whether the public diagnostics core stays in `fretboard` or requires a
      separately published `fret-diag` crate first.

- [ ] FBPUB-diag-320 List the exact first-wave public diagnostics verbs we are willing to teach.

## M4 — Resolve deferred surfaces

- [ ] FBPUB-hotpatch-400 Decide whether public hotpatch exists only as `fretboard dev ... --hotpatch`
      or stays entirely repo-only for v1.

- [ ] FBPUB-theme-410 Decide whether `theme import-vscode` belongs in the public CLI, remains on
      `fretboard-dev`, or moves to a future dedicated package.

## M5 — Close the policy loop

- [ ] FBPUB-adr-500 Refresh ADR/docs that currently mix future intent with current shipped CLI
      behavior.
  - Primary targets:
    - `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
    - `docs/README.md`
    - `docs/crate-usage-guide.md`

- [ ] FBPUB-plan-510 Convert the agreed public/private split into one or more landable
      implementation work items.
