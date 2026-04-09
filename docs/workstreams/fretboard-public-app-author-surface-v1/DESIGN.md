# Fretboard Public App-Author Surface v1

## Why this lane exists

Fret now has a mechanical CLI split:

- `crates/fretboard` is publishable and installed as `fretboard`
- `apps/fretboard` is repo-only and installed in-tree as `fretboard-dev`

That solved the packaging problem, but not the product-surface problem.

The remaining question is not “can this compile or publish?” but:

> Which CLI workflows should an external Fret app author actually learn and expect to remain
> stable across releases?

This lane defines that answer from the app-author workflow backward.

## Problem

The current CLI tree contains three different ownership classes:

1. **Project-agnostic public workflows**
   - starter scaffolds
   - app-owned asset helpers
   - project-local config helpers
2. **Potentially public app-author workflows whose implementation is still repo-shaped**
   - native/web run loops
   - diagnostics capture/inspect/perf
3. **Repo-owned maintainer workflows**
   - demo and cookbook discovery
   - mono-repo harness launchers
   - campaign/script shortcuts tied to `tools/` and in-tree demos
   - workspace hotpatch trigger/log helpers

If these classes are not separated, we risk freezing the wrong surface:

- either we keep too much capability private and force users onto repo-specific commands,
- or we publish repo assumptions that we are not ready to support as a stable contract.

## Goals

- Define the first-wave public `fretboard` surface for external app authors.
- Identify which current `fretboard-dev` capabilities should eventually move to `fretboard`.
- Make the move criteria explicit: project-agnostic inputs, publishable dependency closure, and
  documentation we are willing to teach publicly.
- Create a narrow execution plan for public `dev` and public diagnostics follow-on work.

## Non-goals

- Reworking the typed `clap` shell shape again.
- Publishing `fretboard-dev`.
- Promoting every current `diag` or `hotpatch` subcommand to a public contract.
- Rewriting repo-only demo tooling to look public without removing its repo assumptions.

## App-author workflow model

An external Fret app author typically needs five capabilities:

1. **Start** a new app quickly.
2. **Regenerate** app-owned assets and config files.
3. **Run** the app on native/web with one obvious command.
4. **Debug** UI behavior with bounded diagnostics artifacts.
5. **Accelerate** iteration with optional hot reload / hotpatch when the base run loop is already
   stable.

This yields a public CLI taxonomy:

### Tier 1: Public now

These are already valid for any external app repo and belong on `fretboard` immediately:

- `new`
- `assets`
- `config`

Contract rule:

- inputs are local project paths and names,
- outputs are project-owned files,
- implementation may not depend on this mono-repo's `apps/`, `tools/`, or workspace root layout.

### Tier 2: Public target, but not yet productized

These should become public once they are redesigned around project-local inputs:

- `dev native`
- `dev web`
- a reduced `diag` core:
  - capture / latest / inspect / pack / screenshots / perf / bounded query-style helpers

Contract rule:

- commands must target the user app by manifest path, package/bin/example selection, or explicit
  launch command,
- commands must work without in-tree demo registries or repo catalog lookups,
- public docs must teach them as first-class product workflows.

### Tier 3: Repo-only until proven otherwise

These remain on `fretboard-dev`:

- `list`
- repo demo launchers and cookbook discovery
- repo script catalogs / campaign presets / demo-specific shortcuts
- top-level `hotpatch` trigger/status/watch helpers

These are still useful, but they are useful **because** this repo has a mono-repo harness and
shared `.fret/` conventions. That is not enough to make them a public framework contract.

## Public-contract criteria

A command belongs on `fretboard` only if all of the following are true:

1. It can run in an external app repo.
2. Its direct dependency closure is publishable.
3. Its input vocabulary is app-owned rather than repo-owned.
4. We are willing to document it in first-hour / installed-CLI guidance.
5. We are willing to support its behavior changes as part of semver-facing release notes.

If any of the above fail, the command stays on `fretboard-dev`.

## Concrete command decisions

### `new`

Status: public now.

Reason:

- external users need it on day one,
- it already has a public-vs-repo mode split in the scaffold implementation,
- and the public path emits versioned dependencies rather than workspace path dependencies.

### `assets`

Status: public now.

Reason:

- this is a project-owned file generation workflow,
- it is already shared between the public and repo-only binaries.

### `config`

Status: public now.

Reason:

- this is a project-local config bootstrap helper,
- it is already shared between the public and repo-only binaries.

### `dev`

Status: should become public, but must be redesigned first.

Current blockers:

- current help/docs and implementation are tied to repo demos and cookbook examples,
- current native/web selection logic assumes this repo's app inventory,
- current command names teach repo targets rather than user projects.

Public target shape:

- `fretboard dev native --manifest-path <path>`
- `fretboard dev native --manifest-path <path> --bin <name>`
- `fretboard dev native --manifest-path <path> --example <name>`
- `fretboard dev web --manifest-path <path>`

