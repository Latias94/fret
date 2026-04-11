# Adaptive Presentation Surface v1 — Evidence and Gates

Status: Closed lane evidence map
Last updated: 2026-04-11

## Repro / proof surfaces

- Drawer responsive pairing:
  - `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
  - `apps/fret-ui-gallery/src/ui/pages/drawer.rs`
- Sidebar app-shell boundary:
  - `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
  - `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs`
- Shared helper call-site visibility:
  - `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
- Editor-rail owner split:
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
- Container-aware panel proof:
  - `tools/diag-scripts/container-queries-docking-panel-resize.json`

## Canonical gate commands

- `cargo nextest run -p fret-ui-gallery --test device_shell_strategy_surface --test device_shell_recipe_wrapper_surface --test sidebar_docs_surface --no-fail-fast`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app drawer_responsive_dialog_keeps_desktop_dialog_on_composable_content_lane --no-fail-fast`
- `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --test editor_notes_editor_rail_surface --no-fail-fast`
- `cargo run -p fretboard -- diag run tools/diag-scripts/container-queries-docking-panel-resize.json --dir target/fret-diag/adaptive-presentation-panel-resize --session-auto --pack --include-screenshots --launch target/release/container_queries_docking_demo`
- `git diff --check`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/adaptive-presentation-surface-v1/WORKSTREAM.json > /dev/null`

## Evidence anchors

- Adaptive taxonomy:
  - `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
  - `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
- Dialog/drawer family verdict:
  - `docs/workstreams/adaptive-presentation-surface-v1/M1_CONTRACT_FREEZE_2026-04-11.md`
  - `docs/workstreams/adaptive-presentation-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- Shared device-shell helper closeout:
  - `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- Wrapper boundary closeout:
  - `docs/workstreams/device-shell-recipe-wrapper-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- Editor-rail owner split closeout:
  - `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- Outer-shell mobile downgrade closeout:
  - `docs/workstreams/outer-shell-editor-rail-mobile-downgrade-v1/CLOSEOUT_AUDIT_2026-04-11.md`
