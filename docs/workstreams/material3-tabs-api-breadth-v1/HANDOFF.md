# Material3 Tabs API Breadth v1 - Handoff

Status: Closed
Last updated: 2026-05-30

## Current State

M3TAB-010 through M3TAB-090 are complete and the lane is closed. `Tabs` now exposes `TabsVariant`,
primary remains the default, and `.secondary()`/`.variant(TabsVariant::Secondary)` opt into
Compose-aligned secondary tab rows. Secondary tab rows use Compose-backed v30 aliases for
container/content/state tokens and a full-tab-width rectangular active indicator. Primary tab rows
keep the existing content-sized indicator behavior.

## Decisions

- Do not reopen `material3-token-visual-matrix-v1`.
- Keep one `Tabs` recipe root with a public variant enum.
- Keep active-indicator paint/motion in Material foundation and geometry in the Tabs recipe.
- Seed secondary aliases in v30 manually with Compose-backed source comments until a generated
  upstream source exists.
- Skip M3TAB-030 fixture rows for now because v30 token-resolution and `tabs_state` geometry tests
  already cover the shipped secondary API/token behavior.

## Next Recommended Action

No action remains in this lane. Start a new narrow follow-on for icon-and-label tabs, divider
rendering, gallery snippets, RTL/auto-scroll polish, or richer overflow behavior.

## Useful Gates

```powershell
python -m json.tool docs/workstreams/material3-tabs-api-breadth-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state
cargo nextest run -p fret-ui-material3 --lib tokens::v30
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```
