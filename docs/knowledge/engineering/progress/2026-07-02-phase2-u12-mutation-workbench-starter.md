---
type: Work Progress
title: Phase 2 U12 mutation-workbench starter
tags: fret,phase2,u12,scaffold,mutation,query,diagnostics,public-facade
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Phase 2 U12 Mutation-Workbench Starter

## Summary

Phase 2 U12 adds `fretboard new mutation-workbench`, the async public-app starter after
`workbench-lite`. The generated app demonstrates mutation submit, retry, toast feedback, query
invalidation, and app-owned async runtime setup while staying on the public `AppUi` facade.

The generated source intentionally imports only the default app prelude plus explicit
`fret::mutation`, `fret::query`, and `fret::style` nouns. It does not import raw runtime crates,
host adapters, raw element erasure, model-store plumbing, or retained tree mechanisms.

## Changes

- Added `mutation-workbench` to the public scaffold contract, repo scaffold mode, interactive
  wizard, and root help examples.
- Added a generated app template with an in-memory `PresetCatalog`, async query loading, mutation
  submit/retry actions, local completion projection, query namespace invalidation, and
  shadcn/Sonner success/error toasts.
- Mounted `shadcn::Toaster::new()` as an ordinary typed child so the generated app does not teach
  `cx.elements()` or a raw element-context seam.
- Made the forced-error path one-shot by storing an `Arc<AtomicBool>` inside the mutation input.
  This preserves the user-facing "fail next save" meaning while allowing `mutation_retry_last` to
  succeed with the same payload.
- Moved status diagnostics selectors onto text nodes instead of shadcn `Badge` roots. The public
  diag script can now assert `label_contains` against actual visible status/count text without
  depending on Badge internals.
- Added `tools/diag-scripts/public-app/mutation-workbench-flow.json`, covering initial empty state,
  submit success, query refresh, forced error, editable input preservation, retry success, and final
  bundle capture.
- Updated first-hour, examples, and crate-usage docs to route async public-app authors to
  `mutation-workbench` before copying advanced demos.

## Verification

Verification passed before commit:

- `cargo nextest run -p fretboard mutation_workbench --no-fail-fast`
- `cargo nextest run -p fretboard scaffold --no-fail-fast`
- `cargo check --manifest-path local/u12-mutation-workbench/Cargo.toml`
- `cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/public-app/mutation-workbench-flow.json`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/public-app/mutation-workbench-flow.json --timeout-ms 240000 --launch -- cargo run --manifest-path local/u12-mutation-workbench/Cargo.toml`
- `cargo nextest run -p fret --lib --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `git diff --check`

## Next Action

Continue to U13: replace broad advanced allowlists with retiring quarantine records. Use the new
mutation starter as a public-source-policy fixture and avoid adding raw runtime names to generated
README text, even in "do not use this" phrasing.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [Scaffold templates](../../../../crates/fretboard/src/scaffold/templates.rs)
- [Scaffold contracts](../../../../crates/fretboard/src/scaffold/contracts.rs)
- [Public mutation diagnostics script](../../../../tools/diag-scripts/public-app/mutation-workbench-flow.json)
- [First-hour guide](../../../first-hour.md)