The repo may still keep convenience wrappers on `fretboard-dev` for `--demo`, gallery presets, and
other in-tree shortcuts.

### `diag`

Status: public target is now defined at the command-family level, but not as a 1:1 copy of the
current tree.

Current blockers:

- `fret-diag` is not published today,
- first-party examples and docs still assume repo-local script catalogs,
- many subcommands are maintainer-oriented rather than app-author-oriented.

Public target rule:

- external users should learn `fretboard diag ...` as part of the same installed CLI product,
- if implementation requires publishing `fret-diag` first, treat that as dependency closure work
  rather than a second CLI product,
- the public core accepts user-supplied launch commands, script paths, and bundle paths,
- and the baseline story may not depend on `tools/diag-scripts/*`, promoted script ids, or repo
  campaign presets.

Frozen public-core target:

- capture/perf:
  - `run`
  - `perf`
- artifact/share helpers:
  - `latest`
  - `resolve`
  - `meta`
  - `pack`
  - `screenshots`
  - `windows`
  - `ai-packet`
- bounded inspection:
  - `query`
  - `slice`
  - `stats`
  - `compare`

Repo-only retained:

- `suite`
- `campaign`
- `registry`
- `list`
- repo dashboards, promoted script catalogs, and other mono-repo diagnostics inventory helpers

Deferred:

- `inspect`
- `script normalize|upgrade|validate|lint|shrink`
- `repro`

Authoritative target-state:

- `docs/workstreams/fretboard-public-app-author-surface-v1/DIAG_TARGET_INTERFACE_STATE.md`

### `hotpatch`

Status: public posture is now decided; keep repo-only for v1 and only consider a later
`dev native --hotpatch` follow-on.

Reason:

- the public value, if any, is as an optional mode of `dev native`,
- the top-level `hotpatch` command exposes transport/debug mechanics rather than the smallest
  stable app-author promise,
- and the only reasonable future public contract is capability-oriented ("accelerate the dev loop")
  with safe fallback, not transport-oriented (`dx`, touch files, or explicit devserver plumbing).

Frozen posture:

- public `fretboard` v1 does not expose hotpatch,
- top-level `fretboard hotpatch ...` remains repo-only,
- future public follow-on, if any:
  - `fretboard dev native --hotpatch`
- all current advanced hotpatch flags remain repo-only:
  - `--hotpatch-reload`
  - `--hotpatch-trigger-path`
  - `--hotpatch-poll-ms`
  - `--hotpatch-devserver`
  - `--hotpatch-dx`
  - `--hotpatch-dx-ws`
  - `--hotpatch-build-id`

Authoritative target-state:

- `docs/workstreams/fretboard-public-app-author-surface-v1/HOTPATCH_TARGET_INTERFACE_STATE.md`

### `list`

Status: repo-only.

Reason:

- it is an index over this mono-repo's demos/examples, not a framework user capability.

### `theme import-vscode`

Status: posture decided; keep it off public `fretboard` and treat any future public utility as a
dedicated package.

Reason:

- it is plausibly useful to external users,
- but it is not part of the central app-author lifecycle,
- the repo already has a domain-specific library boundary in `fret-vscode-theme`,
- and widening the main public CLI for a niche conversion workflow is the wrong product tradeoff.

Frozen posture:

- do not add `theme import-vscode` to public `fretboard` v1,
- keep the current command on `fretboard-dev` for now,
- future public direction, if warranted:
  - dedicated package built on `fret-vscode-theme`

Authoritative target-state:

- `docs/workstreams/fretboard-public-app-author-surface-v1/THEME_TARGET_INTERFACE_STATE.md`

## Dioxus comparison

Dioxus is the useful reference point here.

Its public CLI is one installed product (`dioxus-cli`, command `dx`) because the workflows it
exposes are already project-facing:

- create
- serve
- build
- bundle
- run
- doctor
- config

Fret should copy the **product principle**, not the exact command tree:

- public commands should center on user projects,
- maintainer-only harnesses should stay internal,
- and optional hotpatch/hot-reload flows should hang off the public run loop only after that run
  loop is stable.

## Expected outcomes

When this lane is complete:

- external onboarding docs can clearly distinguish installed `fretboard` usage from repo-local
  `cargo run -p fretboard-dev -- ...` usage,
- ADR 0109 and related docs stop mixing future intent with current shipped behavior,
- and follow-on implementation work can move command families across the public/private boundary
  without re-litigating the product taxonomy each time.

## Evidence anchors

- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/scaffold/mod.rs`
- `apps/fretboard/src/cli/contracts.rs`
- `apps/fretboard/src/dev/native.rs`
- `apps/fretboard/src/dev/web.rs`
- `apps/fretboard/src/demos.rs`
- `apps/fretboard/src/diag.rs`
- `apps/fretboard/src/hotpatch/contracts.rs`
- `apps/fretboard/src/theme.rs`
- `docs/README.md`
- `docs/first-hour.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
