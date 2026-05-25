# Parity Discovery Tools

This directory contains the first tools-level prototype for source-to-evidence parity discovery.
It is intentionally not a crate yet.

The prototype answers one question:

```text
Which upstream shadcn facts are already proven by Fret evidence, and which facts still need live
measurement before they can become diag scripts, component fixtures, or mechanism harness cases?
```

M3b adds a second evidence source: upstream shadcn web DOM snapshots. The report generator can now
read shadcn golden-style DOM JSON, address nodes through `upstream_dom_targets`, and evaluate those
nodes with the same `bounds_metric` / `bounds_metric_delta` predicate vocabulary used for Fret
layout sidecars.
For Fret sidecars, `root_metric` can also gate the captured layout root before component nodes are
compared; use this for responsive/native runs where the requested OS window size may differ from the
effective layout viewport.
If a sibling `bundle.schema2.json` is present beside a layout sidecar, the generator also reads the
semantics table and uses stable `test_id` nodes as fallback evidence when the layout sidecar misses
a selector. You can also pass bundle evidence explicitly with `--fret-bundle-schema2` and
`--fret-bundle-schema2-dir`.

Coverage manifests live in `tools/parity-discovery/manifests/`. The v2 manifest defines the next
coverage-driven sweep order for the highest-risk shadcn surfaces and keeps the existing v1 lock
rows available as regression anchors.

Each generated report also contains an `agent_packet` section. This packet is intentionally derived
from the same fixture/report data instead of introducing a second source of truth. It gives repair
agents a stable queue:

- `repair_queue`: non-passing rows with owner, layer, promotion target, source refs, Fret refs,
  test ids, evidence refs, and a next-step hint.
- `hardening_queue`: passing rows that are still medium/low confidence and should become stronger
  live measurements before broad reuse.
- `gate_queue`: rows that can be promoted into diag scripts, component fixtures, or mechanism
  harness cases once the owner/layer classification is confirmed.

The first packet-focused lane is
`docs/workstreams/component-parity-fact-harness-v1/`; the Button Group pilot artifact lives at
`docs/workstreams/component-parity-fact-harness-v1/artifacts/button_group_agent_packet_pilot_v1.json`.

Reports may also include `live_facts`. The first fact extractor reads upstream DOM snapshots and
records class tokens, computed layout values, text metrics, paint values, border/radius values, and
icon descendant bounds for `upstream.dom_target_ids`. Fret facts come from layout sidecars, bundle
schema2 semantics, and bundle schema2 `tables.text_paint` rows. Semantics/text-paint packet rows are
compacted by stable fact signatures and keep observed counts plus bounded evidence-path samples
instead of repeating every captured snapshot node. This is deliberately conservative; `tables.text_paint`
is a sparse diagnostics table, so bundle-level table presence is distinct from per-node paint/text
association.

`live_measurement_required` checks may declare `live_fact_requirements` as a map from live-fact
summary field to minimum count, for example:

```json
{
  "live_fact_requirements": {
    "fret_semantics_fact_count": 2,
    "fret_text_paint_bundle_entry_count": 1
  }
}
```

When those counts are satisfied, the row becomes `pass_known` with `observed_source:
live_fact_requirements`; otherwise it stays in the repair queue.

## Current Seeds

- Context Menu:
  - Fixture: `tools/parity-discovery/fixtures/context_menu_demo_open_parts_v1.json`
  - Reports:
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/context_menu_demo_open_mismatch_report_v2_pre_fix.json`
    and
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/context_menu_demo_open_mismatch_report_v2.json`
  - Upstream DOM evidence:
    `F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/context-menu-demo.open.json`
  - Capture script:
    `tools/diag-scripts/ui-gallery/context-menu/ui-gallery-context-menu-docs-smoke.json`
- Navigation Menu:
  - Fixture: `tools/parity-discovery/fixtures/navigation_menu_docs_demo_components_open_parts_v1.json`
  - Reports:
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/navigation_menu_docs_demo_components_open_mismatch_report_v2_pre_fix.json`
    and
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/navigation_menu_docs_demo_components_open_mismatch_report_v2.json`
  - Upstream DOM evidence:
    `F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/navigation-menu-demo.components.open.json`
  - Capture script:
    `tools/diag-scripts/ui-gallery/navigation/ui-gallery-navigation-menu-hover-switch-and-escape.json`
