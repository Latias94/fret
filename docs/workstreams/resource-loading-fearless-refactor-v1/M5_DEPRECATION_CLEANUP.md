# Resource Loading Fearless Refactor v1 — M5 Deprecation Cleanup

Status: Completed (legacy UI-specific reload names and legacy trigger/env defaults removed)

## Purpose

This document is the explicit delete/rename checklist for the final M5 cleanup pass.

It exists to prevent the last stage of the resource-loading refactor from drifting back into
"leave the old name around forever" behavior.

## Removed surfaces

| Surface | Current status | Why it still exists | M5 action |
| --- | --- | --- | --- |
| historical `UiAssetsReloadEpoch` alias from the deleted `fret-ui-assets::reload` shim | Removed | UI-specific reload alias no longer matches the shared asset contract | Deleted |
| historical `bump_ui_assets_reload_epoch(...)` alias from the deleted `fret-ui-assets::reload` shim | Removed | UI-specific helper no longer matches the shared asset contract | Deleted |
| `FRET_DEV_RELOAD_UI_ASSETS_TRIGGER_PATH` | Removed | Generic asset-reload env naming is now the only first-party contract | Deleted |
| `.fret/ui_assets.touch` | Removed as the first-party default | Generic asset-reload trigger naming is now the only first-party contract | Replaced by `.fret/asset_reload.touch` |

## Explicit decisions

### 1. Deprecated UI-specific Rust names

Decision:

- They are not part of the long-term contract.
- They should not gain any new first-party references.
- They were removed in M5.

Exit check:

- `rg -n "UiAssetsReloadEpoch|bump_ui_assets_reload_epoch" apps crates ecosystem docs tools`
  returns only:
  - historical workstream notes that intentionally mention the migration, or
  - zero code references in first-party code.

### 2. Dev-reload env var naming

Decision:

- `FRET_DEV_RELOAD_ASSET_RELOAD_TRIGGER_PATH` is the generic first-party name.
- `FRET_DEV_RELOAD_UI_ASSETS_TRIGGER_PATH` is no longer supported.

Exit check:

- docs teach only the generic env var,
- code accepts only the generic env var.

### 3. Trigger file naming

Decision:

- `.fret/asset_reload.touch` is now the first-party trigger filename.
- `.fret/ui_assets.touch` is no longer the first-party default.

Exit check:

- docs/tooling use `.fret/asset_reload.touch`,
- first-party code no longer defaults to `.fret/ui_assets.touch`.

## Practical M5 checklist

- [x] Delete `UiAssetsReloadEpoch`
- [x] Delete `bump_ui_assets_reload_epoch(...)`
- [x] Remove `FRET_DEV_RELOAD_UI_ASSETS_TRIGGER_PATH`
- [x] Rename the preferred trigger file from `.fret/ui_assets.touch` to `.fret/asset_reload.touch`
- [x] Re-run the repo-wide grep and confirm no first-party code/docs teach the UI-specific names
- [x] Update the workstream status docs to record the removal

## Evidence anchors

- `crates/fret-runtime/src/asset_reload.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret-bootstrap/src/dev_reload.rs`
- `docs/workstreams/resource-loading-fearless-refactor-v1/README.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/TODO.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/hotpatch-devloop-alignment-v1/hotpatch-devloop-alignment-v1.md`
