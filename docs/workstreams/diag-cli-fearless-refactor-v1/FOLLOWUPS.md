# Diag CLI Fearless Refactor v1 — Follow-up Lanes

Status: Closeout-ready handoff index
Last updated: 2026-03-26

Related:

- `docs/workstreams/diag-cli-fearless-refactor-v1/CLOSEOUT.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/OWNERSHIP.md`

## 0) Purpose

This file records the residual work that was split out after the parser reset lane was closed out.

Rule:

- do not reopen `diag-cli-fearless-refactor-v1` for broad “more parser cleanup” work
- continue in a narrow lane with explicit scope and exit criteria
- once a follow-up lane reaches its exit criteria, close it in place instead of folding work back
  into the original refactor lane

## 1) Main execution lane hardening

- Lane: `docs/workstreams/diag-cli-main-lanes-hardening-v1/README.md`
- Status: `Closeout-ready`
- Carries:
  - `DCR-core-021`
  - `DCR-core-022`
  - `DCR-core-023`
  - `DCR-core-024`
  - `DCR-core-027`
  - `DCR-core-028`
  - `DCR-core-029`
  - `DCR-core-030`

## 2) First-party caller migration

- Lane: `docs/workstreams/diag-cli-first-party-migration-v1/README.md`
- Status: `Closeout-ready`
- Carries:
  - `DCR-repo-050`
  - `DCR-repo-051`
  - `DCR-repo-052`
  - `DCR-repo-055`
  - `DCR-repo-056`

## 3) Help snapshots and smoke gates

- Lane: `docs/workstreams/diag-cli-help-and-gates-v1/README.md`
- Status: `Closeout-ready`
- Carries:
  - `DCR-repo-053`
  - `DCR-repo-054`

## 4) Index decision

This file remains the durable handoff index for the follow-up lanes, but the original three lanes
named here are now closeout-ready.

Any future `diag` CLI contract work should start from a new narrowly scoped workstream instead of
reusing these closed follow-up lanes.
