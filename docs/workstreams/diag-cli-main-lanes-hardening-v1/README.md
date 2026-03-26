# Diag CLI Main Lanes Hardening v1

Status: Proposed follow-up lane
Last updated: 2026-03-26

Source lane:

- `docs/workstreams/diag-cli-fearless-refactor-v1/FOLLOWUPS.md`

## Scope

Finish the remaining hardening for the main execution command families after the parser reset
landed:

- `run`
- `suite`
- `repro`
- `repeat`

## Carries

- `DCR-core-021`
- `DCR-core-022`
- `DCR-core-023`
- `DCR-core-024`
- `DCR-core-027`
- `DCR-core-028`
- `DCR-core-029`
- `DCR-core-030`

## Exit criteria

- parser-local validation for the main lanes is explicit and local
- help/examples for the main lanes are fully generated and current
- representative valid and invalid parser coverage exists for the main lanes
- no remaining “partial migration” wording is needed for `run` / `suite` / `repro` / `repeat`
