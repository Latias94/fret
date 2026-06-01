# Material 3 Tabs Presence and Overflow Proof v1

Status: Complete
Last updated: 2026-06-01

This note records the bounded parity proof for `ecosystem/fret-ui-material3` Tabs presence work.
It is intentionally recipe-scoped: `fret-ui` owns the mounted/present/interactive mechanism,
`fret-ui-kit` owns the Radix-aligned primitive, and Material 3 owns the public component policy.

## Truth

- Inactive tab panels are unmounted by default.
- `TabPanel::force_mount(true)` keeps an inactive panel subtree mounted while making it not present
  or interactive through the existing `interactivity_gate` mechanism.
- Only the active tab panel is exported in semantics, labelled by the selected tab, and controlled by
  the selected tab.
- The root-derived `.panel` test id is only assigned to the active default panel to avoid duplicate
  ids across force-mounted panels. Force-mounted multi-panel tests use explicit panel ids.
- Scrollable Tabs already have Material edge-padding, minimum-width, and active-indicator gates.
  No new scroll core mechanism is introduced without a concrete failing selected-visibility case.

## Artifacts

- `ecosystem/fret-ui-material3/src/tabs.rs`
- `ecosystem/fret-ui-material3/tests/tabs_state.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/tabs.rs`
- `docs/workstreams/material3/material3-shadcn-level-completeness-v1.md`

## Wiring

- Material 3 exposes `TabPanel::force_mount(...)` on the recipe API.
- Rendering delegates mounted/present behavior to `fret-ui-kit::primitives::tabs`, keeping policy out
  of `crates/fret-ui`.
- The Material recipe renders every active or force-mounted panel, while default inactive panels are
  swept before they reach the tree.

## Proof

- Add a `tabs_state` regression proving force-mounted inactive panels remain live but absent from
  semantics until selected.
- Preserve the existing active tabpanel semantics/relations test for the default unmounted behavior.
- Preserve existing scrollable primary/secondary Material metric tests.

## Validation

- `cargo test -p fret-ui-material3 --features diagnostics --test tabs_state tabs_force_mounted_panels_stay_mounted_but_only_active_panel_is_semantic -- --exact`
- `cargo test -p fret-ui-material3 --features diagnostics --test tabs_state`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo check -p fret-ui-gallery`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`
- `cargo clippy -p fret-ui-material3 --features diagnostics --test tabs_state --no-deps -- -D warnings`
- `cargo clippy -p fret-ui-kit --lib --no-deps -- -D warnings`
- `python tools/check_layering.py`
- `python tools/check_workstream_catalog.py`
- `git diff --check`

## Residual Risk

- Presence motion is not added in this batch. Material Tabs only gain the mounted-vs-present contract.
- A richer scroll affordance or programmatic selected-tab visibility API should wait for a concrete
  app failure or diagnostic gate that proves the selected tab can be obscured.
