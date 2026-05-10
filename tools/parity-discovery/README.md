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

## Current Seeds

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

Generate the full current suite and cross-component summary:

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/shadcn_parity_discovery_v1.json --suite-output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/shadcn_parity_suite_report_v1.json
```

Generate individual reports when debugging one surface:

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/button_group_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-sweep-v1/button-group-select-after-select-padding/sessions/1778337694097-135816 --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-input.json --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-select.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/button_group_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/dropdown_menu_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324862209-126448 --upstream-dom-snapshot F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/goldens/shadcn-web/v4/new-york-v4/_tmp_extract/dropdown-menu-demo.submenu.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/dropdown_menu_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/input_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324505209-27984 --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/input-demo.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/input_mismatch_report_v1.json
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

Supported `kind` values:

- `existing_gate`: existing Fret evidence is enough to classify the check.
- `fret_layout_sidecar`: evaluate structured geometry predicates from one or more
  `layout.taffy.v1.json` sidecars.
- `upstream_dom_snapshot`: evaluate structured geometry predicates from one or more shadcn web DOM
  snapshot JSON files.
- `live_measurement_required`: the source fact is known but the prototype cannot measure it live yet.
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

## Upstream DOM Snapshot Predicates

Pass snapshots directly or by directory:

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/dropdown_menu_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324862209-126448 --upstream-dom-snapshot F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/goldens/shadcn-web/v4/new-york-v4/_tmp_extract/dropdown-menu-demo.submenu.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/dropdown_menu_mismatch_report_v1.json
```

`upstream_dom_targets[]` maps stable ids to a snapshot `name`, `theme`, and DOM `path` from the
snapshot JSON. `upstream_predicates[]` then uses the same predicate shape as Fret sidecar predicates.
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