- Hover Card:
  - Fixture: `tools/parity-discovery/fixtures/hover_card_demo_open_parts_v1.json`
  - Reports:
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/hover_card_demo_open_mismatch_report_v2_pre_fix.json`
    and
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/hover_card_demo_open_mismatch_report_v2.json`
  - Upstream DOM evidence:
    `F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/hover-card-demo.open.json`
  - Capture script:
    `tools/diag-scripts/ui-gallery/hover-card/ui-gallery-hover-card-docs-smoke.json`
- Sheet mobile:
  - Fixture: `tools/parity-discovery/fixtures/sheet_demo_vp375x240_open_parts_v1.json`
  - Report:
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/sheet_demo_vp375x240_open_mismatch_report_v2.json`
  - Upstream DOM evidence:
    `F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/sheet-demo.vp375x240.open.json`
  - Capture script:
    `tools/diag-scripts/ui-gallery/overlay/ui-gallery-sheet-demo-vp375x240-open-layout.json`
- Tooltip:
  - Fixture: `tools/parity-discovery/fixtures/tooltip_demo_open_parts_v1.json`
  - Reports:
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/tooltip_demo_open_mismatch_report_v2_pre_fix.json`
    and
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/tooltip_demo_open_mismatch_report_v2.json`
  - Upstream DOM evidence:
    `F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/tooltip-demo.open.json`
  - Capture script:
    `tools/diag-scripts/ui-gallery/overlay/ui-gallery-tooltip-demo-open-arrow.json`
  - Harness note:
    cross-overlay/root delta predicates can request `evidence_source: "bundle_schema2_semantics"`
    when the layout sidecar has local taffy coordinates but bundle semantics has global window
    coordinates.
- Dialog:
  - Fixture: `tools/parity-discovery/fixtures/dialog_demo_open_parts_v1.json`
  - Reports:
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/dialog_demo_open_mismatch_report_v2_pre_fix.json`
    and
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/dialog_demo_open_mismatch_report_v2.json`
  - Upstream DOM evidence:
    `F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/dialog-demo.open.json`
  - Capture script:
    `tools/diag-scripts/ui-gallery/overlay/ui-gallery-dialog-docs-demo-open-screenshot.json`
  - Harness note:
    the slice locks docs-demo content/body/footer geometry while tolerating the remaining native
    text-metric delta; the pre-fix artifact proves the old FieldSet/Field composition failed by
    8-21px and the post-fix report closes it back to `pass_known`.
- Menubar:
  - Fixture: `tools/parity-discovery/fixtures/menubar_demo_open_parts_v1.json`
  - Reports:
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/menubar_demo_open_mismatch_report_v2_pre_fix.json`
    and
    `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/menubar_demo_open_mismatch_report_v2.json`
  - Upstream DOM evidence:
    `F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/menubar-demo.open.json`
  - Capture script:
    `tools/diag-scripts/ui-gallery/menubar/ui-gallery-menubar-demo-open-layout.json`
  - Harness note:
    the slice locks the `h-9` root shell, trigger vertical lane, and File menu row geometry. The
    pre-fix artifact proves auto-height root chrome and logical-pixel border layout drifted from
    upstream; the post-fix report closes the recipe lane back to `pass_known`.
- Button Group:
  - Fixture: `tools/parity-discovery/fixtures/button_group_parts_v1.json`
  - Report: `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/button_group_mismatch_report_v1.json`
  - Upstream DOM evidence:
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-input.json`
    and
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-select.json`
- Dropdown Menu:
  - Fixture: `tools/parity-discovery/fixtures/dropdown_menu_parts_v1.json`
  - Report: `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/dropdown_menu_mismatch_report_v1.json`
  - Upstream DOM evidence:
    `repo-ref/ui/apps/v4/goldens/shadcn-web/v4/new-york-v4/_tmp_extract/dropdown-menu-demo.submenu.open.json`
  - Capture script:
    `tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-m3-dropdown-menu-layout.json`
- Input:
  - Fixture: `tools/parity-discovery/fixtures/input_parts_v1.json`
  - Report: `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/input_mismatch_report_v1.json`
  - Upstream DOM evidence:
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/input-demo.json`
  - Capture script:
    `tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-m3-input-layout.json`
- Select demo open:
  - Fixture: `tools/parity-discovery/fixtures/select_demo_open_parts_v1.json`
  - Report: `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/select_demo_open_mismatch_report_v1.json`
  - Upstream DOM evidence:
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/select-demo.open.json`
  - Capture script:
    `tools/diag-scripts/ui-gallery/select/ui-gallery-select-demo-open-layout.json`
