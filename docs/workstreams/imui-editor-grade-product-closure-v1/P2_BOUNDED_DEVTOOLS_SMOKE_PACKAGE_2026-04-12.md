# P2 Bounded DevTools Smoke Package - 2026-04-12

Status: focused P2 gate decision / first-open loop smoke freeze

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
- `P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md`
- `tools/diag_gate_imui_p2_devtools_first_open.py`
- `tools/diag-campaigns/devtools-first-open-smoke.json`
- `tools/diag-scripts/tooling/todo/todo-baseline.json`
- `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_AUTOMATION_FLOW.md`
- `tools/diag-campaigns/README.md`

## Purpose

P2 still needed one bounded proof package for this question:

> what is the smallest repo-owned gate that proves the first-open diagnostics/devtools loop as one
> workflow instead of as isolated `diag` commands?

This note freezes that package before P2 expands into a broader tooling-product lane.

## Audited evidence

- `P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
- `P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md`
- `tools/diag-scripts/tooling/todo/todo-baseline.json`
- `crates/fret-diag/src/diag_run.rs`
- `crates/fret-diag/src/diag_compare.rs`
- `crates/fret-diag/src/diag_dashboard.rs`
- `crates/fret-diag/src/diag_campaign.rs`
- `crates/fret-diag/src/commands/resolve.rs`
- `tools/diag-campaigns/README.md`
- `docs/workstreams/diag-fearless-refactor-v2/MAINTAINER_AUTOMATION_FLOW.md`

## Assumptions-first resume set

1. Confident: `tools/diag-scripts/tooling/todo/todo-baseline.json` is the smallest repo-owned
   script that already exercises a real first-open loop with multiple named `capture_bundle`
   checkpoints.
   Evidence:
   - the script adds, toggles, and removes one row,
   - it emits `todo-after-add`, `todo-after-toggle-done`, and `todo-after-remove`.
   Consequence if wrong:
   - the smoke package would need a new script before it could prove compare or latest-bundle
     resolution.
2. Confident: direct compare should be validated as an artifacts-layer verdict, not as a
   "must-report-no-diff" check.
   Evidence:
   - the first-open path already treats `diag compare` as the portable bundle/session diff entry,
   - this bounded todo script intentionally moves through three different UI states.
   Consequence if wrong:
   - the smoke package would overfit to self-compare or a fake no-op scenario and stop proving the
     real compare path.
3. Confident: aggregate first-open consumer proof belongs in a campaign root, not in GUI-only
   state.
   Evidence:
   - `tools/diag-campaigns/README.md` and maintainer automation docs both treat
     `campaigns/<campaign_id>/<run_id>/` as the shared handoff root,
   - `regression.summary.json` and `regression.index.json` are the portable aggregate artifacts
     used by CLI, DevTools, and MCP.
   Consequence if wrong:
   - P2 would keep teaching another consumer-specific state model instead of the shared artifacts
     contract.

## Frozen smoke package

From this point forward, the bounded P2 smoke package is:

- gate entry:
  `python3 tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke`
- bounded aggregate manifest:
  `tools/diag-campaigns/devtools-first-open-smoke.json`
- bounded direct script:
  `tools/diag-scripts/tooling/todo/todo-baseline.json`

The gate intentionally proves two halves of the first-open story.

### A) Direct script -> bundle -> latest -> compare half

The gate must:

1. build `fretboard-dev` and `todo_demo`, then run
   `diag run tools/diag-scripts/tooling/todo/todo-baseline.json --session-auto --launch`
   against the built `todo_demo` binary,
2. verify that the session root contains the three named bundle directories:
   - `todo-after-add`
   - `todo-after-toggle-done`
   - `todo-after-remove`
3. verify that `diag resolve latest` and `diag latest` resolve through
   `script.result.json:last_bundle_dir`,
4. run direct `diag compare` over `todo-after-add` vs `todo-after-toggle-done`,
5. require a structured diff report rather than a transport/tooling error.

Important rule:

- this compare step is expected to produce a non-empty diff because the script intentionally moves
  through different UI states,
- the success condition is "portable compare verdict exists and is machine-readable", not
  "`ok=true`".

### B) Aggregate campaign root -> dashboard half

The gate must:

1. run
   `diag campaign run devtools-first-open-smoke --launch`
   against the same built `todo_demo` binary,
2. locate the resulting
   `campaigns/devtools-first-open-smoke/<run_id>/` root,
3. run
   `diag summarize <campaign_root> --dir <campaign_root> --json`
   so the nested script-level regression summary is promoted into the shared root,
4. require these shared artifacts:
   - `campaign.manifest.json`
   - `regression.summary.json`
   - `regression.index.json`
5. run `diag dashboard <campaign_root> --json`,
6. require the dashboard JSON load to succeed against the shared root index and expose at least one
   summarized entry plus at least one aggregate item.

Important rule:

- this half exists to prove the aggregate handoff root that DevTools GUI and MCP consume,
- do not replace it with GUI-only smoke or a second summary/index schema.

## Decision

From this point forward:

1. `python3 tools/diag_gate_imui_p2_devtools_first_open.py` is the bounded repo-owned P2 smoke
   package.
2. The package stays CLI-first for evidence production.
3. The direct run half proves:
   - script execution,
   - named bundle emission,
   - latest bundle resolution through `script.result.json:last_bundle_dir`,
   - and direct compare as a shared artifacts-layer verdict.
4. The campaign/dashboard half proves:
   - one aggregate handoff root,
   - explicit root `diag summarize` over the campaign root,
   - shared `regression.summary.json` / `regression.index.json`,
   - and dashboard consumption without DevTools-specific state.
5. Do not create a second GUI-only or MCP-only smoke package for this same first-open contract.

## Immediate execution consequence

For this lane:

- treat the bounded smoke package above as the default P2 regression entry,
- use it when validating that the first-open diagnostics story still hangs together after refactors,
- and leave the remaining open P2 work focused on teaching/discoverability rather than on creating
  another tooling architecture.
