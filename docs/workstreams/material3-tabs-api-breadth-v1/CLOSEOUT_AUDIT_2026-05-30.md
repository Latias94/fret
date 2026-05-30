# Material3 Tabs API Breadth v1 - Closeout Audit

Status: Closed
Date: 2026-05-30

## Result

This lane is closed. It shipped the missing Material3 secondary tabs API breadth without reopening
the closed token visual matrix.

## Shipped Surface

- `TabsVariant::Primary` remains the default.
- `TabsVariant::Secondary` is public and opt-in through `.variant(...)` or `.secondary()`.
- Primary tabs keep Compose-aligned content-sized active indicators.
- Secondary tabs use Compose-aligned full-tab-width rectangular active indicators.
- Fixed and scrollable secondary indicator geometry is covered by `tabs_state`.
- Secondary navigation-tab v30 aliases are seeded from Compose `SecondaryNavigationTabTokens` and
  `TabRowDefaults`.
- The shared Material active-indicator canvas now applies target-sized minimum bounds so scrollable
  indicator rects do not clamp to zero while parent width is still resolving.

## M3TAB-030 Decision

M3TAB-030 is skipped. The shipped secondary token aliases are already protected by
`cargo nextest run -p fret-ui-material3 --lib tokens::v30`, and the behavior difference is protected
by `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`.

Adding secondary rows to the M3TVM fixture suite would duplicate token alias evidence without
proving additional API breadth. A future token-fixture follow-on should only be opened if secondary
tabs gain independent visual matrix requirements beyond the current alias and geometry coverage.

## Residual Follow-Ons

- Icon-and-label tabs.
- Tab row divider rendering.
- Primary/secondary fixed/scrollable gallery snippets.
- RTL indicator positioning.
- Selected-tab auto-scroll and richer overflow behavior.
- Replacing manual secondary aliases if a future generated Material token source includes
  secondary navigation tabs.

## Gates

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state
cargo nextest run -p fret-ui-material3 --lib tokens::v30
cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures
python -m json.tool docs/workstreams/material3-tabs-api-breadth-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
```
