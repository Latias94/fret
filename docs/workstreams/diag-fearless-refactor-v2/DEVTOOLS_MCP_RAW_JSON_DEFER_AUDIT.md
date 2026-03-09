---
title: DevTools MCP Raw JSON Defer Audit
status: draft
date: 2026-03-09
scope: diagnostics, devtools, mcp, raw-json, naming, audit
---

# DevTools MCP Raw JSON Defer Audit

Status: Draft

Tracking docs:

- `docs/workstreams/diag-fearless-refactor-v2/M3_ORCHESTRATION_VOCABULARY_AND_CONTRACT_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/ARTIFACT_AND_EVIDENCE_MODEL_V1.md`
- `docs/workstreams/diag-fearless-refactor-v2/M3_VOCABULARY_ADOPTION_AUDIT.md`

## 0) Why this audit exists

The repo now has a clearer M3 vocabulary contract for persisted artifacts.

What still causes naming anxiety is a different class of names inside:

- `apps/fret-devtools/src/native.rs`
- `apps/fret-devtools-mcp/src/native.rs`

Examples include:

- `last_bundle_json`
- `last_script_result_json`
- `regression_summary_json`
- `regression_index_json`
- `bundle_json`
- `payload_json`

This audit answers one narrow question:

- should these names be renamed to match canonical artifact-path vocabulary now,
- or should they remain deferred because they mostly represent raw text payload holders?

## 1) Audit scope

Included:

- DevTools GUI state fields that cache JSON text for display,
- MCP request/response fields that explicitly return JSON text bodies,
- helper functions that materialize or reserialize raw bundle payload text.

Not included:

- canonical persisted artifact file names,
- `crates/fret-diag` summary/result/evidence schema fields,
- run-manifest or summary machine-field naming inside `crates/fret-diag`.

## 2) Current implementation shape

### 2.1 DevTools GUI state mostly stores raw JSON text for tabs

The DevTools state contains fields such as:

- `last_pick_json`
- `last_inspect_hover_json`
- `last_script_result_json`
- `last_bundle_json`
- `last_screenshot_json`
- `regression_summary_json`
- `regression_index_json`

These are stored as `Model<String>` and are used to feed text tabs or drill-down panels.

Evidence:

- `apps/fret-devtools/src/native.rs`

Audit judgment:

- these are view-model text caches, not canonical artifact-path contract fields.

### 2.2 DevTools regression refresh reads artifact files into text models

`refresh_regression_artifacts(...)` reads:

- `regression.summary.json`
- `regression.index.json`

Then stores the file contents into:

- `regression_summary_json`
- `regression_index_json`

It separately keeps:

- `regression_loaded_dir`
- `regression_selected_summary_path`

Audit judgment:

- the canonical path contract already exists in the path-bearing fields and filenames,
- the `*_json` fields are text snapshots for UI presentation and parsing convenience.

### 2.3 MCP exposes explicit JSON text payloads as tool results

Examples:

- `payload_json: Option<String>`
- `bundle_json: Option<String>`
- `script_json: String`
- `selector_json: Option<String>`

The descriptions in the tool surface already say things like:

- "Return the latest bundle.json text"
- "JSON text for a `UiActionScriptV1` or `UiActionScriptV2` payload"

Audit judgment:

- these names are semantically correct because the API is intentionally exchanging raw JSON text,
- not abstract artifact-path handles.

### 2.4 MCP bundle helpers also operate on text bodies, not only file paths

`bundle_json_from_bundle_dumped_payload(...)` returns `Result<String, String>`.

When the payload already contains an inline `bundle`, it serializes that value back to formatted
JSON text.

When the payload only references disk artifacts, it reads `bundle.json` and returns the file text.

Audit judgment:

- `bundle_json` in this area means "bundle JSON body", not "canonical bundle artifact vocabulary".

## 3) Contract interpretation

The M3 and artifact/evidence notes distinguish between:

- canonical persisted artifact names and machine fields,
- derived projections,
- and presentation-facing views.

The DevTools/MCP `*json` names in this audit mostly sit in the third bucket:

- text-holder state,
- request/response payload text,
- UI panel buffers,
- convenience serialization boundaries.

That means they should not be used as evidence that the persisted artifact contract is drifting.

## 4) Audit conclusion

Current recommendation:

- keep these DevTools/MCP `*json` names deferred,
- do not do a broad rename wave now,
- and only touch them when the module is already being changed for a stronger reason.

Why this is the correct boundary:

1. these names mostly describe JSON text bodies, not persisted contract fields,
2. renaming them now would create high churn with low contract value,
3. the real contract risk lives in shared persisted artifacts under `crates/fret-diag`,
4. DevTools/MCP already consume shared summary/index projections for aggregate semantics instead of
   inventing a second persisted schema.

## 5) What would justify renaming later

Renaming should happen only if one of these becomes true:

1. a field stops carrying raw JSON text and starts carrying a canonical artifact path,
2. a GUI/MCP-local name leaks into a shared machine-readable schema,
3. the module is being substantially refactored anyway and the rename removes real ambiguity rather
   than only changing spelling.

## 6) Safe rule of thumb going forward

Use this rule:

- if the value is a raw JSON text blob used for display, editing, or request/response transfer,
  `*_json` is acceptable,
- if the value is a persisted artifact path or canonical machine field shared across consumers, it
  should follow the repo-level artifact vocabulary instead.

## 7) Recommended next action

No code change is required from this audit.

The right action is:

- record this defer boundary in the roadmap,
- keep shared artifact naming work focused under `crates/fret-diag`,
- and avoid mixing view-model text-holder cleanup with contract-surface cleanup.

## 8) Exit criteria for this audit

This audit has done its job when:

- contributors stop treating DevTools/MCP text-holder names as contract drift by default,
- future naming work stays focused on persisted shared artifacts,
- and any later rename in these apps is justified by actual data-shape changes rather than
  vocabulary anxiety alone.
