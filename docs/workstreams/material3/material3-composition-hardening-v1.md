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

- Search + menu and navigation + routed content are not covered by this batch.

## Batch 3: Field Overlays Inside ModalBottomSheet

Truth:

- `TextField`, `Select`, and `Autocomplete` inside a modal bottom sheet must receive a definite,
  hit-testable width from the sheet content composition.
- A nested `Select` popover inside `ModalBottomSheet` must paint above the bottom-sheet modal layer.
- Pressing `Escape` while the nested `Select` popover is open must close the `Select` first, keep
  the sheet open, and restore focus to the `Select` trigger.
- A nested `Autocomplete` listbox inside the sheet must keep input focus, close before the sheet on
  `Escape`, and leave the sheet open until the next modal dismiss.
- A final `Escape` must close the `ModalBottomSheet` and restore focus to the opener.

Artifacts:

- Regression test:
  `ecosystem/fret-ui-material3/tests/material3_overlay_interactions.rs`
  (`field_overlays_inside_modal_bottom_sheet_close_before_sheet`)
- Gallery repro:
  `apps/fret-ui-gallery/src/ui/snippets/material3/bottom_sheet.rs`
  (`ui-gallery-material3-bottom-sheet-text-field`,
  `ui-gallery-material3-bottom-sheet-select`,
  `ui-gallery-material3-bottom-sheet-autocomplete`)
- Diag script:
  `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-bottom-sheet-fields-nested-overlays.json`

Wiring:

- The BottomSheet gallery snippet now renders real `TextField`, `Select`, and `Autocomplete`
  controls inside the modal sheet.
- The sheet content wrapper owns the page/container width negotiation with
  `w_full().min_w_0()`; this follows the Material default-style ownership rule because upstream
  width negotiation is caller/container-owned, not an intrinsic field recipe default.
- No component-specific overlay policy was required: the existing `fret-ui-kit` modal/popover
  arbitration covers bottom-sheet modal layers after the gallery composition exposes valid bounds.
- Local Compose reference: `ModalBottomSheetExample.kt` contains a text field inside a modal sheet,
  while `ModalBottomSheet.kt` models the sheet as a dialog-like modal surface with scrim, dismiss,
  max-width, and content inset policy.

Proof:

- The first diag attempt exposed the composition bug: `TextField` and `Autocomplete` had zero-width
  semantics bounds inside the gallery sheet, so `click_stable` timed out.
- The diag script now asserts non-zero field bounds before interaction, types into the
  `TextField`, opens/closes nested `Select` and `Autocomplete` overlays with `Escape`, checks focus
  restoration/retention, and finally closes the sheet.
- Validation:
  - `cargo fmt -p fret-ui-gallery -p fret-ui-material3`
  - `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-bottom-sheet-fields-nested-overlays.json > $null`
  - `cargo check -p fret-ui-gallery --features gallery-material3`
  - `cargo test -p fret-ui-material3 --features diagnostics --test material3_overlay_interactions field_overlays_inside_modal_bottom_sheet_close_before_sheet -- --exact`
  - `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-bottom-sheet-fields-nested-overlays.json --dir target/fret-diag-material3-bottom-sheet-fields-nested-overlays --session-auto --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
    produced a passed script result in
    `target/fret-diag-material3-bottom-sheet-fields-nested-overlays/sessions/1780277052217-238272/script.result.json`
    with `run_id=1780277446904`; the wrapper process timed out after the pass while no cargo/fret
    process remained.

Residual risk:

- The gate covers field editing, overlay stacking/dismiss, focus, and basic hit-testing. It does not
  prove real mobile IME/window-inset behavior yet.
- Search + menu and navigation + routed content are not covered by this batch.

## Batch 4: SearchView + DropdownMenu Sibling Popovers

Truth:

- A docked `SearchView` must keep input focus after its suggestion panel opens; suggestions must not
  steal focus just because the panel mounted.
- The Search + Menu gallery composition must give `SearchView` a definite hit-testable width using
  caller-owned layout constraints.
- Typing in the open `SearchView` must update the query while suggestions remain visible.
- Clicking the sibling Material `DropdownMenu` trigger must close the `SearchView` panel and open
  the menu in the same interaction.
- The menu must expose `role=menu`, move focus to its first enabled item, and restore focus to the
  trigger on `Escape`.

Artifacts:

- Component fix:
  `ecosystem/fret-ui-material3/src/search_view.rs`
- Regression test:
  `ecosystem/fret-ui-material3/tests/material3_overlay_interactions.rs`
  (`search_view_and_dropdown_menu_arbitrate_sibling_popovers`)
- Gallery repro:
  `apps/fret-ui-gallery/src/ui/snippets/material3/menu.rs`
  (`ui-gallery-material3-menu-search`, `ui-gallery-material3-menu-search-actions`)
- Diag script:
  `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-search-menu-sibling-popovers.json`

Wiring:

- Docked `SearchView` now assigns its input element as the popover `initial_focus`, preserving
  Compose-like text input focus while non-modal suggestions are present.
- `fret-ui-kit` still owns shared overlay arbitration; `SearchView` only supplies the correct
  Material recipe policy for this popover.
- The Search + Menu gallery owns row width negotiation with an explicit `SearchView` width; no core
  or recipe default width changed.
- Material `DropdownMenu` remains menu-like and consumes initial focus once it becomes the active
  overlay.

Proof:

- Early diag runs exposed two composition bugs before the final fix: the gallery `SearchView` had
  zero-width bounds, and then the suggestion list stole focus from the input after opening.
- The targeted test now asserts input focus after suggestion open, query editing, sibling popover
  arbitration, menu first-item focus, and `Escape` focus restoration.
- The diag script asserts search bounds, input focus, query text, panel existence/size, sibling
  `SearchView` dismissal, menu role, first-item focus, and trigger restore.
- Validation:
  - `cargo fmt -p fret-ui-gallery -p fret-ui-material3`
  - `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-search-menu-sibling-popovers.json`
  - `cargo check -p fret-ui-gallery --features gallery-material3`
  - `cargo test -p fret-ui-material3 --features diagnostics --test material3_overlay_interactions search_view_and_dropdown_menu_arbitrate_sibling_popovers -- --exact`
  - `.\target\debug\fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-search-menu-sibling-popovers.json --dir target/fret-diag-material3-search-menu-sibling-popovers --session-auto --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
    passed with `run_id=1780281469011`; session:
    `target/fret-diag-material3-search-menu-sibling-popovers/sessions/1780281105569-228848`.

Residual risk:

- The gate covers docked `SearchView` + sibling `DropdownMenu`. Full-screen `SearchView` + menu is
  not covered.
- Edge-collision behavior when a `SearchView` is opened at the bottom of a viewport should get its
  own focused follow-up if it becomes a product requirement.
- Navigation routed-content composition is still pending.
