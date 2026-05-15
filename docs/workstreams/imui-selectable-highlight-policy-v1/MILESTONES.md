# ImUi Selectable Highlight Policy Milestones

Status: closed
Last updated: 2026-05-16

Status note (2026-05-16): M0-M2 are complete; see `CLOSEOUT_AUDIT_2026-05-16.md`.

## M0 - Lane Open

Exit criteria:

- Workstream docs exist.
- Scope names only selectable forced-highlight policy and the input-picker semantic cleanup.
- Gates name the existing selectable public smoke floor.

## M1 - API And Behavior

Exit criteria:

- `SelectableOptions::highlighted` defaults to false.
- Highlighted rows use hover-style palette only when enabled and not selected.
- Selected styling remains selected even if highlighted is also set.
- Disabled highlighted rows remain muted with no hover background.
- Text picker active candidates use highlighted policy instead of selected semantics.

## M2 - Gates And Evidence

Exit criteria:

- Focused `fret-ui-kit` selectable smoke passes.
- Focused selectable palette tests pass.
- IMUI source gate and workstream catalog pass.
