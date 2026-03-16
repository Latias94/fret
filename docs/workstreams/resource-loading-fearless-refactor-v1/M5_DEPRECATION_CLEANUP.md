# Resource Loading Fearless Refactor v1 — M5 Deprecation Cleanup

Status: In progress (first-party migration complete; compatibility surfaces still intentionally present)

## Purpose

This document is the explicit delete/rename checklist for the final M5 cleanup pass.

It exists to prevent the last stage of the resource-loading refactor from drifting back into
"leave the old name around forever" behavior.

## Current rule

Before M5:

- first-party code should already use the generic asset-reload contract,
- compatibility surfaces may remain only when they protect external callers or an existing
  shell/devloop contract,
- every remaining legacy name must have a named delete or rename decision below.

## Remaining compatibility surfaces

| Surface | Current status | Why it still exists | M5 action |
| --- | --- | --- | --- |
| `UiAssetsReloadEpoch` in `ecosystem/fret-ui-assets/src/reload.rs` | Deprecated shim | External callers may still compile against the old UI-specific type alias | Delete the alias |
| `bump_ui_assets_reload_epoch(...)` in `ecosystem/fret-ui-assets/src/reload.rs` | Deprecated shim | External callers may still compile against the old helper | Delete the function |
| `FRET_DEV_RELOAD_UI_ASSETS_TRIGGER_PATH` | Legacy env alias | Existing local scripts/shells may still export the old variable | Remove the alias after docs and tooling no longer advertise it |
| `.fret/ui_assets.touch` | Legacy trigger filename | The trigger file name is still external-shell-facing and has not yet been renamed in a coordinated pass | Replace with `.fret/asset_reload.touch` only when the first-party docs/tooling default changes together |

## Explicit decisions

### 1. Deprecated UI-specific Rust names

Decision:

- They are not part of the long-term contract.
- They should not gain any new first-party references.
- They are removed in M5, not later.

Exit check:

- `rg -n "UiAssetsReloadEpoch|bump_ui_assets_reload_epoch" apps crates ecosystem docs tools`
  returns only:
  - historical workstream notes that intentionally mention the migration, or
  - zero code references outside the deprecated shim file.

### 2. Dev-reload env var naming

Decision:

- `FRET_DEV_RELOAD_ASSET_RELOAD_TRIGGER_PATH` is the preferred generic name.
- `FRET_DEV_RELOAD_UI_ASSETS_TRIGGER_PATH` is a compatibility alias only.

Exit check:

- docs teach only the generic env var,
- code still accepts the legacy env var during the overlap window,
- M5 removes the legacy alias after the overlap window is no longer needed.

### 3. Trigger file naming

Decision:

- `.fret/ui_assets.touch` is a legacy filename, not the target naming model.
- The final target name should be `.fret/asset_reload.touch`.
- Rename only when first-party docs and any first-party tooling defaults move together.

Reason:

- The file path is shell-facing and easier for users to script against than a Rust symbol, so it
  deserves a small overlap window instead of a silent rename.

Exit check:

- the preferred path in docs/tooling is `.fret/asset_reload.touch`,
- any temporary fallback to `.fret/ui_assets.touch` is removed in M5.

## Practical M5 checklist

- [ ] Delete `UiAssetsReloadEpoch`
- [ ] Delete `bump_ui_assets_reload_epoch(...)`
- [ ] Remove `FRET_DEV_RELOAD_UI_ASSETS_TRIGGER_PATH`
- [ ] Rename the preferred trigger file from `.fret/ui_assets.touch` to `.fret/asset_reload.touch`
- [ ] Re-run the repo-wide grep and confirm no first-party code/docs teach the UI-specific names
- [ ] Update the workstream status docs to record the removal commit(s)

## Evidence anchors

- `ecosystem/fret-ui-assets/src/reload.rs`
- `ecosystem/fret-bootstrap/src/dev_reload.rs`
- `docs/workstreams/resource-loading-fearless-refactor-v1/README.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/TODO.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/hotpatch-devloop-alignment-v1/hotpatch-devloop-alignment-v1.md`
