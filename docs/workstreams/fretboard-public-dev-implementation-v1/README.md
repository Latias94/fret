# Fretboard Public Dev Implementation v1

Status: Closed
Last updated: 2026-04-09

Status note (2026-04-09): this lane is closed. The shipped public guidance now lives in
`README.md`, `docs/README.md`, `docs/crate-usage-guide.md`, and
`docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`. References below to the target
surface should be read as the execution record that led to the shipped `fretboard dev` command.

Goal: implement the first shipped public `fretboard dev` surface as a project-facing app run loop,
starting from the target contract frozen in the closed taxonomy lane.

This lane is a narrow follow-on to:

- `docs/workstreams/fretboard-public-app-author-surface-v1/README.md`

It does not reopen the product-taxonomy debate. The prior lane already froze:

- public `fretboard` owns project-facing `dev`,
- repo-only `fretboard-dev` keeps repo demo ids, hotpatch orchestration, and other maintainer
  shortcuts,
- public `diag` / theme-import remain separate follow-ons.

This lane owned implementation only:

- add a shipped public `fretboard dev` command,
- keep it project-shaped (`--manifest-path`, package/target selection),
- avoid repo demo registries, interactive choosers, and hotpatch-only flags,
- update public docs/ADR wording once the command is real.

## Assumptions-first snapshot

1. `Confident`: public `fretboard dev` must run against an arbitrary Cargo project, not this
   mono-repo layout.
   - Evidence:
     - `docs/workstreams/fretboard-public-app-author-surface-v1/TARGET_INTERFACE_STATE.md`
     - `crates/fretboard/src/cli/contracts.rs`
     - `apps/fretboard/src/dev/native.rs`
   - Consequence if wrong:
     - the public CLI would keep leaking repo-only concepts like demo ids and shell packages.

2. `Confident`: `cargo metadata` is the correct first-principles source for package/target
   selection.
   - Evidence:
     - `cargo metadata --format-version 1 --no-deps --manifest-path <path>`
     - `docs/workstreams/fretboard-public-app-author-surface-v1/TARGET_INTERFACE_STATE.md`
   - Consequence if wrong:
     - target resolution would drift from Cargo semantics and break external workspaces.

3. `Likely`: public `dev native` should land before or at least more robustly than public
   `dev web`.
   - Evidence:
     - `apps/fretboard/src/dev/native.rs`
     - `apps/fretboard/src/dev/web.rs`
     - `docs/README.md`
   - Consequence if wrong:
     - we may overfit this lane to native-only wording and delay a necessary web path.

4. `Likely`: repo-only `fretboard-dev dev` can remain richer than the public command even after the
   public implementation lands.
   - Evidence:
     - `apps/fretboard/src/dev/contracts.rs`
     - `docs/workstreams/fretboard-public-app-author-surface-v1/FINAL_STATUS.md`
   - Consequence if wrong:
     - we would force unstable maintainer workflows into the public package boundary too early.

## Closed target

- Implement public `fretboard dev native` with:
  - `--manifest-path`
  - `--package`
  - `--bin | --example`
  - `--profile`
  - `--watch|--no-watch`
  - `--supervise|--no-supervise`
  - `--dev-state-reset`
  - `--watch-poll-ms`
  - trailing `-- ...`
- Implement public `fretboard dev web` only if it can stay package-root/index.html based and does
  not require repo shell assumptions.
- Keep public `dev` free of:
  - `--demo`
  - `--choose`
  - `--all`
  - hotpatch flags
  - repo cookbook/demo feature hints

## Primary lane docs

- `docs/workstreams/fretboard-public-dev-implementation-v1/README.md`
- `docs/workstreams/fretboard-public-dev-implementation-v1/DESIGN.md`
- `docs/workstreams/fretboard-public-dev-implementation-v1/TODO.md`
- `docs/workstreams/fretboard-public-dev-implementation-v1/MILESTONES.md`
- `docs/workstreams/fretboard-public-dev-implementation-v1/EVIDENCE_AND_GATES.md`
