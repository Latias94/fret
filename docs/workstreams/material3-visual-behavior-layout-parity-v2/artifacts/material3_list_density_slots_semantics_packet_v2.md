# Material3 List Density, Slots, And Semantics Packet v2

Date: 2026-05-29
Task: M3PV2-063

## Truth

- List items expose the Material text slots needed for one-line, two-line, and three-line list
  templates: headline, overline, supporting text, and trailing supporting text.
- List item chrome resolves to Material heights: 56px for one-line, 72px for two-line, and 88px
  for overline + supporting three-line rows.
- Stable automation surfaces exist for root, chrome, headline, overline, supporting text,
  trailing supporting text, leading icon, and trailing icon parts.
- List semantics expose `List` / `ListItem` roles, selected state, disabled state, and collection
  position metadata.
- The gallery snippet renders the new multi-line item slots, so the public teaching surface uses
  the same recipe paths as the tests.

## Sources

- Compose Material3 `ListItem.kt`: `ListItem` accepts `headlineContent`, `overlineContent`,
  `supportingContent`, `leadingContent`, and `trailingContent`; its layout chooses one-line,
  two-line, or three-line types from overline/supporting presence and supporting multiline state.
- Compose Material3 `ListItemDefaults.kt`: interactive list content uses `ItemLeadingSpace`,
  `ItemTrailingSpace`, `ItemTopSpace`, `ItemBottomSpace`, and default vertical alignment.
- Compose Material3 `tokens/ListTokens.kt`: one-line, two-line, and three-line container heights
  are 56dp, 72dp, and 88dp; headline, supporting, overline, and trailing supporting text have
  distinct color and typography tokens.
- Material Web v30 token exports in Fret: generated `md.comp.list.list-item.*` metrics, colors,
  opacity, and expressive shape tokens.

MUI Material UI is not available in this worktree's `repo-ref/`; this packet used local Compose
and generated Material Web token snapshots.

## Layer Finding

This packet found a Material recipe/API completeness gap, not a core or kit mechanism gap:

- `ListItem` was still an MVP one-line API with headline plus leading/trailing icons only.
- `tokens/list.rs` already had some two-line/supporting-token accessors, but the recipe never
  selected the 72px/88px item densities or rendered secondary text slots.
- Automation only exposed root and `.chrome`, so slot layout regressions could not be targeted.
- Fret core semantics already support roles, selected/disabled flags, and collection metadata; the
  missing work was recipe-level wiring and proof.

No `crates/*` or `fret-ui-kit` mechanism change was justified.

## Artifacts

- `ecosystem/fret-ui-material3/src/list.rs`
- `ecosystem/fret-ui-material3/src/tokens/list.rs`
- `ecosystem/fret-ui-material3/tests/list_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/list.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/gallery.rs`
- `goldens/material3-headless/v1/material3-list.*.json`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gates before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test list_state
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_list_suite_goldens_v1
```

`list_state` initially failed because `ListItem` had no `supporting_text` or `overline_text`
builders. After adding the slots, the headless list suite failed because the intentional 72px/88px
multi-line layout changed the scene signature.

Green gates:

```powershell
cargo fmt --package fret-ui-material3 --package fret-ui-gallery
cargo nextest run -p fret-ui-material3 --features diagnostics --test list_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_list_suite_goldens_v1; Remove-Item Env:\FRET_UPDATE_GOLDENS
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_list_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --lib list
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
cargo check -p fret-ui-gallery
```

## Residual Risk

- List behavior remains classified as seeded: roving focus and selection-follow-focus already
  exist, but this packet did not add a dedicated keyboard-navigation behavior test.
- Drag/reorder, reveal, segmented list items, avatars, images, and video leading content remain
  outside this packet.
- Supporting text is single-line clipped in the current Fret recipe; Compose's multiline
  supporting-content heuristic can be covered later when Fret list items support wrapped secondary
  content.
