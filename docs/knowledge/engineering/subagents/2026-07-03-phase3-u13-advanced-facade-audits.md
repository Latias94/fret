---
type: Subagent Finding
title: Phase 3 U13 advanced facade split audits
tags: fret,phase3,u13,advanced-facade,cookbook,surface-policy,subagent
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Finding

Three read-only explorers audited the U13 advanced facade split.

- `fret::advanced` already has natural lanes: `advanced::view`, `advanced::interop`,
  `advanced::dev`, driver/builder helpers, kernel aliases, and raw action/model traits.
- Raw traits must not be reachable through `advanced::prelude::*`; they should live behind
  `advanced::raw` and be imported at the call site that actually needs the runtime seam.
- Cookbook examples still need a second pass: some examples can leave `advanced::prelude::*`
  entirely, while true driver/view/interop/raw examples should import explicit lanes.
- Surface-policy quarantine records are too broad for the next phase. `fret-framework` is a
  manual/kernel facade, not a temporary quarantine; cookbook and examples should be classified as
  advanced cookbook, internal harness, migration reference, or manual facade.

# Evidence

- `ecosystem/fret/src/lib.rs` advanced facade exports.
- `apps/fret-cookbook/examples/*` advanced/raw imports.
- `tools/check_surface_policy.py` `ADVANCED_MANUAL_SURFACES`.

# Recommendation

Land U13 in two slices:

1. Split the core facade by adding `advanced::raw` and `advanced::driver`, stop
   `advanced::prelude::*` from forwarding raw traits, and update docs/tests/imports to
   `advanced::raw`.
2. Reclassify cookbook/examples/quarantine records: remove obvious `advanced::prelude::*` from
   default-ish examples, keep true driver/view/interop/raw examples explicit, and add surface-policy
   checks for unused/overbroad allowed raw seams.

# Disposition

The first slice has been implemented locally and verified. The second slice remains the next U13
action before U14.

