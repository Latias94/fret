# ImUi Color Edit History Swatches v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Editor users expect color pickers to expose recent colors near palette colors. This lane adds a
small app-owned history row to editor `ColorEdit` without introducing global color history,
automatic recording policy, or runtime state.

## Ownership

- `ColorEditOptions::history` is an app-owned recent-color source.
- `popup.rs` inserts history swatches before the palette row when history is non-empty.
- `popup/swatches.rs` shares the existing alpha-preserving swatch activation and RGB drag-source
  behavior between history and palette rows.
- Apps own when and how a color is recorded into history.

## Must-Be-True Outcomes

- Default `ColorEditOptions` have no history row.
- Apps can pass recent RGB slots with stable labels through `ColorEditOptions::history`.
- A non-empty history list counts as visible popup content even when palette presets are disabled.
- History swatches apply RGB while preserving the current alpha, matching the palette activation
  rule.
- History data remains app-owned; there is no framework-global recent-color service.

## Non-Goals

- No automatic history recording.
- No deduplication or capacity policy.
- No global color history.
- No eyedropper behavior.
- No preview/context-popup polish.
