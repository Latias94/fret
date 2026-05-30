# Material3 SearchBar Accessibility Packet v2

Date: 2026-05-29
Task: M3PV2-072

## Truth

- A standalone SearchBar exposes a `TextField` semantics node at the root `test_id`.
- Without an explicit accessible name, SearchBar still publishes Material's localized default
  search label.
- Explicit `.a11y_label(...)` remains caller-owned and overrides the Material default.
- SearchBar preserves placeholder semantics separately from the accessible name.
- When expanded, SearchBar publishes `expanded=true` plus a state description for suggestions.

## Sources

- Compose Material3 `SearchBarDefaults.InputField` sets `contentDescription` from
  `Strings.SearchBarSearch`.
- Compose Material3 `SearchBarDefaults.InputField` sets `stateDescription` from
  `Strings.SuggestionsAvailable` while the search state is expanded.
- Compose Material3 English localization maps these to `Search` and `Suggestions below`.
- Base UI Combobox input uses the headless input/list relationship pattern as a cross-check for
  input-owned accessibility, while SearchView relation wiring remains covered by the previous
  SearchView packet.

The local `repo-ref/` mirror for this checkout contains Compose Multiplatform Core and Base UI but
does not contain the MUI Material UI mirror, so this packet uses Compose as the primary source for
SearchBar semantics and Base UI only as a headless a11y cross-check.

## Layer Finding

This packet found both a core mechanism gap and a Material recipe gap:

- `fret-core` exposed role descriptions but had no portable state-description field, so Material
  recipes could not represent Compose's expanded suggestions phrase without overloading label or
  value.
- `fret-ui` text/decorative/pressable authoring surfaces therefore also had no way to write that
  state-description field.
- `fret-a11y-accesskit` already maps role descriptions and AccessKit supports state descriptions;
  the missing piece was the Fret-side contract and mapping.
- Material SearchBar made accessible labels entirely caller-owned, while Compose provides a
  Material default label even when the caller omits one.

The mechanism change remains policy-free; Material owns the localized strings and the decision to
publish the expanded suggestions state.

## Artifacts

- `docs/adr/0324-a11y-state-description-semantics-v1.md`
- `crates/fret-core/src/semantics.rs`
- `crates/fret-ui/src/widget.rs`
- `crates/fret-ui/src/element.rs`
- `crates/fret-ui/src/declarative/host_widget/semantics.rs`
- `crates/fret-a11y-accesskit/src/mapping.rs`
- `ecosystem/fret-ui-material3/src/foundation/strings.rs`
- `ecosystem/fret-ui-material3/src/search_bar.rs`
- `ecosystem/fret-ui-material3/tests/search_bar_accessibility.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test search_bar_accessibility
```

The red gate failed because standalone SearchBar without `.a11y_label(...)` published no label
instead of the Material default `Search`.

Green gates:

```powershell
cargo fmt --package fret-core --package fret-ui --package fret-a11y-accesskit --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test search_bar_accessibility
cargo nextest run -p fret-ui --lib declarative_text_input_respects_a11y_role_override_and_expanded declarative_attach_semantics_can_override_state_and_relations
cargo nextest run -p fret-a11y-accesskit --lib maps_state_description maps_role_description
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test search_view_behavior
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1 material3_headless_search_view_suite_goldens_v1
cargo check -p fret-ui --lib
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
cargo clippy -p fret-a11y-accesskit --lib --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
git diff --check
```

Note: the first `fret-ui` narrow test attempt hit a Windows linker failure with unresolved
standard-library/hashbrown/taffy symbols after earlier timed-out builds were force-stopped. Re-running
with `CARGO_INCREMENTAL=0` passed, which indicates stale incremental objects rather than a code
failure.

## Residual Risk

- Diagnostics snapshot JSON does not yet export `state_description`; direct semantics snapshots and
  AccessKit mapping cover this packet.
- SearchBar focus/IME submit behavior was not changed in this packet.
- SearchView trigger-to-overlay relations remain covered by M3PV2-033 and are only regression-run
  here.
