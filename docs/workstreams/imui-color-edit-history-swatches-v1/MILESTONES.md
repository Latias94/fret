# ImUi Color Edit History Swatches v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - App-Owned Source

- `ColorEditOptions::history` accepts app-owned recent color entries.
- Default options keep history empty.

## M2 - Popup Rendering

- Non-empty history counts as visible popup content.
- History swatches render before palette swatches.
- Swatch rendering is shared so history keeps palette click and drag behavior without a parallel
  implementation.

## M3 - Evidence and Closeout

- Focused tests lock source ownership and visibility behavior.
- Adapter and surface-policy tests anchor the public API.
- Workstream docs name the remaining gaps.
