# M2 Gallery Query-Axis Proof — 2026-04-10

Status: accepted execution note
Last updated: 2026-04-10

This note records the promotion of an explicit UI Gallery teaching surface that keeps
container-driven and viewport-driven adaptive behavior separate on the `Navigation Menu` page.

## Goal

Prove that this lane now owns one reviewable Gallery proof that:

- compares viewport-driven and container-driven behavior side by side,
- keeps the query-axis copy explicit on the docs-path teaching surface,
- and survives a real pointer-driven toggle between query sources without regressing the trigger
  reopen path.

## Root cause found during promotion

The failing path was not a container-query contract bug.

The regression lived in `NavigationMenuTrigger` pointer coordination:

- after closing an item, `skipDelayDuration` leaves a short immediate-open window active,
- clicking the query-source `Switch` moved the pointer away and then back onto the trigger,
- pointer move reopened the trigger synchronously inside that window,
- and the following click in the same interaction sequence toggled the item closed again.

Applied fix:

- keep the repair in `ecosystem/fret-ui-kit` rather than pushing policy down into `crates/fret-ui`,
- detect the narrower case where pointer move transitions this trigger from closed to open,
- and suppress only the first follow-up pointer activation for that hover-opened state so normal
  click-close behavior still works.

## Commands used

```bash
cargo nextest run -p fret-ui-shadcn --test navigation_menu_query_mode_reopen --no-fail-fast
cargo nextest run -p fret-ui-gallery --test navigation_menu_docs_surface --no-fail-fast
cargo check -p fret-ui-gallery --message-format short
cargo build -p fret-ui-gallery --release
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/navigation/ui-gallery-navigation-menu-md-breakpoint-query-source-toggle.json --dir target/fret-diag/adaptive-navigation-menu-query-axis --session-auto --pack --include-screenshots --launch target/release/fret-ui-gallery
```

## Result

Promotion succeeded.

Successful run:

- session dir:
  `target/fret-diag/adaptive-navigation-menu-query-axis/sessions/1775826527322-55840`
- packed share artifact:
  `target/fret-diag/adaptive-navigation-menu-query-axis/sessions/1775826527322-55840/share/1775826529426.zip`

Passing regression gates:

- `ecosystem/fret-ui-shadcn/tests/navigation_menu_query_mode_reopen.rs`
  - proves both direct query-source changes and real `Switch` clicks still allow the same trigger
    to reopen.
- `apps/fret-ui-gallery/tests/navigation_menu_docs_surface.rs`
  - proves the docs page keeps the query-axis teaching copy and the promoted diag script aligned.
- `tools/diag-scripts/ui-gallery/navigation/ui-gallery-navigation-menu-md-breakpoint-query-source-toggle.json`
  - proves the Gallery surface under a real launched app with screenshots and bounded bundles.

Produced evidence includes:

- screenshots under
  `target/fret-diag/adaptive-navigation-menu-query-axis/sessions/1775826527322-55840/screenshots/`
- bounded bundles under
  `target/fret-diag/adaptive-navigation-menu-query-axis/sessions/1775826527322-55840/`
- top-level verdict in
  `target/fret-diag/adaptive-navigation-menu-query-axis/sessions/1775826527322-55840/script.result.json`

## Consequence for this lane

`ALC-031` is now considered complete.

M2 now owns three explicit proof classes:

1. narrow-window Gallery proof,
2. fixed-window panel-resize proof,
3. query-axis teaching proof that separates container and viewport behavior.

The next active adaptive work no longer lives in M2 proof promotion.
It returns to the M3 bounded follow-up queue, starting with `ALC-044`.