- Calendar:
  - Fixtures:
    `tools/parity-discovery/fixtures/calendar_custom_cell_size_sm_parts_v1.json`
    `tools/parity-discovery/fixtures/calendar_custom_cell_size_lg_parts_v1.json`
    `tools/parity-discovery/fixtures/calendar_responsive_mixed_parts_v1.json`
    `tools/parity-discovery/fixtures/calendar_hijri_parts_v1.json`
  - Reports:
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/calendar_custom_cell_size_sm_mismatch_report_v1.json`
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/calendar_custom_cell_size_lg_mismatch_report_v1.json`
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/calendar_responsive_mixed_mismatch_report_v1.json`
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/calendar_hijri_mismatch_report_v1.json`
  - Upstream DOM evidence:
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/calendar-20.vp375x900.json`
    `goldens/shadcn-web/v4/new-york-v4/calendar-18.json`
    `goldens/shadcn-web/v4/new-york-v4/date-picker-with-range.open.json`
    `goldens/shadcn-web/v4/new-york-v4/calendar-hijri.json`
  - Capture scripts:
    `tools/diag-scripts/ui-gallery/calendar/ui-gallery-calendar-custom-cell-size-responsive.json`
    `tools/diag-scripts/ui-gallery/calendar/ui-gallery-calendar-mixed-responsive-popover-vs-panel.json`
    `tools/diag-scripts/ui-gallery/calendar/ui-gallery-calendar-hijri-icons-and-alignment.json`
- Combobox Responsive:
  - Fixtures:
    `tools/parity-discovery/fixtures/combobox_responsive_open_parts_v1.json`
    and
    `tools/parity-discovery/fixtures/combobox_responsive_vp375x240_open_parts_v1.json`
  - Reports:
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_open_mismatch_report_v1.json`
    and
    `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_vp375x240_open_mismatch_report_v1.json`
  - Upstream DOM evidence:
    `goldens/shadcn-web/v4/new-york-v4/combobox-responsive.open.json`
    and
    `goldens/shadcn-web/v4/new-york-v4/combobox-responsive.vp375x240.open.json`
  - Capture scripts:
    `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-open.json`
    and
    `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-vp375x240-open.json`

## Commands

Generate the current v2 suite and cross-component summary:

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/shadcn_parity_discovery_v2.json --suite-output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json
```

Generate only the v2 suite summary from already generated report artifacts when archived Fret
sidecars are not present in the current worktree:

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/shadcn_parity_discovery_v2.json --suite-from-existing-reports --suite-output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json
```

Generate the v1 regression suite and cross-component summary:

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/shadcn_parity_discovery_v1.json --suite-output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/shadcn_parity_suite_report_v1.json
```

Generate individual reports when debugging one surface:

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/button_group_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-sweep-v1/button-group-select-after-select-padding/sessions/1778337694097-135816 --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-input.json --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-select.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/button_group_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/dropdown_menu_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324862209-126448 --upstream-dom-snapshot F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/goldens/shadcn-web/v4/new-york-v4/_tmp_extract/dropdown-menu-demo.submenu.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/dropdown_menu_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/input_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324505209-27984 --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/input-demo.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/input_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/select_demo_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-select-demo-open-layout-after-scroll-padding-bin/sessions/1778437252984-132992 --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/select-demo.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/select_demo_open_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/calendar_custom_cell_size_sm_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/calendar-custom-cell-size-responsive/sessions/1778438847930-90664/1778438863332-ui-gallery-calendar-custom-cell-size-responsive-sm.layout --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/calendar-20.vp375x900.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/calendar_custom_cell_size_sm_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/calendar_responsive_mixed_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/calendar-mixed-responsive-popover-vs-panel/sessions/1778449575455-175832/1778449837104-ui-gallery-calendar-mixed-responsive-panel.layout --fret-layout-sidecar-dir target/fret-diag/calendar-mixed-responsive-popover-vs-panel/sessions/1778449575455-175832/1778449837738-ui-gallery-calendar-mixed-responsive-popover.layout --upstream-dom-snapshot goldens/shadcn-web/v4/new-york-v4/date-picker-with-range.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/calendar_responsive_mixed_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/combobox_responsive_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/combobox-responsive-post-shell-sizing-desktop-final --upstream-dom-snapshot goldens/shadcn-web/v4/new-york-v4/combobox-responsive.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_open_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/combobox_responsive_vp375x240_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/combobox-responsive-post-shell-sizing-mobile-effective-vp375x240 --upstream-dom-snapshot goldens/shadcn-web/v4/new-york-v4/combobox-responsive.vp375x240.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_vp375x240_open_mismatch_report_v1.json
```

