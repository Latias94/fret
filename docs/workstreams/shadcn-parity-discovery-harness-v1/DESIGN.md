---
title: Shadcn Parity Discovery Harness v1
status: active
date: 2026-05-09
scope: shadcn parity, ui-gallery, diagnostics, mechanism-harness
---

# Shadcn Parity Discovery Harness v1

This workstream owns the proactive parity loop for shadcn-style components in Fret. The goal is not
another one-off Button Group fix. The goal is a reusable harness shape that can discover layout,
chrome, interaction, and mechanism drift before a user has to report a screenshot by hand.

The seed lane `docs/workstreams/shadcn-parity-harness-v1/README.md` already turned the reported
Button Group problems into stable selectors, render-flow assertions, and one diagnostics script.
This lane starts from that evidence and promotes it into a source-to-evidence discovery format.

## Problem

Fret is a GPU-first self-drawn UI framework, so DOM snapshots alone cannot prove parity. The harness
must compare upstream shadcn facts with Fret facts through explicit part mappings:

1. upstream source and docs-path example facts,
2. Fret recipe, UI Gallery, and diagnostics evidence,
3. upstream shadcn web DOM snapshot evidence,
4. stable part ids and selectors,
5. deterministic mismatch reports,
6. promotion rules that send each high-confidence diff to the right owner.

## Non-goals for M0-M1

- Do not add a new crate yet.
- Do not claim live browser-vs-Fret extraction exists in this slice.
- Do not fix additional Button Group component bugs unless the report exposes a confirmed mismatch.
- Do not use screenshot-only judgment as the oracle.
- Do not move shadcn policy into `crates/fret-ui`.

The M0-M1 implementation lives under `tools/parity-discovery/`. A crate becomes justified only after
the report format survives multiple components, viewport classes, and theme modes.

## Source Precedence

Use source ownership by axis:

- shadcn chrome and layout recipe truth:
  `repo-ref/ui/apps/v4/registry/new-york-v4/ui/button-group.tsx`,
  `dropdown-menu.tsx`, and `input.tsx`
