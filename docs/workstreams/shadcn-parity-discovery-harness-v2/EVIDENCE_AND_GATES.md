---
title: Shadcn Parity Discovery Harness v2 Evidence and Gates
status: active
date: 2026-05-11
---

# Evidence and Gates

## Baseline Gates

Validate the manifest and workstream shape:

```powershell
python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json | Out-Null
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v2/WORKSTREAM.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/button_group_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/dropdown_menu_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/input_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/select_demo_open_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/calendar_custom_cell_size_sm_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/calendar_custom_cell_size_lg_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/calendar_responsive_mixed_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/calendar_hijri_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/combobox_responsive_open_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/combobox_responsive_vp375x240_open_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/context_menu_demo_open_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/navigation_menu_docs_demo_components_open_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/hover_card_demo_open_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/sheet_demo_vp375x240_open_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/tooltip_demo_open_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/dialog_demo_open_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/menubar_demo_open_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/input_otp_demo_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/fixtures/table_demo_parts_v1.json | Out-Null
python -m json.tool tools/parity-discovery/suites/shadcn_parity_discovery_v2.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-guide-demo-checkbox-only-selection.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-listlike-pointer-selection.json | Out-Null
python tools/check_workstream_catalog.py
```

## Baseline Evidence

- The completed v1 suite report remains the current known-good baseline:
  `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/shadcn_parity_suite_report_v1.json`
- The v1 evidence notes remain the reference for current lock surfaces:
  `docs/workstreams/shadcn-parity-discovery-harness-v1/EVIDENCE_AND_GATES.md`
- The v2 coverage manifest lives at:
  `tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json`
- The v2 coverage manifest enumerates every v1 suite row as a `covered_v1` regression-lock row
  before the new v2 discovery targets.

## Mechanism Invariant Gates

Checkbox "Enable notifications" exposed a `fret-ui` mechanism bug rather than a checkbox recipe
bug: removing a dirty child hidden behind a `layout_dirty_children_suppressed` parent applied the
child delta to a parent that intentionally does not aggregate child dirtiness. The fix keeps child
dirty deltas from crossing suppressed layout-dirty boundaries during invalidation walks and subtree
removal.

Evidence:

- Focused tests:
  `crates/fret-ui/src/tree/tests/subtree_layout_dirty_underflow_repair.rs`
- Runtime gate:
  `tools/diag-scripts/ui-gallery/checkbox/ui-gallery-checkbox-demo-with-title-toggle-underflow.json`
- Suite gate:
  `tools/diag-scripts/suites/diag-hardening-smoke/suite.json`
- Latest recheck artifact:
  `target/fret-diag/checkbox-demo-label-underflow-latest-recheck/sessions/1778497032417-172344/1778497041912/script.result.json`
- Latest recheck packet:
  `target/fret-diag/checkbox-demo-label-underflow-latest-recheck/sessions/1778497032417-172344/1778497041912/ai.packet/doctor.json`

```powershell
$env:CARGO_BUILD_JOBS='1'; cargo test --profile dev-fast -p fret-ui --lib subtree_layout_dirty -- --nocapture
```

```powershell
$env:FRET_UI_GALLERY_VIEW_CACHE='1'; $env:FRET_UI_GALLERY_VIEW_CACHE_SHELL='1'; $env:FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE='1'; $env:FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE_PANIC='1'; target/debug/fretboard-dev.exe diag run ui-gallery-checkbox-demo-with-title-toggle-underflow --dir target/fret-diag/checkbox-demo-label-underflow-recheck --session-auto --pack --ai-packet --launch -- target/debug/fret-ui-gallery.exe
```

## First Sweep Candidates

