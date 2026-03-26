# Diag CLI First-party Migration v1

Status: Proposed follow-up lane
Last updated: 2026-03-26

Source lane:

- `docs/workstreams/diag-cli-fearless-refactor-v1/FOLLOWUPS.md`

## Scope

Finish the repo-owned caller migration after the parser reset landed.

This lane covers docs, helper scripts, parser-sensitive tests, and repo-wide grep cleanup for
deleted command spellings.

## Carries

- `DCR-repo-050`
- `DCR-repo-051`
- `DCR-repo-052`
- `DCR-repo-055`
- `DCR-repo-056`

## Exit criteria

- repo-owned docs teach only the intended CLI surface
- helper scripts and maintainer notes no longer rely on deleted spellings
- parser-sensitive tests are updated to the intended surface
- repo grep no longer finds stale deleted syntax except where explicitly labeled historical
