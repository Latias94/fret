# Material3 Tabs Stacked Icon v1 - Closeout Audit

Status: Closed
Date: 2026-05-30

## Result

This lane is closed. It shipped the Compose generic `Tab(icon, text)` stacked shape as a narrow
follow-on after the leading-icon lane.

## Shipped Surface

- Public `TabIconPlacement` enum with `Leading` and `Stacked`.
- `TabItem::icon(IconId, TabIconPlacement)` for future placement-aware icon API.
- `TabItem::stacked_icon(IconId)` convenience API.
- 72px row height when any tab item uses stacked icon content.
- Vertical icon-over-label layout with 24px icon sizing.
- v30 Fret runtime aliases for primary/secondary stacked row height.
- Focused `tabs_state` coverage for stacked row height, vertical content, and primary indicator
  geometry.

## Source Divergence

Material Web v30 exposes a 64px `with-icon-and-label-text.container.height` token. This lane keeps
that generated token untouched and adds a Fret runtime alias for the Compose 72px behavior. The
decision is deliberate because this component lane targets Compose toolkit layout parity.

## Gates

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state
cargo nextest run -p fret-ui-material3 --lib tokens::v30
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-tabs-stacked-icon-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
```
