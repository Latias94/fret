---
title: Diag Concurrency (Multiple Agents) - Sessions Design v1
status: draft
date: 2026-02-28
scope: diagnostics, automation, filesystem-transport, fearless-refactor
---

# Diag concurrency (multiple agents) - sessions design v1

## Problem

The diagnostics filesystem transport is intentionally simple: the tool and the runtime coordinate via files under the
diagnostics output directory (out dir). This includes:

- control-plane triggers: `trigger.touch`, `script.touch`, `pick.result.touch`, ...
- request payloads: `script.json`, `dump.request.json`, ...
- result payloads: `script.result.json`, `pick.result.json`, ...
- convenience pointers: `latest.txt` (points to the most recently exported bundle dir)

This design assumes a single writer/driver per out dir. When multiple concurrent runs share the same out dir (e.g.
multiple terminals, multiple AI agents, multiple demos), they will race on these files and produce:

- non-deterministic `latest.txt`,
- spurious timeouts (`--wait` polling sees the wrong pointer),
- mismatched `script.json` vs `script.result.json`,
- mixed artifacts across runs, making triage misleading.

Today the practical workaround is “always use a unique `--dir` per agent/task”, but we want a design that is resilient
to parallel agent workflows and reduces footguns.

## Goals

1. Make “parallel agents” safe by default for tool-launched runs (`--launch`).
2. Keep the artifact layout small-by-default and deterministic.
3. Preserve the existing transport for manual runs (where the app is already running), while making the concurrency
   boundary explicit.
4. Keep the migration additive and reviewable (fearless refactor): no batch rewrites.

Non-goals (v1):

- Distributed locking across machines.
- Supporting multiple independent script drivers talking to the same running app instance via filesystem transport (use
  DevTools WS for multi-client scenarios).

## Key design principle: `out_dir` is a session boundary

We treat the diagnostics out dir as a **session root**: only one active driver should own it at a time.

This implies:

- For agent automation, the recommended workflow is “allocate a fresh session dir → run everything inside it”.
- `latest.txt` remains useful, but only within a session root (it is not a global pointer for multiple writers).

## Proposed design (v1): session roots under a base directory

Introduce the concept of a **base dir** (human-friendly bucket) and a **session dir** (exclusive owner).

Example layout:

```text
target/fret-diag/
  sessions/
    1772345678901-12345/
      diag.config.json
      trigger.touch
      ready.touch
      exit.touch
      script.json
      script.touch
      script.result.json
      latest.txt
      1772345679000-ui-gallery-gesture-tap-smoke/
        bundle.schema2.json
      42/
        manifest.json
        script.result.json
        bundle.schema2.json
```

Where:

- `target/fret-diag/` is the base dir (optional; can be any `--dir`).
- `sessions/<session_id>/` is the session root (tooling-owned).
- per-run dirs remain `<session_dir>/<run_id>/...` (manifest-first).
- bundle export dirs remain under the same session root and `latest.txt` points to the latest export within the session.

### Session id

Session ids must be:

- unique enough for parallel invocations,
- short (path-length friendly on Windows),
- safe for filenames.

Recommended format: `<unix_ms>-<pid>` (optionally with a short suffix if needed).

### Session metadata

Tooling writes a small, best-effort `session.json` into the session dir:

- `schema_version`
- `created_unix_ms`
- `pid`
- `tool_cmd` (optional)
- `base_dir` (optional)

This is purely for human/agent discoverability and does not affect runtime behavior.

## CLI surface (proposed)

### Tool-launched runs (preferred)

Add an opt-in flag to create a session dir under `--dir` and run everything there:

- `--dir <base_dir> --session-auto`
  - tooling picks a fresh `session_id`
  - tooling uses `<base_dir>/sessions/<session_id>/` as the effective out dir
  - tooling prints the effective out dir at the start of the command

Optionally allow explicit reuse:

- `--dir <base_dir> --session <session_id>`

### Manual runs

Manual runs cannot be “retargeted” by tooling after the app starts (filesystem transport). Therefore:

- session isolation is only meaningful if the app is started with `FRET_DIAG_DIR=<session_dir>` (or a config file that
  points to it).
- tooling should document this clearly and avoid pretending it can isolate manual runs.

## Migration plan (fearless refactor)

P0 (docs + workflow hygiene):

- Document that `--dir` / `FRET_DIAG_DIR` is a session boundary and must not be shared by concurrent agents.
- Update the `fret-diag-workflow` skill to recommend per-agent `--dir`.

P1 (tooling support, additive):

- Implement `--session-auto` / `--session` for tool-launched commands (`diag run/suite/repro/perf/repeat/script shrink`)
  to isolate control-plane files automatically.
- Keep existing behavior unchanged when the flags are not used.

P2 (de-risk `latest.txt`):

- Make tooling prefer per-run `manifest.json` + `script.result.json` for “what just happened”, with `latest.txt` as a
  best-effort convenience pointer inside a session dir.

## Agent workflows (recommended patterns)

1. One agent → one base dir:

- `--dir target/fret-diag-agent-a --session-auto`

2. One agent → multiple tasks:

- keep the base dir stable, allocate a fresh session per task:
  - `--dir target/fret-diag-agent-a --session-auto`

3. Multi-agent swarm:

- each agent gets its own base dir (for easier cleanup),
- each command allocates a fresh session under that base.

## Failure modes and mitigations

- If users forget `--dir` and all agents write to `target/fret-diag/`, runs will race.
  - Mitigation (P0): skill/docs + `diag config doctor` warnings.
  - Mitigation (P1): `--session-auto` makes it easy to avoid shared roots without inventing naming conventions.

## Open questions

- Should `--session-auto` become the default for tool-launched runs (`--launch`) in the future?
  - Pros: eliminates the biggest concurrency footgun.
  - Cons: changes default artifact locations; may surprise existing scripts relying on `target/fret-diag/latest.txt`.

