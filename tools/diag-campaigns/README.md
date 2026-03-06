# Diagnostics campaign manifests

This folder contains repo-owned campaign manifests consumed by `fretboard diag campaign`.

Current rules:

- one file per campaign (`*.json`),
- `schema_version: 1`,
- `kind: "diag_campaign_manifest"`,
- canonical authoring uses ordered `items`,
- each item has `kind` (`suite` or `script`) plus `value`,
- legacy top-level `suites` / `scripts` is still accepted for compatibility,
- supported metadata fields include `owner`, `platforms`, `tier`, and `expected_duration_ms`,
- manifest entries override same-id built-in fallback definitions.

Run artifact layout:

- single campaign runs write under `campaigns/<campaign_id>/<run_id>/`,
- filtered or multi-id runs that select more than one campaign also write a batch root under
  `campaign-batches/<selection_slug>/<run_id>/`,
- batch roots persist `batch.manifest.json`, `batch.result.json`, `regression.summary.json`, and
  `regression.index.json`,
- batch summary/index reuse the existing `diag summarize` aggregate contract so DevTools, MCP, and
  `diag dashboard` can open one shared handoff directory.

Example:

- `cargo run -p fretboard -- diag campaign list --json`
- `cargo run -p fretboard -- diag campaign list --lane smoke --tag ui-gallery --platform native`
- `cargo run -p fretboard -- diag campaign show ui-gallery-smoke --json`
- `cargo run -p fretboard -- diag campaign show ui-gallery-accordion-script-smoke --json`
- `cargo run -p fretboard -- diag campaign run ui-gallery-smoke --launch -- <cmd...>`
- `cargo run -p fretboard -- diag campaign run --lane smoke --tag ui-gallery --platform native --launch -- <cmd...>`
- `cargo run -p fretboard -- diag campaign share target/fret-diag/campaigns/ui-gallery-smoke/<run_id>`
- `cargo run -p fretboard -- diag campaign share target/fret-diag/campaign-batches/<selection_slug>/<run_id> --json`

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
- campaign and batch result manifests record `aggregate.share_manifest_path` plus `share_error`
  when export fails.
