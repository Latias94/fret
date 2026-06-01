# Material 3 Composition Hardening v1

Status: Active
Owner: Codex goal `Material3 cross-component composition hardening`
Started: 2026-06-01

This lane hardens Material 3 components in real compositions instead of auditing only isolated
recipes. The target bar is shadcn-like completeness in application flows: stable automation anchors,
predictable focus restoration, overlay/dismiss arbitration, semantics, RTL/layout-direction behavior,
motion, and gallery/diag evidence.

## Scope

- Field overlays inside modal surfaces:
  - `Select` / `Autocomplete` inside `Dialog`
  - `TextField` / `Autocomplete` inside `ModalBottomSheet`
- Search + menu composition:
  - `SearchBar` / `SearchView` next to Material `DropdownMenu`
  - overlay focus and outside-dismiss interactions
- Navigation + routed content:
  - `NavigationBar` / `NavigationRail` / `NavigationDrawer` driving visible content regions
  - focus, selected state, and route/content automation anchors

## Batch 1: Select Inside Dialog

Truth:

- A nested `Select` popover inside a modal `Dialog` must paint above the dialog layer.
- Pressing `Escape` while the nested popover is open must close the `Select` first and keep the
  `Dialog` open.
- Focus must restore to the `Select` trigger inside the `Dialog`, not to a stale popover option or
  the dialog root.
- A second `Escape` must then close the `Dialog` and restore the dialog trigger.

Artifacts:

- Policy fix: `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- Regression test:
  `ecosystem/fret-ui-material3/tests/material3_overlay_interactions.rs`
  (`select_inside_dialog_closes_inner_popover_before_modal_dialog`)
- Gallery repro:
  `apps/fret-ui-gallery/src/ui/snippets/material3/dialog.rs`
  (`ui-gallery-material3-dialog-select`)
- Diag script:
  `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-select-nested-overlay.json`

Wiring:

- The existing Dialog gallery page already renders `Select` instances inside the modal content.
- The fix stays in `fret-ui-kit` window overlay policy: modal focus containment now waits for a
  still-visible closing popover whose restore target is inside an open modal.
- Material recipes do not carry bespoke nested-overlay code.

Proof:

- Targeted test first failed with focus restored to a stale/non-semantic node, then passed after the
  kit policy fix.
- The diag script opens the gallery Dialog, opens the nested Select, verifies the listbox, closes it
  with `Escape`, checks focus on the Select trigger while the Dialog remains open, then closes the
  Dialog with a second `Escape`.
- Validation:
  - `cargo test -p fret-ui-material3 --features diagnostics --test material3_overlay_interactions`
  - `cargo test -p fret-ui-kit --lib window_overlays::tests::dismissible_popover -- --nocapture`
  - `cargo test -p fret-ui-kit --lib window_overlays::tests::modal -- --nocapture`
  - `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-select-nested-overlay.json --dir target/fret-diag-material3-dialog-select-nested-overlay --session-auto --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`

Residual risk:

- Field overlays inside `ModalBottomSheet` should get equivalent composition gates.
- Search + menu and navigation + routed content are not covered by this batch.

## Batch 2: Autocomplete Inside Dialog

Truth:

- A nested `Autocomplete` listbox inside a modal `Dialog` must paint above the dialog layer.
- The combobox input keeps focus while the listbox is open; active option state is exposed through
  active-descendant rather than roving focus.
- Pressing `Escape` while the nested listbox is open must close the `Autocomplete` first and keep the
  `Dialog` open.
- A second `Escape` must then close the `Dialog` and restore the dialog trigger.

Artifacts:

- Regression test:
  `ecosystem/fret-ui-material3/tests/material3_overlay_interactions.rs`
  (`autocomplete_inside_dialog_escape_closes_inner_popover_before_modal_dialog`)
- Gallery repro:
  `apps/fret-ui-gallery/src/ui/snippets/material3/autocomplete.rs`
  (`ui-gallery-material3-autocomplete-dialog`)
- Diag script:
  `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-dialog-nested-overlay.json`

Wiring:

- The existing Autocomplete gallery page already contains a Dialog probe with top and bottom-edge
  Autocomplete fields.
- No component-specific policy fix was required: the Batch 1 overlay controller changes also cover
  combobox-style nested popovers where focus stays on the input.
- Material recipes continue to use the shared `fret-ui-kit` overlay arbitration and Material-owned
  combobox behavior.

Proof:

- The new targeted test passed without extra component or kit changes, proving the Batch 1 modal /
  popover focus arbitration is reusable across `Select` and `Autocomplete`.
- The diag script opens the gallery Autocomplete Dialog, filters the nested listbox, checks focus on
  the combobox input, closes the listbox with `Escape`, checks the Dialog remains open, then closes
  the Dialog with a second `Escape`.
- Validation:
  - `cargo test -p fret-ui-material3 --features diagnostics --test material3_overlay_interactions autocomplete_inside_dialog_escape_closes_inner_popover_before_modal_dialog -- --exact`
  - `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-dialog-nested-overlay.json --dir target/fret-diag-material3-autocomplete-dialog-nested-overlay --session-auto --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`

Residual risk:

- `TextField` / `Autocomplete` inside `ModalBottomSheet` still need composition gates.
- Search + menu and navigation + routed content are not covered by this batch.
