# Material 3 Modal Navigation Drawer Routed Content v1

Status: Complete
Owner: Codex goal `Material3 ModalNavigationDrawer routed-content hardening`
Started: 2026-06-01
Completed: 2026-06-01

This lane hardens `ModalNavigationDrawer` as a real application navigation surface, rather than a
standalone focus-trap/motion demo.

## Truth

- A modal navigation drawer destination can drive caller-owned routed content.
- Roving within the open drawer may update the selected route without closing the modal drawer.
- Activating the focused/selected destination may close the modal drawer when the caller wires that
  policy.
- Closing after destination activation must restore focus to the drawer opener.
- Reopening the drawer must show selected semantics synchronized with the caller-owned route model.

## Source Notes

- Compose Material3 samples wire `NavigationDrawerItem(onClick = { selected = item; drawerState.close() })`
  at the call site; the drawer shell exposes state, but item selection owns the route update and the
  sample explicitly closes the drawer.
- Base UI's mobile drawer example similarly keeps close/navigation composition in the app surface
  rather than in the headless drawer shell.
- Fret should therefore expose a destination activation hook and keep close-after-selection as
  caller-owned policy, not a built-in `NavigationDrawer` default.

## Artifacts

- API:
  `ecosystem/fret-ui-material3/src/navigation_drawer.rs`
  (`NavigationDrawerItem::on_select`)
- Regression test:
  `ecosystem/fret-ui-material3/tests/material3_navigation_interactions.rs`
  (`modal_navigation_drawer_drives_routed_content_and_closes_on_destination_activation`)
- Gallery repro:
  `apps/fret-ui-gallery/src/ui/snippets/material3/modal_navigation_drawer.rs`
  (`ui-gallery-material3-modal-navigation-drawer-route-panel`,
  `ui-gallery-material3-modal-navigation-drawer-route-panel-{search,settings,play}`)
- Diag script:
  `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-modal-navigation-drawer-routed-content.json`

## Wiring

- `NavigationDrawerItem::on_select` runs on explicit destination activation and is invoked even when
  the destination is already selected. This preserves the common modal drawer flow where roving can
  update the route, then Enter/click closes the drawer.
- The route model remains caller-owned. `NavigationDrawer` still owns destination chrome, selected
  semantics, roving focus, badges, state layer, ripple, and active indicator styling.
- `ModalNavigationDrawer` still owns modal overlay, scrim, focus trap/restore, and slide motion.
  No core or `fret-ui-kit` mechanism change was required.

## Proof

- The targeted test proves initial route content, modal barrier presence, roving route update without
  closing, explicit destination activation close, barrier unmount, focus restoration, and stale route
  panel removal.
- The gallery diag script drives the real gallery page, selects Settings and Play from the modal
  drawer, waits for the drawer panel to unmount, verifies route-panel replacement, focus restoration
  to the opener, and selected semantics after reopening.
- Validation:
  - `cargo fmt -p fret-ui-gallery -p fret-ui-material3`
  - `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-modal-navigation-drawer-routed-content.json`
  - `cargo test -p fret-ui-material3 --features diagnostics --test material3_navigation_interactions`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --test material3_navigation_interactions --no-deps -- -D warnings`
  - `cargo check -p fret-ui-gallery --features gallery-material3`
  - `.\target\debug\fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-modal-navigation-drawer-routed-content.json --dir target/fret-diag-material3-modal-navigation-drawer-routed-content --session-auto --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
    passed with `run_id=1780287762946`; session:
    `target/fret-diag-material3-modal-navigation-drawer-routed-content/sessions/1780287428862-242144`.

## Residual Risk

- This lane does not add a full app-router proof for ADR 0230 route hooks.
- Drawer icon/text slot mirroring under RTL remains a separate visual/layout audit.
- Touch/gesture drawer behavior remains out of scope for this desktop-first modal navigation gate.
