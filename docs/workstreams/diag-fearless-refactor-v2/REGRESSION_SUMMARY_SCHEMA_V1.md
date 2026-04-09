# Diag Fearless Refactor v2 — Regression Summary Schema v1

Status: Draft

Tracking docs:

- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-fearless-refactor-v2/REGRESSION_CAMPAIGN_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`

## 0) Why this note exists

`REGRESSION_CAMPAIGN_V1.md` defines the lane model and campaign concept.
This note defines the machine-readable summary artifact that campaign-style runs should emit.

Recommended output file:

- `regression.summary.json`

Goal:

- one top-level artifact for humans, CLI, CI, DevTools GUI, and MCP-style consumers,
- enough structure to classify failures and locate evidence quickly,
- small and stable enough to become a first-open artifact.

## 1) Design goals

The summary schema should be:

- **bounded**
  - it should summarize runs, not duplicate full bundle payloads,
- **stable**
  - consumers should not need to chase shifting ad-hoc keys,
- **reason-code friendly**
  - failures should be classifiable without free-form text parsing,
- **evidence-oriented**
  - every non-passing item should point to the best available artifacts,
- **presentation-neutral**
  - CLI, GUI, CI, and MCP should all be able to consume the same file.

## 2) Schema shape

Recommended top-level shape:

```json
{
  "schema_version": 1,
  "kind": "diag_regression_summary",
  "campaign": { ... },
  "run": { ... },
  "totals": { ... },
  "items": [ ... ],
  "highlights": { ... },
  "artifacts": { ... }
}
```

Notes:

- `schema_version` is required and explicit.
- `kind` is required to make mixed artifact directories safer to inspect.
- `items` is the main per-run result list.
- `highlights` is optional but recommended for quick scanning.
- `artifacts` is optional but recommended for top-level navigation.

## 3) Top-level fields

### 3.1 `campaign`

Recommended fields:

- `name: string`
- `lane: string`
- `profile: string | null`
- `schema_version: u32 | null`
- `requested_by: string | null`
- `filters: object | null`

Purpose:

- explain what logical run was requested,
- preserve lane/profile vocabulary,
- carry selection context without depending on CLI arguments being preserved elsewhere.

Vocabulary rule:

- `lane` should use the canonical lane set from
  `M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`.
- persisted broad scheduled runs should prefer `nightly`; `full` may be accepted as a user-facing
  alias at selection time but should not become a second persisted broad-lane term unless the repo
  later gives it different semantics.

### 3.2 `run`

Recommended fields:

- `run_id: string`
- `created_unix_ms: u64`
- `started_unix_ms: u64 | null`
- `finished_unix_ms: u64 | null`
- `duration_ms: u64 | null`
- `workspace_root: string | null`
- `out_dir: string | null`
- `tool: string`
- `tool_version: string | null`
- `git_commit: string | null`
- `git_branch: string | null`
- `host: object | null`

Purpose:

- make results attributable and reproducible,
- support later archive/search/report tooling.

### 3.3 `totals`

Recommended fields:

- `items_total: u32`
- `passed: u32`
- `failed_deterministic: u32`
- `failed_flaky: u32`
- `failed_tooling: u32`
- `failed_timeout: u32`
- `skipped_policy: u32`
- `quarantined: u32`

Purpose:

- provide a stable scoreboard,
- avoid forcing every consumer to re-derive status buckets.

### 3.4 `highlights`

Recommended optional fields:

- `first_failure: object | null`
- `worst_perf_failure: object | null`
- `flake_examples: array`
- `quarantine_examples: array`
- `top_reason_codes: array`

Purpose:

- give humans and dashboards a small first-open summary.

### 3.5 `artifacts`

Recommended optional fields:

- `summary_dir: string | null`
- `packed_report: string | null`
- `index_json: string | null`
- `html_report: string | null`

Purpose:

- give top-level navigation without embedding large payloads.

### 3.6 Optional companion index artifact

Recommended companion file for aggregate consumers:

- `regression.index.json`

Intent:

- keep `regression.summary.json` as the canonical per-run/per-campaign summary,
- allow GUI, CI, MCP, and local maintainer dashboards to open one lighter-weight index for
  counts and ranked lists,
- avoid forcing every consumer to recompute the same counters from all items repeatedly.

Recommended top-level fields for the index artifact:

- `schema_version`
- `kind = "diag_regression_index"`
- `generated_unix_ms`
- `out_dir`
- `counters`
- `top_reason_codes`
- `failing_summaries`
- `summaries`

Recommended `counters` fields:

- `by_lane`
- `by_status`
- `by_tool`
- `by_reason_code`

Recommended ranked helper fields:

- `top_reason_codes`: top N `{ reason_code, count }` rows for first-open diagnosis,
- `failing_summaries`: top N summary rows ordered by failure count.

Rules:

- the index artifact is additive and consumer-oriented,
- consumers must treat it as a convenience cache over the underlying summaries,
- if the index and underlying summaries disagree, the summaries remain the source of truth.

## 4) Per-item result shape

Recommended `items[]` shape:

```json
{
  "item_id": "...",
  "kind": "script|suite|matrix_case|perf_case|campaign_step",
  "name": "...",
  "status": "passed|failed_deterministic|failed_flaky|failed_tooling|failed_timeout|skipped_policy|quarantined",
  "reason_code": "...",
  "source_reason_code": "...",
  "lane": "smoke|correctness|matrix|perf|nightly",
  "owner": "...",
  "feature_tags": ["..."],
  "timing": { ... },
  "attempts": { ... },
  "evidence": { ... },
  "source": { ... },
  "notes": { ... }
}
```

Required minimum:

- `item_id`
- `kind`
- `name`
- `status`

Recommended strongly:

- `reason_code`
- `lane`
- `evidence`
- `source`

## 5) Item sub-objects

### 5.1 `timing`

Recommended fields:

- `duration_ms: u64 | null`
- `queue_delay_ms: u64 | null`
- `started_unix_ms: u64 | null`
- `finished_unix_ms: u64 | null`

### 5.2 `attempts`

Recommended fields:

- `attempts_total: u32`
- `attempts_passed: u32`
- `attempts_failed: u32`
- `retried: bool`
- `repeat_summary_path: string | null`
- `shrink_summary_path: string | null`

Purpose:

- model flake handling without bloating the top-level summary.

### 5.3 `evidence`

Recommended fields:

- `bundle_artifact: string | null`
- `bundle_dir: string | null`
- `triage_artifact: string | null`
- `script_result: string | null`
- `share_artifact: string | null`
- `packed_report: string | null`
- `screenshots_manifest: string | null`
- `perf_summary_json: string | null`
- `compare_json: string | null`
- `extra`: object | null

Purpose:

- keep one stable place to look for artifacts,
- allow lane-specific extras without widening the root schema too often.

Compatibility rule:

- producers may continue to read legacy names such as `triage_json`, `script_result_json`,
  `ai_packet_dir`, and `pack_path`,
- newly written docs and new emitters should prefer `triage_artifact`, `script_result`,
  `share_artifact`, and `packed_report`,
- consumers should ignore unknown fields and may support both spellings during migration.

### 5.4 `source`

Recommended fields:

- `script: string | null`
- `suite: string | null`
- `campaign_case: string | null`
- `metadata`: object | null

Purpose:

- show where the item came from in repo terms.

### 5.5 `notes`

Recommended fields:

- `summary: string | null`
- `details`: array

Rules:

- notes are for bounded human hints only,
- consumers must not rely on parsing note text to determine machine semantics.

## 6) Status and reason-code rules

### 6.1 `status`

Recommended enum:

- `passed`
- `failed_deterministic`
- `failed_flaky`
- `failed_tooling`
- `failed_timeout`
- `skipped_policy`
- `quarantined`

Rules:

- `status` is the canonical bucket for dashboards and summaries.
- `reason_code` refines the failure or skip cause.
- presentation surfaces may relabel these statuses for humans, but must not persist alternate
  machine-readable bucket names.

### 6.2 `reason_code`

Rules:

- use the normalized naming rules from `M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`,
- do not force consumers to parse free-form `reason` strings,
- campaign-level logic may wrap lower-level failures, but should preserve the original reason code when possible.

Suggested compatibility pattern:

- `reason_code`: the normalized campaign-level code,
- `source_reason_code`: the lower-level originating code when different.

Recommended first bucket families:

- `tooling.*`
- `script.*`
- `policy.*`
- `perf.*`

## 7) Lane-specific expectations

### 7.1 `smoke`

Expected evidence minimum:

- failing item has `reason_code`,
- failing item points to at least one bounded evidence artifact.

### 7.2 `correctness`

Expected evidence minimum:

- failing item points to the most relevant bundle artifact,
- optional screenshot references only when needed by the check.

### 7.3 `matrix`

Expected evidence minimum:

- item includes compare-related output,
- item can point to both compared sides when useful.

Suggested extra fields under `evidence.extra`:

- `left_bundle_artifact`
- `right_bundle_artifact`
- `compare_mode`

### 7.4 `perf`

Expected evidence minimum:

- item points to the selected worst evidence bundle,
- item points to perf summary output.

Suggested extra fields under `evidence.extra`:

- `baseline_path`
- `metric_key`
- `threshold_value`
- `actual_value`

## 8) Boundedness and size policy

The summary artifact should stay small enough to be a first-open file.

Recommended constraints:

- avoid embedding bundle contents,
- avoid embedding large stack traces or huge log text,
- keep `notes.details` bounded,
- push large payloads into referenced artifacts.

If a consumer needs deeper data, it should follow the artifact paths.

## 9) Compatibility policy

Recommended rules:

- additive fields are preferred,
- unknown fields must be ignored by consumers,
- `schema_version` must only change for incompatible structure changes,
- lane-specific extras should prefer `evidence.extra` or `source.metadata` before widening the root repeatedly,
- legacy evidence-field spellings may be supported during migration, but new schema notes and new
  emitters should prefer the canonical evidence path names from
  `M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`.

## 10) Suggested minimal example

```json
{
  "schema_version": 1,
  "kind": "diag_regression_summary",
  "campaign": {
    "name": "ui-gallery-pr",
    "lane": "smoke",
    "profile": "default"
  },
  "run": {
    "run_id": "20260306-001",
    "created_unix_ms": 1772760000000,
    "tool": "fretboard-dev diag campaign"
  },
  "totals": {
    "items_total": 3,
    "passed": 2,
    "failed_deterministic": 1,
    "failed_flaky": 0,
    "failed_tooling": 0,
    "failed_timeout": 0,
    "skipped_policy": 0,
    "quarantined": 0
  },
  "items": [
    {
      "item_id": "ui-gallery-dialog-escape-focus-restore",
      "kind": "script",
      "name": "dialog escape focus restore",
      "status": "failed_deterministic",
      "reason_code": "script.assert.focus_restore.mismatch",
      "lane": "smoke",
      "timing": { "duration_ms": 1420 },
      "attempts": {
        "attempts_total": 1,
        "attempts_passed": 0,
        "attempts_failed": 1,
        "retried": false,
        "repeat_summary_path": null,
        "shrink_summary_path": null
      },
      "evidence": {
        "bundle_artifact": "target/fret-diag/.../bundle.schema2.json",
        "bundle_dir": "target/fret-diag/...",
        "triage_artifact": "target/fret-diag/.../triage.json",
        "script_result": "target/fret-diag/.../script.result.json",
        "share_artifact": "target/fret-diag/.../ai.packet",
        "packed_report": null,
        "screenshots_manifest": null,
        "perf_summary_json": null,
        "compare_json": null,
        "extra": null
      },
      "source": {
        "script": "tools/diag-scripts/ui-gallery-dialog-escape-focus-restore.json",
        "suite": "ui-gallery",
        "campaign_case": null,
        "metadata": null
      },
      "notes": {
        "summary": "focus did not return to trigger",
        "details": []
      }
    }
  ],
  "highlights": {
    "first_failure": {
      "item_id": "ui-gallery-dialog-escape-focus-restore",
      "reason_code": "script.assert.focus_restore.mismatch"
    }
  },
  "artifacts": {
    "summary_dir": "target/fret-diag-campaign/20260306-001"
  }
}
```

## 11) Suggested aggregate index example

```json
{
  "schema_version": 1,
  "kind": "diag_regression_index",
  "generated_unix_ms": 1772760001234,
  "out_dir": "target/fret-diag-campaign/20260306-001",
  "counters": {
    "by_lane": { "correctness": 24, "perf": 8 },
    "by_status": { "passed": 28, "failed_deterministic": 4 },
    "by_tool": {
      "fretboard-dev diag suite": 24,
      "fretboard-dev diag perf": 8
    },
    "by_reason_code": {
      "script.assert.focus_restore.mismatch": 2,
      "diag.perf.threshold_failed": 2
    }
  },
  "top_reason_codes": [
    { "reason_code": "script.assert.focus_restore.mismatch", "count": 2 },
    { "reason_code": "diag.perf.threshold_failed", "count": 2 }
  ],
  "failing_summaries": [
    {
      "path": "target/fret-diag-campaign/20260306-001/perf/regression.summary.json",
      "campaign_name": "perf",
      "lane": "perf",
      "tool": "fretboard-dev diag perf",
      "failures": 2,
      "items_total": 8
    }
  ]
}
```

## 12) Definition of done for this note

This schema note is successful when:

- future campaign work can point to one stable summary artifact design,
- CI and GUI do not need separate ad-hoc result summary formats,
- the summary remains small enough to open first,
- failures are traceable to evidence without re-scanning large bundles.