- shadcn docs-path example truth:
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/button-group-input.tsx`,
  `button-group-dropdown.tsx`, `input-group-button-group.tsx`, `dropdown-menu-demo.tsx`,
  `input-demo.tsx`, `input-file.tsx`, and `input-with-button.tsx`
- Fret recipe ownership: `ecosystem/fret-ui-shadcn/src/button_group.rs`,
  `dropdown_menu.rs`, and `input.rs`
- Fret teaching and selector ownership: `apps/fret-ui-gallery/src/ui/snippets/button_group/*.rs`
- Fret deterministic evidence: `apps/fret-ui-gallery/src/driver/render_flow.rs` and
  `tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json`
- Runtime mechanism escalation: `docs/mechanism-harness-v2.md`

## Harness Shape

The discovery loop is:

```text
upstream source facts
-> upstream DOM evidence when available
-> Fret observed facts
-> part mapping
-> mismatch report
-> promotion decision
```

This is deliberately broader than a test harness. A report can say:

- `pass_known`: existing Fret evidence or measured Fret sidecar predicates prove the mapped part
  currently satisfies the parity fact.
- `needs_live_measurement`: the fact is known, but the current prototype has no live extractor or
  snapshot evidence for the specific measurement.
- `mismatch`: Fret evidence contradicts the upstream fact.
- `blocked`: the mapping cannot be evaluated because required selectors or evidence are missing.

M4c adds a suite manifest lane on top of individual reports. The manifest records the mapping,
output, Fret sidecar, and upstream DOM inputs for each component report so the whole current sweep
can be regenerated with one command and summarized through a cross-component `top_findings` queue.

## Part Mapping Schema v1

The mapping fixture is JSON and must stay human-reviewable. Required top-level fields:

- `schema_version`: integer, currently `1`.
- `component`: stable component id.
- `style`: upstream visual style id, currently `new-york-v4`.
- `report`: report metadata, including deterministic `generated_date`.
- `source_refs`: upstream and Fret source/evidence anchors.
- `upstream_dom_targets`: optional stable DOM target ids for upstream shadcn web snapshots.
- `parts`: stable mapped parts.

Each `parts[]` entry has:

- `id`: stable part id.
- `label`: human-readable label.
- `axis`: one of `layout`, `chrome`, `interaction`, `semantics`, or `teaching`.
- `upstream`: source path ids plus source facts.
- `fret`: Fret source ids, test ids, and observed/evidence facts.
- `checks`: case-id-addressable checks.

Each `checks[]` entry has:

- `id`: stable check id.
- `kind`: `existing_gate`, `fret_layout_sidecar`, `upstream_dom_snapshot`,
  `live_measurement_required`, `expected_mismatch`, or `blocked`.
- `predicates`: optional structured Fret measurement predicates, evaluated from
  `layout.taffy.v1.json` sidecars when the report command receives `--fret-layout-sidecar` or
  `--fret-layout-sidecar-dir`.
- `upstream_predicates`: optional structured upstream DOM predicates evaluated from shadcn web
  snapshot JSON when the report command receives `--upstream-dom-snapshot` or
  `--upstream-dom-snapshot-dir`.
- `expected`: the parity outcome being checked.
- `observed`: `pass`, `fail`, `missing`, or `unknown`.
- `confidence`: `high`, `medium`, or `low`.
- `owner`: optional but recommended owner classification: `component_recipe`,
  `gallery_composition`, `mechanism_core`, `diagnostics_surface`, `upstream_reference`, or
  `unknown`.
- `evidence_refs`: paths that justify the observed state.
- `promotion`: the owner to use if this check becomes a high-confidence diff.

`upstream_dom_targets[]` entries map stable ids to a shadcn web DOM snapshot `name`, `theme`, and
DOM `path`. `upstream_predicates[]` reuse the same `bounds_metric` / `bounds_metric_delta`
vocabulary as Fret sidecar predicates, but resolve target ids from upstream DOM snapshots instead of
Fret `test_id`s.

## Mismatch Report Schema v1

The report is generated from the mapping fixture and must be deterministic. Required top-level
fields:

- `schema_version`: integer, currently `1`.
- `component`, `style`, and `generated_date`.
- `generated_by`: tool path.
- `source_mapping`: fixture path.
- `summary`: counts by status, owner, owner/status, and promotion target.
- `parts`: report rows with `id`, `axis`, `status`, `confidence`, `checks`, and `promotion`.
- `triage`: derived priority metadata on each check and part. It contains `score`, `level`, and
  `reasons`, with passing rows scoring as `none`.
- `summary.top_findings`: the highest-priority non-passing checks sorted by derived triage score.
- `limitations`: explicit statements about what the prototype does not measure yet.

The report must not fabricate mismatches. If the prototype cannot measure a fact, it must say
`needs_live_measurement`.

## Fret Layout Sidecar Measurement

M2 adds Fret-side live measurement without adding a crate. The generator can read one or more
`layout.taffy.v1.json` files, index nodes by stable `test_id`, treat sidecar `local_rect` and
`abs_rect` values as window-local logical pixels, and evaluate structured predicates:

- `bounds_metric`: check one target metric such as `width`, `height`, or `center_y`.
- `bounds_metric_delta`: check a metric difference between two test ids.

Supported comparisons are `between`, `gte`, `lte`, and `eq`, each with `eps_px`.

M3 also preserves `raw_bounds`, `coordinate_units`, and `scale_factor` in each measured predicate
result. `raw_bounds` is retained as a compatibility alias for the sidecar rect values; it is not a
device-pixel lane for current `layout.taffy.v1.json` sidecars. `scale_factor` remains useful
metadata for consumers that explicitly need physical-pixel conversion, but parity predicates compare
the logical sidecar values directly with upstream DOM CSS pixels.

## Upstream Web DOM Measurement

M3b adds upstream shadcn web DOM snapshot measurement without adding a crate. The generator accepts
`--upstream-dom-snapshot` and `--upstream-dom-snapshot-dir`, reads shadcn golden-style JSON, and
evaluates `upstream_predicates` against stable `upstream_dom_targets`.

The first DOM-backed targets are:

- Dropdown Menu open demo content:
  `repo-ref/ui/apps/v4/goldens/shadcn-web/v4/new-york-v4/_tmp_extract/dropdown-menu-demo.submenu.open.json`
  at `portalWrapper.0.0`.
- Input direct demo control:
  `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/input-demo.json` at
  `0`.

When a check has both Fret and upstream predicates for the same metric, the report emits comparison
deltas. The M3b unit audit fixed the earlier false classification for Dropdown Menu `w-56`: upstream
DOM width is `224` CSS px and the Fret sidecar `abs_rect.w` is `224` logical px, so the report should
show `logical_delta_px=0` without `classification_hint=diagnostics_unit_contract`.

## Promotion Rules

Promote high-confidence diffs by owner:

- Promote to a diagnostics script when the drift needs a running UI, viewport size, scrolling,
  overlay state, screenshot, or layout sidecar.
- Promote to a component fixture when the drift is a shadcn recipe, slot sizing, token, default
  chrome, or docs-path example composition issue.
- Promote to a mechanism harness case when the drift crosses components or points at runtime
  layout, hit-testing, focus, overlay routing, clipping, text measurement, semantics, or responsive
  query mechanisms.
- Leave as `needs_live_measurement` when the source fact is real but the current tool cannot yet
  collect Fret evidence at the right fidelity.

## Button Group M1 Slice

The first fixture maps:

- Button Group root layout facts (`flex`, `w-fit`, `items-stretch`, direct input `flex-1`).
- Input example search button sizing.
- Dropdown trigger sizing with upstream `!pl-2`.
- ButtonGroupText prefix, control, and suffix centering.

The M2 report classifies the Button Group seed parts as `pass_known` from measured Fret layout
sidecar predicates. Remaining discovery gaps are upstream web extraction and broader component
coverage, not Fret-side geometry access.

## M3 Component Coverage Slice

M3 adds two more component reports without adding a crate:

- Dropdown Menu: captures the open docs-path demo menu through
  `tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-m3-dropdown-menu-layout.json`.
  After the sidecar unit contract fix, `ui-gallery-dropdown-menu-demo-overlay-content` passes the
  logical `w-56` width predicate (`observed_px=224`, expected `>=224`).
- Input: captures the direct demo control plus file and button-group composed controls through
  `tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-m3-input-layout.json`. The
  report classifies all three measured parts as `pass_known`.

M3 also adds `DropdownMenuContent::test_id(...)` so overlay content can be addressed directly
without colliding with UI Gallery DocSection `*-content` selectors.

## M3b DOM Evidence Slice

M3b wires upstream DOM snapshots into the same report generator:

- Dropdown Menu report now combines upstream DOM and Fret sidecar evidence for the `w-56` content
  panel. The comparison now shows upstream DOM width `224`, Fret logical sidecar width `224`, and
  `logical_delta_px=0` with no diagnostics unit-contract hint.
- Input report now combines upstream DOM and Fret sidecar evidence for direct demo control height.
  The direct h-9 comparison now shows upstream DOM height `36`, Fret logical sidecar height `36`,
  and `logical_delta_px=0`.

## M4 Responsive Drawer Drilldown

The responsive combobox lane exposed an important harness boundary:

- The mobile drawer shell can legally sit flush against the window edge, so `bounds_within_window`
  is not a useful oracle for the shell itself.
- The command root and listbox can also extend beyond the shell rect in both upstream and Fret, so
  absolute window containment is the wrong comparison for this surface.
- The shell still needs its own size oracle. When the command/listbox subparts pass but the shell
  height fails, the report should promote the issue to the mechanism harness instead of blaming
  gallery composition or the Command recipe.
- The useful evidence chain is:
  1. stable shell selector exists,
  2. stable command-wrapper selector exists when the docs-path demo has an intermediate wrapper,
  3. stable command root selector exists,
  4. stable listbox selector exists,
  5. the sidecar is captured before the first strict layout gate,
  6. shell size uses source-backed dimensions, while command/listbox comparisons use relative
     offsets and heights instead of raw window containment.

Current fresh-desktop evidence:

- Fret pass: `target/fret-diag/combobox-responsive-fresh-desktop-2/sessions/1778389967499-159176`
- Fret bounds:
  `content=460,477.33334,200,160`,
  `command=460,477.33334,200,206`,
  `listbox=461.33334,514.6667,198,168`
- Upstream desktop shell/command/listbox bounds from
  `F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/combobox-responsive.open.json`
  show `popover-content=0,40,200,205.33334`, `command=0.6667,40.6667,198.6667,204`,
  and `listbox=0.6667,76.6667,198.6667,168`.

Post-fix mobile evidence:

- Fret pass: `target/fret-diag/combobox-responsive-post-shell-sizing-mobile-effective-vp375x240/sessions/1778412497299-166628`
- Fret bundle: `.../1778412501651-ui-gallery-combobox-responsive-vp375x240-open/bundle.schema2.json`
- Fret bounds:
  `root=0,0,375.33334,240`,
  `content=0,48,375.33334,192`,
  `command-wrapper=0,89.33334,375.33334,206.66666`,
  `command=0,89.33334,375.33334,206.66666`,
  `listbox=1.3333334,128,372.66666,168`
- Upstream mobile command/listbox bounds from
  `goldens/shadcn-web/v4/new-york-v4/combobox-responsive.vp375x240.open.json`
  stay on the same relative lane inside `portal.1.1.0`.
- The regenerated reports classify zero mismatches. Mobile now includes a diagnostics-surface
  `mobile_effective_viewport` guard so viewport drift is caught before component or mechanism
  geometry is compared.

## M4a Shell Sizing Root Cause Slice

The segmented responsive combobox reports found two real shell-sizing defects that a component-only
snapshot would have blurred:

- Desktop `PopoverContent` shell height was stuck at the placement fallback (`160px`) while the
  command subtree measured about `206px`. The source issue is the popover size-hint collector: it
  read `Container` and `Scroll` layout constraints but skipped `HoverRegion` and `Stack` layout
  constraints. `CommandList` exposes its list viewport through `ScrollArea`: the outer hover region
  owns the interaction surface, while the stack child carries the `max_h(168px)` constraint in this
  demo. Skipping those wrappers made the first open placement underestimate shell height even though
  the command/listbox subparts were correct. The fix also builds the content child before wrapping it
  in the Radix dialog wrapper so the placement pass can read the hint during the open frame.
- Mobile `DrawerContent` shell height used `min(80vh, viewport - 96px)`. Upstream shadcn v4 drawer
  uses `max-h-[80vh]` for top/bottom drawers; the extra edge-gap clamp is not source-backed for the
  responsive combobox bottom sheet and caused the 240px viewport shell to cap at `164px` instead of
  `192px`.
- Post-fix native validation then exposed a diagnostics-harness issue: the Windows native runner's
  requested resize height can differ from the effective layout root. The mobile report now includes
  `mobile_effective_viewport` using a `root_metric` predicate, so viewport drift is reported before
  Drawer shell geometry is compared.
- M4b promotes that issue out of the combobox workaround: diagnostics scripts can now use
  `window_inner_size_approx_equal` to assert the effective window-local layout viewport directly,
  and `set_window_inner_size` records the requested size as script evidence. The focused runner
  contract script lives at
  `tools/diag-scripts/ui-gallery/window/ui-gallery-window-inner-size-effective-vp375x240.json`.
- The parity mapping schema now has `upstream_contexts[]` for viewport/theme/DPR metadata, and
  generated reports preserve both declared mapping contexts and actual upstream DOM snapshot
  contexts.
- Reports now separate local `owner` from broader `layer` classification. This keeps existing
  owner buckets stable while making the goal-level buckets explicit: `runner`, `mechanism`,
  `policy`, `recipe`, `app_demo`, `upstream`, and `unknown`.
- Crate decision: keep `tools/parity-discovery/` as a tool for this lane. The schema now spans
  five fixtures and two viewport classes, but the next hard boundary is suite/CI promotion and
  material-style adapters, not Rust API reuse. Revisit a crate only when another design-system
  adapter needs the same parser/generator from Rust code.

The landed code changes are intentionally layer-local:

- `ecosystem/fret-ui-shadcn/src/popover.rs`: `size_hint_px(...)` now includes
  `ElementKind::HoverRegion` and `ElementKind::Stack` layout constraints, and popover placement now
  reads the content child's hint before wrapping it in the Radix dialog wrapper.
- `ecosystem/fret-ui-shadcn/src/drawer.rs`: `DrawerContent` top/bottom max-height now follows the
  upstream `80vh` lane without the additional `DRAWER_EDGE_GAP_PX` clamp.
- `ecosystem/fret-ui-shadcn/tests/fixtures/mechanism_layout_recipe_cases_v1.json`: the Drawer
  visible-lane shell cap and the Popover command shell wrapping rule are promoted into the lightweight
  recipe mechanism harness as `responsive-drawer-bottom-sheet-caps-visible-lane` and
  `popover-command-shell-wraps-hover-region-max-height`, so they no longer depend on full UI Gallery
  diagnostics for first-line regression coverage.

Fresh responsive reports now close this slice: desktop and mobile both report zero mismatches, with
mobile proving the effective `375x240` sidecar root before comparing the `192px` Drawer shell.

Reusable pattern for future overlay/drawer/popover cases:

1. Map the outer shell, immediate policy wrapper, command/root content, and scrolling/list body as
   separate parts.
2. Compare shell dimensions against source-backed upstream DOM facts.
3. Gate effective viewport/root bounds before comparing responsive or constrained-viewport parts.
4. Compare child content with relative offsets and heights when absolute page chrome differs.
5. If child parts pass but shell fails, classify the finding as `mechanism_core` /
   `mechanism_harness` instead of treating it as a recipe-size tweak.
6. Keep pre-fix discovery sidecars and post-fix validation sidecars separate so reports do not
   silently mix evidence generations.
