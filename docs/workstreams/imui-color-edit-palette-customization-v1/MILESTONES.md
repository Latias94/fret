# ImUi Color Edit Palette Customization v1 Milestones

Status: Closed.

## M1 - Palette Source API

Status: Done.

Exit criteria:

- `ColorEdit` exposes an app-owned palette entry type.
- `ColorEditOptions::default()` keeps the existing 12-entry palette.
- Empty custom palettes are valid.

## M2 - Popup Wiring

Status: Done.

Exit criteria:

- `popup/swatches.rs` renders the provided palette entries.
- Palette activation preserves alpha and updates the hex draft/error state as before.
- Test IDs remain deterministic from palette order.

## M3 - Evidence and Closeout

Status: Done.

Exit criteria:

- Focused tests and source-policy anchors pass.
- Roadmap, tracker, gap audit, umbrella evidence, and catalog entries point at this lane.
- The lane has a closeout audit with exact gates.
