# Fretboard Public App-Author Surface v1 — Target Interface State

Status: target state for the future public `fretboard dev` surface
Last updated: 2026-04-09

This document freezes the intended **public** `dev` contract before implementation work starts.

It is intentionally narrower than the current repo-only `fretboard-dev dev` surface.

## Target reading rule

Public `fretboard dev` must be explainable in one sentence:

> Run the current app project by manifest path and Cargo target selection, without depending on the
> Fret mono-repo's demo registry, cookbook inventory, or workspace shell layout.

If a flag cannot be explained that way, it does not belong in the public v1 `dev` contract.

## Target public command model

### Native

Target shape:

```bash
fretboard dev native [--manifest-path <path>] [--package <pkg>] [--bin <name> | --example <name>] [--profile <profile>] [--watch|--no-watch] [--supervise|--no-supervise] [--dev-state-reset] [--watch-poll-ms <ms>] [-- <arg>...]
```

Target semantics:

- `--manifest-path`
  - points at the user project's `Cargo.toml`
  - defaults to `./Cargo.toml`
- `--package`
  - optional package selection inside the current workspace
- `--bin`
  - selects a Cargo binary target from the chosen manifest/workspace
- `--example`
  - selects a Cargo example target from the chosen manifest/workspace
- `--profile`
  - forwards to Cargo profile selection
- `--watch`, `--no-watch`
  - control the restart/watch loop for the selected app target
- `--supervise`, `--no-supervise`
  - control the restart supervisor
- `--dev-state-reset`
  - resets the dev-state file before launch
- `--watch-poll-ms`
  - tunes watch polling
- trailing `-- ...`
  - passes through to the launched app target

Selection rule:

- exactly zero or one of `--bin` / `--example` may be given,
- if neither is given and the manifest resolves to exactly one runnable default target, `fretboard`
  may run it,
- if selection is ambiguous, the command should fail with an explicit hint instead of falling back
  to an interactive chooser.

### Web

Target shape:

```bash
fretboard dev web [--manifest-path <path>] [--package <pkg>] [--bin <name>] [--port <port>] [--open|--no-open] [--devtools-ws-url <url>] [--devtools-token <token>]
```

Target semantics:

- `--manifest-path`
  - points at the user project's `Cargo.toml`
  - defaults to `./Cargo.toml`
- `--package`
  - optional package selection inside the current workspace
- `--bin`
  - selects the wasm/web target when the workspace has more than one runnable candidate
- `--port`
  - forwards the dev-server port choice
- `--open`, `--no-open`
  - control browser auto-open
- `--devtools-ws-url`, `--devtools-token`
  - remain advanced but project-facing overrides

Selection rule:

- no repo demo ids,
- no interactive chooser,
- no implicit dependency on `apps/fret-demo-web`.

## Public v1 exclusions

These stay out of public `fretboard dev` v1:

- `--demo`
- `--choose`
- `--all`
- repo cookbook feature auto-enable logic
- repo gallery/demo shortcuts
- top-level hotpatch orchestration flags:
  - `--hotpatch`
  - `--hotpatch-reload`
  - `--hotpatch-trigger-path`
  - `--hotpatch-poll-ms`
  - `--hotpatch-devserver`
  - `--hotpatch-dx`
  - `--hotpatch-dx-ws`
  - `--hotpatch-build-id`

Reason:

- these either depend on repo-owned inventories, or they encode an advanced dev loop that should
  not freeze before the base public run contract is stable.

## Repo-only retained surface

The repo may keep a richer `fretboard-dev dev` surface for maintainers:

- `--demo`
- `--choose`
- `--all`
- cookbook-specific `--example` shortcuts and feature hints
- repo web demo ids
- hotpatch-oriented flows

Those are still useful, but they are not the public product contract.

## Migration mapping

| Current repo-only flag/behavior | Public target posture |
| --- | --- |
| `--demo <id>` | removed from public surface |
| `--choose` | removed from public surface |
| `--all` | removed from public surface |
| cookbook `--example <id>` with feature hints | keep only generic Cargo `--example <name>` semantics |
| web `--demo <id>` | removed from public surface |
| workspace root auto-discovery | replace with manifest-path/current-project resolution |
| `apps/fret-demo` / `apps/fret-demo-web` launchers | repo-only convenience only |
| hotpatch flags | deferred until a later public `dev` follow-on |

## Target user examples

Native app in current directory:

```bash
fretboard dev native
```

Native app with explicit bin:

```bash
fretboard dev native --manifest-path ./Cargo.toml --bin my-app
```

Native example target:

```bash
fretboard dev native --manifest-path ./Cargo.toml --example simple_todo
```

Web target:

```bash
fretboard dev web --manifest-path ./Cargo.toml --bin my-app-web --port 8080
```

## Explicit non-targets

This target-state document does **not** decide:

- the exact implementation crate split for public `dev`,
- whether Trunk remains the long-term web runner,
- the hotpatch follow-on posture beyond public `dev` v1 exclusions
  (see `docs/workstreams/fretboard-public-app-author-surface-v1/HOTPATCH_TARGET_INTERFACE_STATE.md`),
- or whether public diagnostics should live under `fretboard` directly or a separately published
  diagnostics package.

## Done-state summary

The done state is not “public `fretboard` now exposes every maintainer convenience”.

It is:

- public `dev` is project-facing rather than repo-facing,
- docs can teach it without naming this mono-repo's demos,
- and repo convenience remains available on `fretboard-dev` without contaminating the public
  contract.
