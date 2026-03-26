# Diag CLI Help And Gates v1

Status: Proposed follow-up lane
Last updated: 2026-03-26

Source lane:

- `docs/workstreams/diag-cli-fearless-refactor-v1/FOLLOWUPS.md`

## Scope

Add durable regression protection around the user-facing CLI surface after the parser reset landed.

## Carries

- `DCR-repo-053`
- `DCR-repo-054`

## Exit criteria

- help output has snapshot coverage or equivalent drift guards
- the highest-risk command families have focused smoke coverage:
  - `run`
  - `suite`
  - `repro`
  - `perf`
  - `campaign`
