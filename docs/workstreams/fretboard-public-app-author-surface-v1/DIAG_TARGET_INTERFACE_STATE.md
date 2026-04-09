# Fretboard Public App-Author Surface v1 — Diagnostics Target Interface State

Status: target state for the future public `fretboard diag` surface
Last updated: 2026-04-09

This document freezes the intended **public** diagnostics contract before implementation work
starts.

It is intentionally narrower than the current repo-only `fretboard-dev diag` tree.

## Target reading rule

Public `fretboard diag` must be explainable in one sentence:

> Capture, inspect, compare, and share diagnostics for the current app project using explicit
> script paths, launch commands, or bundle paths, without depending on the Fret mono-repo's
> promoted script registry, suite catalog, or campaign taxonomy.

If a command cannot be explained that way, it does not belong in public `diag` v1.

## Product rule

Public diagnostics remain part of the installed `fretboard` product:

- external docs should teach `fretboard diag ...`,
- external users should not need a second CLI product name,
- and if implementation requires publishing `fret-diag` as a reusable dependency first, that is a
  packaging prerequisite rather than a second end-user contract.

This mirrors the same product rule as public `dev`: one project-facing CLI, with internal crate
splits treated as implementation detail.

## Target public command families

These are the first-wave diagnostics verbs we are willing to teach publicly once the dependency
closure is publishable.

### Capture and perf

Target verbs:

- `run`
- `perf`

Target input rule:

- inputs are explicit script paths, explicit bundle paths, or explicit `--launch -- <cmd...>`
  commands,
- no promoted `script_id` indirection is required for the baseline story,
- no repo suite names or campaign lanes are required for the baseline story.

Representative target shapes:

```bash
fretboard diag run ./diag/dialog-escape.json --launch -- cargo run --manifest-path ./Cargo.toml
fretboard diag perf ./diag/dialog-escape.json --repeat 5 --launch -- cargo run --manifest-path ./Cargo.toml
```

### Bundle resolution, metadata, and sharing

Target verbs:

- `latest`
- `resolve`
- `meta`
- `pack`
- `screenshots`
- `windows`
- `ai-packet`

Target input rule:

- commands operate on a concrete bundle directory, bundle artifact, or the session-aware "latest"
  resolution model,
- share/export flows remain bounded to app-owned artifacts,
- no repo dashboard/catalog indirection is required.

Representative target shapes:

```bash
fretboard diag latest
fretboard diag resolve latest --dir ./target/fret-diag
fretboard diag meta ./target/fret-diag/latest --json
fretboard diag pack ./target/fret-diag/latest --ai-only
```

### Bounded bundle inspection

Target verbs:

- `query`
- `slice`
- `stats`
- `compare`

Target input rule:

- commands operate on explicit bundle paths,
- selectors are bounded (`test-id`, `step-index`, `top`, diff pairs),
- semantics are stable enough to teach as artifact-inspection tools instead of repo maintainer
  internals.

Representative target shapes:

```bash
fretboard diag query test-id ./target/fret-diag/latest "dialog"
fretboard diag slice ./target/fret-diag/latest --test-id dialog-overlay
fretboard diag stats ./target/fret-diag/latest --sort time --top 20
fretboard diag compare ./target/fret-diag/baseline ./target/fret-diag/candidate --json
```

## Public v1 exclusions

These stay out of public `fretboard diag` v1:

- `suite`
- `campaign`
- `registry`
- `list`
- repo-promoted `script_id` lookup and `tools/diag-scripts/*` catalogs as the default teaching
  surface
- repo dashboard/index/doctor helpers that summarize the mono-repo's shared diagnostics inventory
- repo workflow helpers tied to triage/pick/repeat/session-maintenance rather than direct app-author
  capture and inspection

Reason:

- these commands are useful because this repository has a shared diagnostics corpus and maintainer
  workflow,
- not because every external app author needs them as a stable framework contract.

## Deferred candidates

These may become public later, but they should not block public `diag` v1:

- `inspect`
  - likely valuable, but it depends on the broader public devtools/inspector posture and should
    ship only when that story is productized end to end
- `script normalize|upgrade|validate|lint|shrink`
  - useful for authors who invest in scripted diagnostics, but still closer to power-user script
    authoring than first-wave capture/inspect/share
- `repro`
  - valuable for deeper perf/resource gates, but currently closer to automation/maintainer
    workflows than the minimal public story

## Repo-only retained surface

The repo may keep a richer `fretboard-dev diag` tree for maintainers:

- suite and campaign orchestration over repo-owned script catalogs
- promoted script registry maintenance
- diagnostics dashboards, triage helpers, and inventory indexes
- demo/gallery-oriented shortcuts and preset taxonomy
- higher-order maintenance helpers that derive value from the mono-repo's shared demo fleet

Those remain legitimate tooling, but they are not the public app-author contract.

## Migration mapping

| Current repo-oriented shape | Public target posture |
| --- | --- |
| `diag run <script_id>` via promoted registry | require an explicit script path |
| `diag suite <suite>` | repo-only |
| `diag perf ui-gallery` | keep only explicit script/bundle targets; no repo suite ids |
| `diag registry *` | repo-only |
| `diag campaign *` | repo-only |
| `diag script validate/lint/...` | deferred |
| `diag inspect on/off/...` | deferred until public inspector story is stable |

## Explicit non-targets

This target-state document does **not** decide:

- whether the implementation lives entirely inside `crates/fretboard` or is delegated to a
  published `fret-diag` library crate,
- the exact stabilization order inside the public diagnostics core,
- whether public inspector flows ship in the same milestone as public `dev`,
- or whether repo-only maintainer helpers should later move to a separate internal package.

## Done-state summary

The done state is not "make the current `fretboard-dev diag` tree public".

It is:

- public diagnostics are project-facing rather than repo-facing,
- external docs can teach one installed `fretboard diag` story,
- the first-wave verbs are explicit,
- and repo-only maintainer diagnostics remain available without contaminating the public contract.
