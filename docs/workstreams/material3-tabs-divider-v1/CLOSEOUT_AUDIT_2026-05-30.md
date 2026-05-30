# Material3 Tabs Divider v1 - Closeout Audit

Status: Closed
Date: 2026-05-30

## Result

This lane is closed. It shipped the default Material3 TabRow bottom divider and stable diagnostics
surface.

## Shipped Surface

- Default bottom divider layer for `Tabs`.
- Stable `m3-tabs.divider` part id.
- Tab-specific divider token routing with generic divider fallback.
- Secondary v30 divider height/color aliases.
- Focused geometry regression proving the divider sits at the row bottom and the active indicator
  shares its bottom edge.

## Residuals

The Compose `divider` slot is not public in Fret yet. That customization surface should be split
into a separate API breadth lane if needed.

## Gates

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tabs_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --lib tokens::v30
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-tabs-divider-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
```
