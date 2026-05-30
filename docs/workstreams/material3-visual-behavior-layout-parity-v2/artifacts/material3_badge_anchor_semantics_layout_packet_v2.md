# Material3 Badge Anchor Semantics And Layout Packet v2

Date: 2026-05-29

## Truth

- Standalone Badge is a BadgedBox-style composition with stable root, anchor, and badge parts.
- The root `test_id` identifies the badged box group; `<base>.anchor` identifies the anchored
  content slot; `<base>.badge` identifies the visual badge slot.
- Author-provided badge labels belong to the badge part, not to an incidental wrapper.
- Text badges expand beyond the minimum large-badge size when content needs horizontal padding.

## Sources

- Compose Material3 `Badge.kt`: `BadgedBox` lays out explicit `anchor` and `badge` children, and
  `Badge` uses `defaultMinSize` plus horizontal padding for content badges.
- MUI Material UI `Badge.js`: `BadgeRoot` and `BadgeBadge` are separate slots, with badge content
  rendered in the badge slot.
- Fret Material3 conventions: dotted part ids via
  `ecosystem/fret-ui-material3/src/foundation/test_id.rs`.

## Layer Finding

This was a Material recipe gap, not a `crates/fret-ui` or `fret-ui-kit` mechanism gap. Fret already
had the required semantics decoration, test-id, and absolute layout mechanisms. The old Badge recipe
collapsed root/badge identity into one semantics wrapper and that wrapper also masked text-badge
intrinsic width.

## Artifacts

- `ecosystem/fret-ui-material3/src/badge.rs`
- `ecosystem/fret-ui-material3/tests/badge_semantics.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `goldens/material3-headless/v1/material3-badge.*.json`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test badge_semantics
```

Failed because `m3-badge` was still `Generic` and no root/anchor/badge part contract existed.

Green gates:

```powershell
cargo nextest run -p fret-ui-material3 --test badge_semantics
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids
$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_badge_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_badge_suite_goldens_v1
```

## Residual Risk

- Badge has no interactive behavior or motion by design, so those axes remain low risk.
- NavigationBar and NavigationRail badge integration still deserve their own navigation layout
  packet because their icon/label geometry is owned by the navigation recipes, not standalone Badge.
