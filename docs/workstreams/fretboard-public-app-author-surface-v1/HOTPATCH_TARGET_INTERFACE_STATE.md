# Fretboard Public App-Author Surface v1 — Hotpatch Target Interface State

Status: target posture for public hotpatch-facing workflows
Last updated: 2026-04-09

This document freezes the intended **public posture** for hotpatch-related workflows.

It does **not** make hotpatch part of the first-wave public `fretboard` contract.

## Target reading rule

If hotpatch ever becomes public, it must be explainable in one sentence:

> Accelerate the current native app's edit-observe loop as an optional mode of `fretboard dev
> native`, while reserving the exact delivery backend and fallback behavior to the tool.

If a flag or command instead exposes touch-file paths, devserver wiring, log-tail plumbing, or
mono-repo demo assumptions as the primary story, it does not belong in the public contract.

## M4 decision

The public posture is:

- public `fretboard` v1 does **not** expose hotpatch yet,
- top-level `fretboard hotpatch ...` does **not** become public,
- if hotpatch becomes public later, it does so only as an optional submode of
  `fretboard dev native --hotpatch`.

This keeps hotpatch subordinate to the project-facing run loop instead of turning transport details
into a top-level product surface.

## Future public follow-on shape

The only future public entry point we are willing to target is:

```bash
fretboard dev native [project selection flags...] --hotpatch
```

Public semantics:

- hotpatch is optional and native-only,
- it is a best-effort acceleration of the existing public dev loop,
- the tool may use the best available backend for the selected project,
- and when true hotpatch is unavailable or unsafe, the tool may fall back to a safe reload
  boundary or fast restart instead of promising a specific backend.

This means the **user-facing contract** is capability-oriented ("faster dev loop"), not
transport-oriented ("manage touch files" or "speak to dx directly").

## Public exclusions

These remain out of the public contract:

- top-level `hotpatch` subcommands:
  - `poke`
  - `path`
  - `status`
  - `watch`
- advanced `dev native` hotpatch transport flags:
  - `--hotpatch-reload`
  - `--hotpatch-trigger-path`
  - `--hotpatch-poll-ms`
  - `--hotpatch-devserver`
  - `--hotpatch-dx`
  - `--hotpatch-dx-ws`
  - `--hotpatch-build-id`

Reason:

- these flags expose implementation mechanics and workspace-era diagnostics ergonomics,
- not the smallest stable promise that an external app author should learn.

## Repo-only retained surface

The repo may keep richer hotpatch helpers on `fretboard-dev`:

- trigger-file manipulation (`poke`, `path`)
- log-tail and maintenance helpers (`status`, `watch`)
- explicit backend/transport overrides for `dx`, devserver websocket wiring, and reload-boundary
  debugging

Those are valid maintainer tools, but they are not the public app-author product surface.

## Preconditions for any future public hotpatch

Before `fretboard dev native --hotpatch` becomes public, all of the following must be true:

1. public `fretboard dev native` already exists and is project-facing,
2. the selected app target has a documented hotpatch-ready path,
3. the fallback behavior is safe and explicit,
4. docs can teach hotpatch as optional acceleration rather than required setup.

If these are not true, hotpatch stays entirely repo-only.

## Migration mapping

| Current repo-only surface | Public posture |
| --- | --- |
| `fretboard-dev hotpatch poke/path/status/watch` | repo-only |
| `fretboard-dev dev native --hotpatch` | future public candidate only after public `dev native` lands |
| `fretboard-dev dev native --hotpatch-reload` | repo-only |
| `fretboard-dev dev native --hotpatch-devserver ...` | repo-only |
| `fretboard-dev dev native --hotpatch-dx ...` | repo-only |

## Explicit non-targets

This document does **not** decide:

- whether public hotpatch ships in the same release as public `dev`,
- which internal backend wins between `dx`, devserver websocket, or reload-boundary mode,
- or whether additional public dev-loop accelerators (theme/assets/literals reload) land first.

## Done-state summary

The done state is not "publish every current hotpatch knob".

It is:

- public `dev` keeps ownership of the app-author run loop,
- hotpatch remains optional and subordinate to that run loop,
- the public promise is framed around capability and safe fallback,
- and repo-only transport/debug helpers stay internal.
