# TopAppBar Scroll Diagnostics Packet v1

## Truth

- TopAppBar scroll behavior remains a Material recipe concern for the current Fret Material3
  surface.
- The promoted UI Gallery script covers pinned, enter-always, enter-always-settle,
  exit-until-collapsed, and exit-until-collapsed-settle states.
- The script passed without requiring recipe, foundation, kit-policy, or mechanism changes.
- Matrix residual risk should no longer say scroll diagnostics are still waiting on gallery proof.

## Artifacts

- `ecosystem/fret-ui-material3/src/top_app_bar.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-top-app-bar-scroll-screenshots.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_surface_data_display_packet_v1.md`

## Wiring

The script navigates to the Material3 TopAppBar gallery page, scrolls each dedicated scroll viewport,
captures bundles/screenshots at key states, and ends with a final bundle. It uses stable gallery
`test_id` anchors instead of coordinates.

## Proof

Run id: `1779933454871`.

```powershell
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-top-app-bar-scroll-screenshots.json --dir target/fret-diag/material3-top-app-bar-scroll-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

Result: `PASS`.

Bounded evidence:

- `target/fret-diag/material3-top-app-bar-scroll-20260528/sessions/1779933189257-62812/1779933454871/ai.packet`
- `target/fret-diag/material3-top-app-bar-scroll-20260528/sessions/1779933189257-62812/share/1779933454871.zip`

`diag meta` reported 300 snapshots, 135 unique test ids, and one window. `diag query test-id` found
the expected `ui-gallery-material3-top-app-bar-*` scroll viewport and action ids.

## Residual Risk

Nested-scroll consumption and fling velocity remain out of scope until a concrete consumer proves
they are needed.
