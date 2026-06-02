# Material3 Tabs Stacked Icon v1 - Handoff

Status: Closed
Last updated: 2026-05-30

## Current State

The stacked icon follow-on is implemented and closed.

## What Changed

- `TabIconPlacement::{Leading, Stacked}` is public.
- `TabItem::icon(IconId, TabIconPlacement)` is the placement-aware builder.
- `TabItem::leading_icon(...)` and `TabItem::stacked_icon(...)` are convenience APIs.
- Any stacked tab promotes the row height to the Compose 72px large height.
- Stacked tab content lays out icon above label and keeps the primary content-sized indicator tied
  to the measured icon/label span.

## Next Follow-Ons

- Exact Compose baseline placement if Fret exposes text baselines.
- Tab row divider rendering.
- RTL indicator positioning.
- Selected-tab auto-scroll.
- Gallery snippets for primary/secondary, fixed/scrollable, label-only/leading/stacked tabs.
