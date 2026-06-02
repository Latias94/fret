# Material3 Tabs Leading Icon v1 - Closeout Audit

Status: Closed
Date: 2026-05-30

## Result

This lane is closed. It shipped the Compose `LeadingIconTab` slice for Material3 Tabs without
reopening the broader tabs API breadth lane.

## Shipped Surface

- `TabItem::leading_icon(IconId)` adds a leading icon to an individual tab item.
- Leading-icon tabs use a horizontal 24px icon + 8px gap + label layout.
- Primary and secondary tabs route icon size and colors through typed token helpers.
- The v30 theme seeds secondary navigation-tab icon aliases from Compose token facts.
- The active-indicator layout probe now stores the real text/icon element ids, so content-sized
  primary indicators can account for icon + label content.

## Out of Scope

The stacked 72dp icon + label tab remains a separate follow-on. Compose treats that as the generic
`Tab(icon, text)` path, not `LeadingIconTab`.

## Gates

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state
cargo nextest run -p fret-ui-material3 --lib tokens::v30
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-tabs-leading-icon-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
```
