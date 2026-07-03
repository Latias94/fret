---
type: Subagent Finding
title: Phase 3 U13 advanced facade split audits
tags: fret,phase3,u13,advanced-facade,cookbook,surface-policy,subagent
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Finding

Three initial read-only explorers audited the U13 advanced facade split, and three follow-up
read-only explorers audited the remaining cookbook/source-policy/raw-helper migration lanes.

- `fret::advanced` already has natural lanes: `advanced::view`, `advanced::interop`,
  `advanced::dev`, driver/builder helpers, kernel aliases, and raw action/model traits.
- Raw traits must not be reachable through `advanced::prelude::*`; they should live behind
  `advanced::raw` and be imported at the call site that actually needs the runtime seam.
- Cookbook examples still need a second pass: some examples can leave `advanced::prelude::*`
  entirely, while true driver/view/interop/raw examples should import explicit lanes.
- Surface-policy quarantine records are too broad for the next phase. `fret-framework` is a
  manual/kernel facade, not a temporary quarantine; cookbook and examples should be classified as
  advanced cookbook, internal harness, migration reference, or manual facade.
- Follow-up cookbook audit found that `assets_reload_epoch_basics` and `effects_layer_basics` are
  low-risk app/default candidates, while command/keymap, form, text-input, router, undo, and drag
  examples need narrower app-facing helpers or explicit advanced lanes.
- Follow-up source-policy audit found that `ADVANCED_MANUAL_SURFACES` still misses many
  public-looking raw seams under `apps/fret-examples/src`, and that the broad
  `apps/fret-examples-imui/src` record should become per-file once the gate can categorize raw
  seam lanes.
- Follow-up raw-helper audit found no-new-API replacements for `paint_value_in` /
  `layout_value_in`, availability bool captures, and virtual-list `locals_with`; larger work needs
  IMUI model adapters, activation-only local mutation, action-effects transactions, router action
  binding, and output-local adapters.

# Evidence

- `ecosystem/fret/src/lib.rs` advanced facade exports.
- `apps/fret-cookbook/examples/*` advanced/raw imports.
- `tools/check_surface_policy.py` `ADVANCED_MANUAL_SURFACES`.
- Follow-up subagents:
  - `019f292a-9188-7802-9bef-8ac6a6ddcd95` cookbook advanced/prelude audit.
  - `019f292a-cb41-7882-a8b4-8b2c00a01710` source-policy quarantine audit.
  - `019f292b-0ff2-75c0-b930-525461a35fb9` raw helper audit.

# Recommendation

Land U13 in two slices:

1. Split the core facade by adding `advanced::raw` and `advanced::driver`, stop
   `advanced::prelude::*` from forwarding raw traits, and update docs/tests/imports to
   `advanced::raw`.
2. Reclassify cookbook/examples/quarantine records: remove obvious `advanced::prelude::*` from
   default-ish examples, keep true driver/view/interop/raw examples explicit, and add source-policy
   discovery for unclassified public-looking raw seams.
3. Split broad quarantine records by lane and file once discovery is in place, starting with
   `apps/fret-examples/src` and then replacing the broad `apps/fret-examples-imui/src` record.

# Disposition

The advanced raw/driver split, the cookbook migration slices, cookbook high-risk seam discovery,
and no-new-API raw-helper shrink have been implemented locally and verified. The next U13 action
should extend discovery to `apps/fret-examples/src` without adding a broad root quarantine, or start
the larger helper design work for examples that still truly coordinate shared runtime models.
