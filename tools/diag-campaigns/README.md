# Diagnostics campaign manifests

This folder contains repo-owned campaign manifests consumed by `fretboard-dev diag campaign`.

Current rules:

- one file per campaign (`*.json`),
- `schema_version: 1`,
- `kind: "diag_campaign_manifest"`,
- canonical authoring uses ordered `items`,
- each item has `kind` (`suite` or `script`) plus `value`,
- legacy top-level `suites` / `scripts` is still accepted for compatibility,
- supported metadata fields include `owner`, `platforms`, `tier`, `expected_duration_ms`,
  `requires_capabilities`, and `requires_environment`,
- manifest entries override same-id built-in fallback definitions.

Environment requirements:

- `requires_capabilities` stays capabilities-only.
- `requires_environment` is a separate source-scoped admission surface.
- v1 currently supports these admitted source/predicate pairs:
  - `source_id: "host.monitor_topology"`
  - `predicate.kind: "host_monitor_topology"`
- `source_id: "platform.capabilities"`
- `predicate.kind: "platform_capabilities"`
- `host_monitor_topology` threshold keys:
  - `monitor_count_ge`
  - `distinct_scale_factor_count_ge`
- `platform_capabilities` expectation keys:
  - `platform_is`
  - `ui_multi_window_is`
  - `ui_window_tear_off_is`
  - `ui_window_hover_detection_is`
  - `ui_window_z_level_is`
- At least one threshold/expectation must be present.

Example:

```json
{
  "requires_environment": [
    {
      "source_id": "host.monitor_topology",
      "predicate": {
        "kind": "host_monitor_topology",
        "monitor_count_ge": 2,
        "distinct_scale_factor_count_ge": 2
      }
    }
  ]
}
```

```json
{
  "requires_environment": [
    {
      "source_id": "platform.capabilities",
      "predicate": {
        "kind": "platform_capabilities",
        "platform_is": "linux",
        "ui_window_hover_detection_is": "none"
      }
    }
  ]
}
```

Run artifact layout:

- single campaign runs write under `campaigns/<campaign_id>/<run_id>/`,
- filtered or multi-id runs that select more than one campaign also write a batch root under
  `campaign-batches/<selection_slug>/<run_id>/`,
- batch roots persist `batch.manifest.json`, `batch.result.json`, `regression.summary.json`, and
  `regression.index.json`,
- batch summary/index reuse the existing `diag summarize` aggregate contract so DevTools, MCP, and
  `diag dashboard` can open one shared handoff directory.

Example:

- `cargo run -p fretboard-dev -- diag campaign list --json`
- `cargo run -p fretboard-dev -- diag campaign list --lane smoke --tag ui-gallery --platform native`
- `cargo run -p fretboard-dev -- diag campaign validate`
- `cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/ui-gallery-smoke.json --json`
- `cargo run -p fretboard-dev -- diag doctor campaigns --json`
- `cargo run -p fretboard-dev -- diag campaign show ui-gallery-smoke --json`
- `cargo run -p fretboard-dev -- diag campaign show ui-gallery-accordion-script-smoke --json`
- `cargo run -p fretboard-dev -- diag campaign run ui-gallery-smoke --launch -- <cmd...>`
- `cargo run -p fretboard-dev -- diag campaign run --lane smoke --tag ui-gallery --platform native --launch -- <cmd...>`
- `cargo run -p fretboard-dev -- diag campaign share target/fret-diag/campaigns/ui-gallery-smoke/<run_id>`
- `cargo run -p fretboard-dev -- diag campaign share target/fret-diag/campaign-batches/<selection_slug>/<run_id> --json`

`diag campaign validate` behavior:

- with no explicit manifest paths, validates all repo-owned manifests under `tools/diag-campaigns/*.json`,
- with one or more explicit manifest paths, validates only those files,
- reuses the campaign manifest loader contract already used by registry loading,
- supports text or `--json` output for authoring and CI-facing checks.

`diag doctor campaigns` behavior:

- checks the repo-owned manifest set under `tools/diag-campaigns/` as a read-only maintainer preflight,
- reports invalid manifests, duplicate campaign ids, and remaining legacy top-level `suites` / `scripts` authoring shape,
- keeps `diag campaign validate` as the explicit ad hoc validation entrypoint for one-off manifest paths.

`diag campaign share` behavior:

- reads `regression.summary.json` from a campaign or batch root,
- defaults to failed items only (`--include-passed` expands the selection),
- generates bounded AI-only share zips under `<root>/share/*.ai.zip`,
- writes `<root>/share/share.manifest.json` so maintainers, DevTools, and future GUI flows can
  treat one directory as the stable handoff surface.

Automatic failure evidence:

- `diag campaign run` now best-effort exports `share/share.manifest.json` automatically for failed
  campaign roots when `regression.summary.json` is available,
- filtered or multi-id campaign batches also best-effort export a batch-level
  `share/share.manifest.json` when any selected campaign fails,
- share manifest items now also record best-effort `triage.json` paths when the underlying bundle
  artifact is available,
- share manifest items now also record `screenshots_manifest` when a bundle-aligned screenshots
  manifest can be resolved,
- share roots now also best-effort write `share/combined-failures.zip`, which bundles the share
  manifest, aggregate summary/index, per-item AI zips, generated `triage.json`, and screenshot
  manifest JSON files,
- campaign and batch result manifests record `aggregate.share_manifest_path` plus `share_error`
  when export fails.
