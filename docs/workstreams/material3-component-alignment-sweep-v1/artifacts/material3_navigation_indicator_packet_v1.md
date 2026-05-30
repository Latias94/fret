# Material 3 Navigation Indicator Packet v1

Date: 2026-05-27
Status: Pass-known for Fret-side packet scope

## Scope

Components:

- `Tabs`
- `NavigationBar`
- `NavigationRail`

Packet question:

Should active-indicator geometry and motion stay in each recipe, or move to a shared Material
foundation?

Decision:

- Recipe-owned: component-specific target geometry, fallback placement, item layout, labels, badges,
  semantics, and token selection.
- Material foundation-owned: `ActiveIndicatorRect`, spring-driven target animation, absolute canvas
  paint, bounds clamping, and optional active-indicator decoration/test id.
- Diagnostics-owned: fixed-timestep gallery scripts that prove the active-indicator selectors and
  pixel-change path are runnable.
- No `fret-ui-kit` policy or `crates/*` mechanism defect was found.

## Source Alignment

Primary references:

- Compose Material3 `TabRow.kt`: tab row computes tab positions and provides indicator offset/width
  animation with Material motion scheme values.
- Compose Material3 `NavigationBar.kt`: navigation item layout derives indicator size/position from
  the icon placeable plus token padding and keeps indicator/ripple layout ids separate.
- Compose Material3 `NavigationRail.kt`: rail indicator placement is tied to selected icon slot and
  label visibility mode.
- Material Web `tabs/internal/tab.ts` and `_tab.scss`: each tab owns an `.indicator` element with
  active-state opacity, transform animation from the previous indicator, and token-driven height,
  shape, and color.
- Material Web `labs/navigationtab/internal/navigation-tab.ts` and `_navigation-tab.scss`: nav tab
  owns an active-indicator element with token-driven width, height, color, shape, opacity, and
  selected-state styling.
- MUI `Tabs.js`: tabs indicator state is measured from the selected tab relative to the tabs
  container and updated only when start or size changes enough to matter.
- Base UI `TabsIndicator.tsx`: headless tabs expose active tab position and size separately from
  styling and hide the indicator until layout is settled.

Implication:

The upstream split does not support one shared geometry function across tabs, bottom navigation, and
rail. It does support a shared "animate/paint this target rect" primitive after each recipe computes
its target.

## Fret Implementation

New shared helper:

- `ecosystem/fret-ui-material3/src/foundation/active_indicator.rs`

Consumers:

- `ecosystem/fret-ui-material3/src/tabs.rs`
- `ecosystem/fret-ui-material3/src/navigation_bar.rs`
- `ecosystem/fret-ui-material3/src/navigation_rail.rs`

Selector helper from M3CAS-030:

- `ecosystem/fret-ui-material3/src/foundation/test_id.rs`

Packet/report artifacts:

- `tools/parity-discovery/fixtures/material3_navigation_indicator_adapter_v1.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_navigation_indicator_adapter_report_v1.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Diagnostics Evidence

Fixed timestep: `FRET_DIAG_FIXED_FRAME_DELTA_MS=16`

- Tabs:
  - Script: `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-tabs-indicator-pixels-changed-fixed-frame-delta.json`
  - AI packet: `target/fret-diag/material3-tabs-indicator-m3cas040/sessions/1779877622517-107360/1779877907632/ai.packet`
  - Zip: `target/fret-diag/material3-tabs-indicator-m3cas040/sessions/1779877622517-107360/share/1779877907632.zip`
- NavigationBar:
  - Script: `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-bar-indicator-pixels-changed-fixed-frame-delta.json`
  - AI packet: `target/fret-diag/material3-navigation-bar-indicator-m3cas040/sessions/1779877944694-50432/1779878228527/ai.packet`
  - Zip: `target/fret-diag/material3-navigation-bar-indicator-m3cas040/sessions/1779877944694-50432/share/1779878228527.zip`
- NavigationRail:
  - Script: `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-rail-indicator-pixels-changed-fixed-frame-delta.json`
  - AI packet: `target/fret-diag/material3-navigation-rail-indicator-m3cas040/sessions/1779878266615-113520/1779878588068/ai.packet`
  - Zip: `target/fret-diag/material3-navigation-rail-indicator-m3cas040/sessions/1779878266615-113520/share/1779878588068.zip`

The generated adapter report includes 25 Fret bundle-schema2 files from these runs and reports 4
pass-known parts with 0 top findings.

## Gates

Rust and packet gates:

```powershell
cargo test -p fret-ui-material3 --lib --no-run
cargo test -p fret-ui-material3 --lib active_indicator
cargo test -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/material3_navigation_indicator_adapter_v1.json --output docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_navigation_indicator_adapter_report_v1.json --fret-bundle-schema2-dir target/fret-diag/material3-tabs-indicator-m3cas040 --fret-bundle-schema2-dir target/fret-diag/material3-navigation-bar-indicator-m3cas040 --fret-bundle-schema2-dir target/fret-diag/material3-navigation-rail-indicator-m3cas040
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json
```

Diagnostics:

```powershell
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-tabs-indicator-pixels-changed-fixed-frame-delta.json --env FRET_DIAG_FIXED_FRAME_DELTA_MS=16 --dir target/fret-diag/material3-tabs-indicator-m3cas040 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-bar-indicator-pixels-changed-fixed-frame-delta.json --env FRET_DIAG_FIXED_FRAME_DELTA_MS=16 --dir target/fret-diag/material3-navigation-bar-indicator-m3cas040 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-rail-indicator-pixels-changed-fixed-frame-delta.json --env FRET_DIAG_FIXED_FRAME_DELTA_MS=16 --dir target/fret-diag/material3-navigation-rail-indicator-m3cas040 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

## Residual Risk

- The packet proves Fret-side selector, motion, and bundle evidence; it does not pixel-match against
  upstream Material Web screenshots.
- NavigationDrawer, modal drawer, and TopAppBar remain queued in wave 1 because their behavior is
  adjacent navigation, not the shared active-indicator primitive covered here.
- The parity discovery fixture schema still labels this shared Material helper as recipe-layer
  because the current parity tool has no separate `material_foundation` layer enum.
