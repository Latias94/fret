# Diagnostics campaign manifests

This folder contains repo-owned campaign manifests consumed by `fretboard diag campaign`.

Current rules:

- one file per campaign (`*.json`),
- `schema_version: 1`,
- `kind: "diag_campaign_manifest"`,
- `suites` is currently required and is the execution primitive,
- manifest entries override same-id built-in fallback definitions.

Example:

- `cargo run -p fretboard -- diag campaign list --json`
- `cargo run -p fretboard -- diag campaign show ui-gallery-smoke --json`
- `cargo run -p fretboard -- diag campaign run ui-gallery-smoke --launch -- <cmd...>`