## Part Mapping Schema v1

Required top-level fields:

- `schema_version`: must be `1`.
- `component`: stable component id.
- `style`: upstream style id.
- `report`: deterministic report metadata.
- `source_refs`: upstream and Fret anchors.
- `upstream_contexts`: optional explicit upstream measurement contexts. Each context records the
  snapshot id, theme, optional mode/variant, viewport dimensions, and optional device-pixel ratio.
- `upstream_dom_targets`: optional DOM snapshot target ids used by `upstream_predicates`.
  Targets may set `context_id` to reference an `upstream_contexts[].id`; use this when a report
  contains multiple captures with the same snapshot/theme/mode/variant but different viewports or
  DPRs.
- `parts`: stable part mappings.

Suite manifests live under `tools/parity-discovery/suites/`. A suite contains stable report ids,
mapping paths, output paths, and the sidecar/DOM evidence inputs needed to regenerate all report
artifacts with one command. The generated suite report aggregates status, layer, triage, and
cross-component `top_findings`.

Required part fields:

- `id`
- `label`
- `axis`
- `upstream`
- `fret`
- `checks`

Required check fields:

- `id`
- `kind`
- `expected`
- `observed`
- `confidence`
- `evidence_refs`
- `promotion`
- `owner` (optional, but recommended for new parity-discovery rows)
- `layer` (optional; inferred from owner when omitted). Supported values are `runner`,
  `mechanism`, `policy`, `recipe`, `app_demo`, `upstream`, and `unknown`.
- `predicates` when the check is measured from Fret sidecars.
- `upstream_predicates` when the check is measured from upstream shadcn DOM snapshots.

Supported Fret-side predicate `kind` values:

- `bounds_metric`: read a metric from a node selected by stable `test_id`.
- `bounds_metric_delta`: compare the metric delta between two `test_id` nodes.
- `root_metric`: read a metric from the sidecar `meta.root_bounds` effective layout viewport.

Predicates may set `evidence_source` to `layout_sidecar`, `bundle_schema2_semantics`, or `auto`
(the default). Use `bundle_schema2_semantics` for cross-root overlay deltas that need global
window coordinates rather than local taffy-root coordinates.

Supported `kind` values:

- `existing_gate`: existing Fret evidence is enough to classify the check.
- `fret_layout_sidecar`: evaluate structured geometry predicates from one or more
  `layout.taffy.v1.json` sidecars.
- `upstream_dom_snapshot`: evaluate structured geometry predicates from one or more shadcn web DOM
  snapshot JSON files.
- `live_measurement_required`: the source fact needs live evidence. Without
  `live_fact_requirements` it remains `needs_live_measurement`; with satisfied requirements it
  becomes `pass_known`.
- `expected_mismatch`: a known mismatch imported from a prior report or failing gate.
- `blocked`: the mapping cannot run because selectors or evidence are missing.

## Report Schema v1

The generated report contains:

- `schema_version`
- `component`
- `style`
- `generated_date`
- `generated_by`
- `source_mapping`
- `upstream_contexts`
- `evidence_contexts`
- `summary`
- `parts`
- `limitations`

Report summaries also include `owner_counts`, `owner_status_counts`, `layer_counts`, and
`layer_status_counts` so mismatches can be grouped by both local owner taxonomy and the broader
runner/mechanism/policy/recipe/app-demo layers.

Reports also include derived triage metadata. Every check and part receives `triage.score`,
`triage.level`, and `triage.reasons`; the summary includes `triage_level_counts` and
`top_findings`. Scores are intentionally conservative and are derived from the evaluated status,
layer, promotion target, axis, confidence, and measured pixel gap when available. Passing evidence
always scores as `none`, while mismatches and blocked high-confidence mechanism or runner findings
rise to the top of `top_findings`.