The first new fixture-driven sweep should start from the highest-risk uncovered targets in the
manifest. The first context-menu slice is now captured and gated, the navigation-menu
components-open slice is now captured and gated as the next discovered overlay surface, and the
hover-card open slice now proves the bundle-schema2 semantics fallback path for missing sidecar
nodes. The first mobile/responsive slice is now Sheet at `vp375x240`; it locks shell sizing, body
height, field offsets, and footer button geometry. Tooltip is now the first v2 slice that proves
predicate-level evidence-source selection for cross-overlay/root geometry deltas. Dialog now locks
the baseline modal docs demo and proves the sweep can find app-demo composition drift without a
user-provided screenshot. Menubar now locks root chrome, trigger vertical lane, and File menu rows
while proving the sweep can catch scale-factor-sensitive recipe chrome drift. Input OTP now locks
the docs-demo slot/separator geometry lane and records that this slice currently needs
`bundle_schema2_semantics` evidence because the taffy sidecar omits stable OTP test ids. Table now
locks the docs-demo total height, body row height, and row cadence. DataTable now locks the
docs-path policy-heavy suite for column visibility, row-actions overlays, checkbox-only selection,
list-like shift/meta pointer selection, and smoke coverage; this slice found no component policy
defect, but it fixed two scroll-before-click script gaps and a diagnostics runner stale
`last_bundle_dir` bug under reuse-launch suites:

```powershell
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-docs-smoke.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2-context-menu-sidecar --session-auto --timeout-ms 900000 --pack --ai-packet --include-screenshots --exit-after-run --launch -- target/debug/fret-ui-gallery.exe
```

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/context_menu_demo_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v2-context-menu-post-width-fix --upstream-dom-snapshot F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/context-menu-demo.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/context_menu_demo_open_mismatch_report_v2.json
```

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/shadcn_parity_discovery_v2.json --suite-output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json
```

```powershell
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/navigation/ui-gallery-navigation-menu-hover-switch-and-escape.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2-navigation-menu-post-column-fix --session-auto --timeout-ms 900000 --pack --ai-packet --include-screenshots --exit-after-run --launch -- target/debug/fret-ui-gallery.exe
```

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/navigation_menu_docs_demo_components_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v2-navigation-menu-post-column-fix --upstream-dom-snapshot F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/navigation-menu-demo.components.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/navigation_menu_docs_demo_components_open_mismatch_report_v2.json
```

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/hover_card_demo_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v2-hover-card-post-width-fix-2 --upstream-dom-snapshot F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/hover-card-demo.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/hover_card_demo_open_mismatch_report_v2.json
```

```powershell
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-sheet-demo-vp375x240-open-layout.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2-sheet-vp375x240-post-fix-2 --session-auto --timeout-ms 900000 --pack --ai-packet --include-screenshots --exit-after-run --launch -- target/debug/fret-ui-gallery.exe
```

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/sheet_demo_vp375x240_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v2-sheet-vp375x240-post-fix-2 --upstream-dom-snapshot F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/sheet-demo.vp375x240.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/sheet_demo_vp375x240_open_mismatch_report_v2.json
```

```powershell
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-tooltip-demo-open-arrow.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2-tooltip-open-arrow-layout --session-auto --timeout-ms 900000 --pack --ai-packet --include-screenshots --exit-after-run --launch -- target/debug/fret-ui-gallery.exe
```

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/tooltip_demo_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v2-tooltip-open-arrow-layout --upstream-dom-snapshot F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/tooltip-demo.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/tooltip_demo_open_mismatch_report_v2.json
```

```powershell
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-docs-demo-open-screenshot.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2-dialog-docs-demo-open-layout-post-fix --session-auto --timeout-ms 900000 --pack --ai-packet --include-screenshots --exit-after-run --launch -- target/debug/fret-ui-gallery.exe
```

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/dialog_demo_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v2-dialog-docs-demo-open-layout-post-fix --upstream-dom-snapshot F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/dialog-demo.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/dialog_demo_open_mismatch_report_v2.json
```

```powershell
cargo run --profile dev-fast -p fretboard -- diag run tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-demo-open-layout.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2-menubar-demo-open-layout-post-fix-3 --session-auto --timeout-ms 900000 --launch -- cargo run --profile dev-fast -p fret-ui-gallery
```

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/menubar_demo_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v2-menubar-demo-open-layout-post-fix-3 --upstream-dom-snapshot F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/menubar-demo.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/menubar_demo_open_mismatch_report_v2.json
```

