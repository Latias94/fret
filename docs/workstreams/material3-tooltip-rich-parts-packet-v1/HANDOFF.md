# Material 3 Tooltip Rich Parts Packet v1 - Handoff

Status: Closed
Last updated: 2026-05-28

## Current State

The lane is closed. `RichTooltip` now exposes stable title/supporting-text selectors, Plain/Rich
root/chrome wiring is shared, and the current click-through tooltip overlay boundary remains
explicitly gated.

## Next Task

Return to the broader Material3 goal and pick the next narrow follow-on, likely picker
accessibility depth or a separate ADR-backed rich-tooltip action interactivity lane if a real
consumer needs it.

## Boundary Reminder

Do not make rich tooltip content interactive in this lane. Current tooltip overlays are pointer
transparent; changing that needs a separate mechanism/ADR follow-on.