Report statuses:

- `pass_known`
- `needs_live_measurement`
- `mismatch`
- `blocked`

The generator is deliberately conservative. It does not infer pixels from prose, and it does not
claim mismatches without failing evidence.

## Fret Sidecar Predicates

Pass sidecars directly or by directory:

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/button_group_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-sweep-v1/button-group-select-after-select-padding/sessions/1778337694097-135816 --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-input.json --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-select.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/button_group_mismatch_report_v1.json
```

Supported predicate kinds:

- `bounds_metric`: measure one target by stable `test_id`.
- `bounds_metric_delta`: measure the difference between two targets.

Supported metrics include `width`, `height`, `center_x`, and `center_y`. Fret
`layout.taffy.v1.json` sidecar `local_rect` and `abs_rect` coordinates are already window-local
logical pixels. `meta.scale_factor` is retained as diagnostics metadata for consumers that
explicitly need physical-pixel conversion; this tool must not divide sidecar coordinates by it when
comparing against upstream DOM CSS pixels.

Reports preserve `raw_bounds` as a compatibility alias for the sidecar rect values, plus
`coordinate_units` and `scale_factor`, so fractional-DPI contract questions can be audited without
rerunning diagnostics.
When the bundle-semantics fallback supplies a Fret-side predicate, the predicate row records
`evidence_source: bundle_schema2_semantics` and the `bundle_schema2_path` used for the measurement.

## Upstream DOM Snapshot Predicates

Pass snapshots directly or by directory:

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/dropdown_menu_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324862209-126448 --upstream-dom-snapshot F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/goldens/shadcn-web/v4/new-york-v4/_tmp_extract/dropdown-menu-demo.submenu.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/dropdown_menu_mismatch_report_v1.json
```

`upstream_dom_targets[]` maps stable ids to a snapshot `name`, `theme`, and DOM `path` from the
snapshot JSON. When a target sets `context_id`, the loader only accepts snapshots that match the
referenced `upstream_contexts[]` viewport and DPR metadata. `upstream_predicates[]` then uses the
same predicate shape as Fret sidecar predicates.
If a snapshot family matches more than one upstream context, every target in that family must set
`context_id` explicitly so the generator cannot silently mix evidence from different viewports.
When both Fret and upstream predicates share a metric, the report emits comparison deltas:

- `logical_delta_px`: generated Fret logical px minus upstream DOM CSS px.
- `raw_delta_px`: Fret sidecar raw rect px minus upstream DOM CSS px when raw bounds are available.

`classification_hint: diagnostics_unit_contract` should not appear for current layout sidecars when
their `coordinate_units` are `logical_px`; if it does, a reader is probably applying a stale
scale-factor conversion.

Recommended owner kinds:

- `component_recipe`
- `gallery_composition`
- `mechanism_core`
- `diagnostics_surface`
- `upstream_reference`
- `unknown`

## Overlay Shell Sizing Pattern

The responsive combobox slice is the first reusable mechanism-discovery pattern for self-drawn
overlay surfaces. Do not collapse these surfaces into one "content" check.

Map these parts separately:

- shell: `PopoverContent`, `DrawerContent`, `DropdownMenuContent`, etc.
- policy wrapper: responsive drawer wrapper, popper wrapper, command wrapper, or equivalent.
- root content: command/menu/list root.
- scroll/list body: listbox, menu viewport, or scroll area viewport.

Use source-backed upstream DOM facts for shell width/height. Use relative offsets and subtree
heights for child parts when page chrome differs between upstream web and Fret UI Gallery.

Promotion rule:

- If shell and child parts both fail, inspect recipe or gallery composition first.
- If child parts pass and only the shell fails, promote to `mechanism_core` /
  `mechanism_harness`. This usually means placement sizing, intrinsic-size hint extraction,
  viewport max-height policy, clipping, or overlay wrapper layout is wrong.

Keep pre-fix discovery sidecars and post-fix validation sidecars in different directories so report
artifacts do not mix evidence generations.

Current promoted lightweight mechanism cases live in
`ecosystem/fret-ui-shadcn/tests/fixtures/mechanism_layout_recipe_cases_v1.json`:

- `responsive-drawer-bottom-sheet-uses-eighty-vh`
- `popover-command-shell-wraps-hover-region-max-height`
