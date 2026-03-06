# Diag Fearless Refactor v2 - Maintainer Automation Flow

Status: Draft

Tracking doc: `docs/workstreams/diag-fearless-refactor-v2/README.md`

Related notes:

- `docs/workstreams/diag-fearless-refactor-v2/CAMPAIGN_EXECUTION_ENTRY_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_SUMMARY_SCHEMA_V1.md`
- `tools/diag-campaigns/README.md`

## Purpose

This note captures the intended maintainer workflow for diagnostics automation after the first
campaign execution and batch artifact slices.

The goal is not to invent a second tool flow for DevTools or CI.

The goal is to keep one canonical loop that all consumers can follow:

1. choose or author the right script,
2. run a suite or campaign,
3. inspect the aggregate summary/index,
4. generate bounded share artifacts when something fails.

## Recommended workflow

### 1. Choose the smallest useful scope

Prefer the smallest stable unit that still reproduces the problem:

- one script when the issue is already isolated,
- one suite when the issue belongs to one surface area,
- one campaign when the issue should be tracked as a maintainer or CI lane.

Examples:

- `cargo run -p fretboard -- diag run <script_id> --launch -- <cmd...>`
- `cargo run -p fretboard -- diag suite <suite_id> --launch -- <cmd...>`
- `cargo run -p fretboard -- diag campaign run <campaign_id> --launch -- <cmd...>`
- `cargo run -p fretboard -- diag campaign run --lane smoke --tag ui-gallery --platform native --launch -- <cmd...>`

### 2. Treat the aggregate root as the handoff surface

After a campaign run, open the persisted root before drilling into individual bundles.

Per-campaign root:

- `campaigns/<campaign_id>/<run_id>/`

Batch root for filtered or multi-id selection:

- `campaign-batches/<selection_slug>/<run_id>/`

Important files:

- `campaign.manifest.json` or `batch.manifest.json`
- `campaign.result.json` or `batch.result.json`
- `regression.summary.json`
- `regression.index.json`

These files are the intended first-open contract for CLI, DevTools, and future GUI/MCP consumers.

### 3. Inspect aggregate results first

Recommended order:

1. `regression.summary.json` for counters and top failures,
2. `regression.index.json` for consumer-oriented navigation,
3. per-item evidence only after the failing item set is clear.

Examples:

- `cargo run -p fretboard -- diag dashboard <campaign_or_batch_root>`
- `cargo run -p fretboard -- diag summarize <campaign_or_batch_root> --json`

### 4. Generate bounded share artifacts

When a campaign or batch contains failures, prefer one bounded share step instead of hand-picking
bundle directories manually.

Examples:

- `cargo run -p fretboard -- diag campaign share <campaign_or_batch_root>`
- `cargo run -p fretboard -- diag campaign share <campaign_or_batch_root> --json`

Current behavior:

- reads `regression.summary.json` from the selected root,
- defaults to failed items only,
- generates AI-only share zips under `<root>/share/*.ai.zip`,
- writes `<root>/share/share.manifest.json` with the selected items, share zip paths, and errors.

This keeps the default maintainer handoff bounded while still preserving the aggregate root as the
main directory that DevTools and dashboard-style consumers should open.

## Recommended maintainer loop

### Authoring loop

Use this while iterating on one issue:

1. author or patch one script,
2. run one script or one suite,
3. fix determinism problems first,
4. only promote to a campaign once the scenario is worth keeping as a lane-level regression.

### Regression lane loop

Use this after a behavior lands or changes:

1. run one campaign or one filtered campaign batch,
2. inspect `regression.summary.json`,
3. if failed, run `diag campaign share <root>`,
4. attach `share/share.manifest.json` and the generated `*.ai.zip` outputs to the handoff.

## Why this flow is recommended

This preserves the workstream boundary decisions:

- diagnostics remains a general automation and evidence platform,
- DevTools stays a consumer lane instead of becoming the source of truth,
- campaign orchestration composes existing script/suite primitives,
- sharing is bounded by default instead of requiring full bundle artifacts every time.

## Still intentionally open

This note does not decide the following yet:

- whether campaign roots should also emit `dashboard.txt` or HTML,
- whether failing evidence bundles should be generated automatically during `run`,
- whether campaign manifests stay JSON-only forever.