```powershell
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/input/ui-gallery-input-otp-demo-layout.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2-input-otp-demo-layout --session-auto --pack --ai-packet --include-screenshots --launch -- target/debug/fret-ui-gallery.exe
```

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/input_otp_demo_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v2-input-otp-demo-layout --upstream-dom-snapshot F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/input-otp-demo.json --output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/input_otp_demo_mismatch_report_v2.json
```

```powershell
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/table/ui-gallery-table-demo-layout.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2-table-demo-layout --session-auto --pack --ai-packet --include-screenshots --launch -- target/debug/fret-ui-gallery.exe
```

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/table_demo_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v2-table-demo-layout --upstream-dom-snapshot F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/table-demo.json --output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/table_demo_mismatch_report_v2.json
```

```powershell
target/debug/fretboard-dev.exe diag suite tools/diag-scripts/suites/ui-gallery-data-table/suite.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2-data-table-policy-suite-run-dir-fix --session-auto --timeout-ms 900000 --ai-packet --reuse-launch --launch -- target/dev-fast/fret-ui-gallery.exe
```

```powershell
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/navigation/ui-gallery-navigation-menu-hover-switch-and-escape.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2 --session-auto --timeout-ms 900000 --pack --ai-packet --include-screenshots --exit-after-run --launch -- cargo run -p fret-ui-gallery
```

## First Sweep Results

- Pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/context_menu_demo_open_mismatch_report_v2_pre_fix.json`
- Post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/context_menu_demo_open_mismatch_report_v2.json`
- Navigation-menu pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/navigation_menu_docs_demo_components_open_mismatch_report_v2_pre_fix.json`
- Navigation-menu post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/navigation_menu_docs_demo_components_open_mismatch_report_v2.json`
- Hover-card pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/hover_card_demo_open_mismatch_report_v2_pre_fix.json`
- Hover-card post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/hover_card_demo_open_mismatch_report_v2.json`
- Sheet mobile post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/sheet_demo_vp375x240_open_mismatch_report_v2.json`
- Tooltip pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/tooltip_demo_open_mismatch_report_v2_pre_fix.json`
- Tooltip post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/tooltip_demo_open_mismatch_report_v2.json`
- Dialog pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/dialog_demo_open_mismatch_report_v2_pre_fix.json`
- Dialog post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/dialog_demo_open_mismatch_report_v2.json`
- Menubar pre-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/menubar_demo_open_mismatch_report_v2_pre_fix.json`
- Menubar post-fix report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/menubar_demo_open_mismatch_report_v2.json`
- Input OTP post-sweep report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/input_otp_demo_mismatch_report_v2.json`
- Table post-sweep report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/table_demo_mismatch_report_v2.json`
- DataTable policy suite summary:
  `target/fret-diag/shadcn-parity-discovery-harness-v2-data-table-policy-suite-run-dir-fix/sessions/1778496500156-72456/suite.summary.json`
- Suite report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json`
- First-sweep audit:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/FIRST_SWEEP_AUDIT_2026-05-11.md`
- Third-sweep audit:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/THIRD_SWEEP_AUDIT_2026-05-11.md`
- Fourth-sweep audit:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/FOURTH_SWEEP_AUDIT_2026-05-11.md`
- Fifth-sweep audit:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/FIFTH_SWEEP_AUDIT_2026-05-11.md`
- Sixth-sweep audit:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/SIXTH_SWEEP_AUDIT_2026-05-11.md`
- Seventh-sweep audit:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/SEVENTH_SWEEP_AUDIT_2026-05-11.md`
- Eighth-sweep audit:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/EIGHTH_SWEEP_AUDIT_2026-05-11.md`
- Ninth-sweep audit:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/NINTH_SWEEP_AUDIT_2026-05-11.md`
