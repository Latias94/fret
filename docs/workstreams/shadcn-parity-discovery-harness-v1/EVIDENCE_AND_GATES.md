---
title: Shadcn Parity Discovery Harness v1 Evidence and Gates
status: active
date: 2026-05-09
---

# Evidence and Gates

## Primary Gate

Generate deterministic per-component reports from measured Fret layout sidecars:

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/shadcn_parity_discovery_v1.json --suite-output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/shadcn_parity_suite_report_v1.json
```

Individual report commands are kept for targeted debugging:

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/button_group_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-sweep-v1/button-group-select-after-select-padding/sessions/1778337694097-135816 --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-input.json --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-select.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/button_group_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/dropdown_menu_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324862209-126448 --upstream-dom-snapshot F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/goldens/shadcn-web/v4/new-york-v4/_tmp_extract/dropdown-menu-demo.submenu.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/dropdown_menu_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/input_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324505209-27984 --upstream-dom-snapshot docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/input-demo.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/input_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/combobox_responsive_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/combobox-responsive-post-shell-sizing-desktop-final --upstream-dom-snapshot goldens/shadcn-web/v4/new-york-v4/combobox-responsive.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_open_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/combobox_responsive_vp375x240_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/combobox-responsive-post-shell-sizing-mobile-effective-vp375x240 --upstream-dom-snapshot goldens/shadcn-web/v4/new-york-v4/combobox-responsive.vp375x240.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_vp375x240_open_mismatch_report_v1.json
```

## Validation Gates

```powershell
python -m json.tool tools/parity-discovery/fixtures/button_group_parts_v1.json > $null
python -m json.tool tools/parity-discovery/suites/shadcn_parity_discovery_v1.json > $null
python -m json.tool tools/parity-discovery/fixtures/dropdown_menu_parts_v1.json > $null
python -m json.tool tools/parity-discovery/fixtures/input_parts_v1.json > $null
python -m json.tool tools/parity-discovery/fixtures/combobox_responsive_open_parts_v1.json > $null
python -m json.tool tools/parity-discovery/fixtures/combobox_responsive_vp375x240_open_parts_v1.json > $null
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-input.json > $null
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/button-group-select.json > $null
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/input-demo.json > $null
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v1/WORKSTREAM.json > $null
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/button_group_mismatch_report_v1.json > $null
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/dropdown_menu_mismatch_report_v1.json > $null
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/input_mismatch_report_v1.json > $null
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_open_mismatch_report_v1.json > $null
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_vp375x240_open_mismatch_report_v1.json > $null
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/shadcn_parity_suite_report_v1.json > $null
python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py
cargo nextest run -p fret-ui --lib layout_sidecar_bounds_are_logical_px_and_not_scaled_by_scale_factor --no-fail-fast
python tools/check_workstream_catalog.py
git diff --check
```

## M3 Diagnostics Capture Gates

```powershell
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-m3-dropdown-menu-layout.json --dir target/fret-diag/shadcn-parity-discovery-harness-v1-m3 --session-auto --timeout-ms 900000 --pack --ai-packet --include-screenshots --exit-after-run --launch -- cargo run -p fret-ui-gallery
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-m3-input-layout.json --dir target/fret-diag/shadcn-parity-discovery-harness-v1-m3 --session-auto --timeout-ms 900000 --pack --ai-packet --include-screenshots --exit-after-run --launch -- cargo run -p fret-ui-gallery
```

Local evidence captured on 2026-05-09:

- Dropdown Menu PASS run: `target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324862209-126448/1778325154573`.
- Dropdown Menu layout sidecar:
  `target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324862209-126448/1778325155594-ui-gallery-shadcn-parity-m3.dropdown-menu-open.layout/layout.taffy.v1.json`.
- Input PASS run: `target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324505209-27984/1778324773104`.
- Input layout sidecars:
  `target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324505209-27984/1778324773989-ui-gallery-shadcn-parity-m3.input-demo.layout/layout.taffy.v1.json`,
  `target/fret-diag/shadcn-parity-discovery-harness-v1-m3/sessions/1778324505209-27984/1778324780501-ui-gallery-shadcn-parity-m3.input-composed-controls.layout/layout.taffy.v1.json`.
- Combobox Responsive PASS runs:
  `target/fret-diag/combobox-responsive-fresh-desktop-2/sessions/1778389967499-159176`,
  `target/fret-diag/combobox-responsive-fresh-mobile-4/sessions/1778389948227-128664`.
- Combobox Responsive layout sidecars:
  `target/fret-diag/combobox-responsive-fresh-desktop-2/sessions/1778389967499-159176/1778389971044-ui-gallery-combobox-responsive-open.layout/layout.taffy.v1.json`,
  `target/fret-diag/combobox-responsive-fresh-mobile-4/sessions/1778389948227-128664/1778389951975-ui-gallery-combobox-responsive-vp375x240-open.preassert.layout/layout.taffy.v1.json`,
  `target/fret-diag/combobox-responsive-fresh-mobile-4/sessions/1778389948227-128664/1778389952456-ui-gallery-combobox-responsive-vp375x240-open.layout/layout.taffy.v1.json`.

Button Group follow-up captured after the `SelectTrigger` chrome padding fix:

- Initial discovery run: `target/fret-diag/shadcn-parity-discovery-sweep-v1/button-group-select/sessions/1778335284436-9032` with
  `button_group_mismatch_report_v1.json` reporting one `component_recipe` mismatch.
- Repro/fix validation run: `target/fret-diag/shadcn-parity-discovery-sweep-v1/button-group-select-after-select-padding/sessions/1778337694097-135816`.
- Updated report: `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/button_group_mismatch_report_v1.json` now reports
  `7 parts, 7 pass, 0 mismatch`, with the currency trigger width at `58.0` logical px against the upstream
  `58.219` px baseline.

## M3b Upstream DOM Capture Gate

Dropdown Menu open-state upstream DOM evidence reuses the checked-in local reference snapshot:

```text
F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/goldens/shadcn-web/v4/new-york-v4/_tmp_extract/dropdown-menu-demo.submenu.open.json
```

Input direct-demo upstream DOM evidence was captured from the shadcn v4 production view and checked
in as `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom/input-demo.json`.
Run the server command from
`F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4`:

```powershell
node node_modules/next/dist/bin/next start -p 4020
F:/SourceCodes/Rust/fret/repo-ref/ui/node_modules/.bin/tsx.CMD --tsconfig F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/tsconfig.scripts.json F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/scripts/extract-golden.mts input-demo --baseUrl=http://localhost:4020 --style=new-york-v4 --themes=light --outDir=F:/SourceCodes/Rust/fret-worktrees/improve-shadcn/docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/upstream-dom --update --timeoutMs=120000
```

## M4 Responsive Capture Gate

Responsive combobox upstream DOM evidence reuses the checked-in local goldens:

```text
F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/combobox-responsive.open.json
F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/combobox-responsive.vp375x240.open.json
```

Local evidence captured on 2026-05-10:

- Desktop responsive PASS run:
  `target/fret-diag/combobox-responsive-fresh-desktop-2/sessions/1778389967499-159176`.
- Mobile responsive PASS run:
  `target/fret-diag/combobox-responsive-fresh-mobile-4/sessions/1778389948227-128664`.
- Desktop report:
  `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_open_mismatch_report_v1.json`.
- Mobile report:
  `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_vp375x240_open_mismatch_report_v1.json`.

Segmented report outcomes:

- Desktop: 4 parts, 3 pass, 1 mismatch. `desktop_popover_shell_surface` is the only mismatch:
  upstream PopoverContent is about `205.333px` tall while the Fret shell is `160px`; the command
  root and listbox pass.
- Mobile: 5 parts, 4 pass, 1 mismatch. `mobile_drawer_shell_surface` is the only mismatch:
  upstream DrawerContent is `192px` tall while the Fret shell is `164px`; the command wrapper,
  command root, and listbox pass.
- Both mismatches are classified as `mechanism_core` with `mechanism_harness` promotion targets,
  not `gallery_composition`.

M4a shell-sizing fix evidence on 2026-05-10:

- Code changes:
  `ecosystem/fret-ui-shadcn/src/popover.rs` and `ecosystem/fret-ui-shadcn/src/drawer.rs`.
- Popover root cause:
  the first fix needed both the source-backed hint extraction and the overlay frame model. The
  content child must be built before wrapping it in the Radix dialog wrapper so the placement pass
  can read its layout hints, and `size_hint_px(...)` must scan both `HoverRegion` and `Stack`
  layout constraints because `CommandList` exposes its `max_h(...)` through the ScrollArea stack
  inside a hover-region root.
- Drawer root cause:
  upstream `repo-ref/ui/apps/v4/registry/new-york-v4/ui/drawer.tsx` uses `max-h-[80vh]`; the former
  `viewport - 96px` clamp was an unsupported Fret policy.
- Static gate passed:

```powershell
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo check -p fret-ui-shadcn --lib -j 1
```

- Lightweight mechanism harness gate passed:

```powershell
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo check -p fret-ui-shadcn --test web_vs_fret_layout -j 1
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo test -p fret-ui-shadcn --test web_vs_fret_layout mechanism_harness::mechanism_harness_recipe_layout_cases_match_oracles -- --exact --nocapture
```

This runs `ecosystem/fret-ui-shadcn/tests/fixtures/mechanism_layout_recipe_cases_v1.json`, including
the new `responsive-drawer-bottom-sheet-uses-eighty-vh` and
`popover-command-shell-wraps-hover-region-max-height` cases. It proves both shell-sizing rules
without launching UI Gallery.

- Targeted root-cause unit gates passed:

```powershell
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo test -p fret-ui-shadcn --lib popover::tests::popover_size_hint_reads -- --nocapture
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo test -p fret-ui-shadcn --lib drawer::tests::drawer_content_max_height_fraction_clamps_tall_content -- --exact --nocapture
```

- Formatting gate passed:

```powershell
rustfmt --edition 2024 ecosystem/fret-ui-shadcn/src/drawer.rs ecosystem/fret-ui-shadcn/src/popover.rs
rustfmt --edition 2024 ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/mechanism_harness.rs
python -m json.tool ecosystem/fret-ui-shadcn/tests/fixtures/mechanism_layout_recipe_cases_v1.json > $null
git diff --check
```

- Heavy gates attempted but not completed on this machine:
  - `cargo nextest list -p fret-ui-shadcn -E 'test(drawer_content_max_height_fraction_clamps_tall_content) | test(popover_size_hint_reads_hover_region_max_height)'`
    failed with Windows page-file error `1455`.
  - `cargo test -p fret-ui-shadcn popover_size_hint_reads_hover_region_max_height -- --exact --nocapture`
    compiled integration tests and hit rustc OOM/page-file failures.
  - `cargo build -p fret-ui-gallery --bin fret-ui-gallery -j 1` did not complete within the local
    timeout, so no post-fix UI Gallery sidecars were captured in this slice.

The post-fix responsive combobox evidence is now split by effective viewport before component
geometry. The native Windows runner currently needs a `375x220` resize request to produce the
effective `375x240` layout root used by the upstream mobile DOM golden; the mobile fixture gates
that root first with `mobile_effective_viewport`.

M4b runner viewport-contract follow-up:

- Protocol/mechanism:
  `UiPredicateV1::WindowInnerSizeApproxEqual` gates the diagnostics runtime's effective
  window-local layout viewport directly, before any component `test_id` geometry is evaluated.
- Discovery schema:
  mapping fixtures now support `upstream_contexts[]` so report artifacts carry the upstream
  snapshot theme, mode/variant, viewport dimensions, and device-pixel ratio at the report top level
  instead of burying them inside individual predicate measurements.
- Discovery finding:
  regenerating reports with explicit contexts exposed a Dropdown Menu target-context drift: the
  target matched `dropdown-menu-demo` but omitted `mode=open` / `variant=submenu`, which made the
  upstream DOM evidence addressable only by accident in stale artifacts. The fixture now pins the
  same context on both `upstream_contexts[]` and `upstream_dom_targets[]`.
- Source-only cleanup:
  the current fixture corpus no longer has prose-only discovery rows. All five `*_parts_v1.json`
  fixtures express their checks as `fret_layout_sidecar`, with upstream DOM predicates attached
  where upstream snapshots are available.
- Layer taxonomy:
  generated reports now expose both local `owner` and broader `layer` fields. The layer axis maps
  findings to `runner`, `mechanism`, `policy`, `recipe`, `app_demo`, `upstream`, or `unknown`, with
  `layer_counts` / `layer_status_counts` in the report summary.
- Crate decision:
  keep `tools/parity-discovery/` as a tool for now. The harness has stable enough report semantics
  for this workstream, but no current Rust caller needs the parser/generator as a crate API.
- Script evidence:
  `tools/diag-scripts/ui-gallery/window/ui-gallery-window-inner-size-effective-vp375x240.json`
  isolates the current Windows native requested/effective contract (`375x220` request =>
  effective `375x240` viewport).
- Responsive reuse:
  `tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-vp375x240-open.json`
  now waits on `window_inner_size_approx_equal` before opening the Drawer-backed combobox.
- Runner attribution:
  `set_window_inner_size` writes a bounded `window_inner_size.requested` script event so packed
  diagnostics evidence records the request even when the effective size differs.

Post-fix gates:

```powershell
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo build -p fret-ui-gallery --bin fret-ui-gallery -j 1
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-open.json --dir target/fret-diag/combobox-responsive-post-shell-sizing-desktop-final --session-auto --timeout-ms 900000 --pack --ai-packet --include-screenshots --exit-after-run --launch -- target/debug/fret-ui-gallery.exe
target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-vp375x240-open.json --dir target/fret-diag/combobox-responsive-post-shell-sizing-mobile-effective-vp375x240 --session-auto --timeout-ms 900000 --pack --ai-packet --include-screenshots --exit-after-run --launch -- target/debug/fret-ui-gallery.exe
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/combobox_responsive_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/combobox-responsive-post-shell-sizing-desktop-final --upstream-dom-snapshot goldens/shadcn-web/v4/new-york-v4/combobox-responsive.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_open_mismatch_report_v1.json
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/combobox_responsive_vp375x240_open_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/combobox-responsive-post-shell-sizing-mobile-effective-vp375x240 --upstream-dom-snapshot goldens/shadcn-web/v4/new-york-v4/combobox-responsive.vp375x240.open.json --output docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_vp375x240_open_mismatch_report_v1.json
```

Runner viewport-contract validation:

```powershell
python -m json.tool tools/diag-scripts/ui-gallery/window/ui-gallery-window-inner-size-effective-vp375x240.json > $null
python -m json.tool tools/parity-discovery/fixtures/combobox_responsive_vp375x240_open_parts_v1.json > $null
python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo test -p fret-diag-protocol --test script_json_roundtrip script_v2_roundtrip_window_inner_size_predicate -- --exact
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo test -p fret-bootstrap --lib --features ui-app-driver,diagnostics window_inner_size_predicate_reads_effective_window_bounds -- --nocapture
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo test -p fret-ui-gallery --test combobox_diag_surface window_inner_size_contract_diag_script_is_runner_owned -- --exact
```

Post-fix report outcomes:

- Desktop: 4 parts, 4 pass, 0 mismatch.
- Mobile: 6 parts, 6 pass, 0 mismatch. The added diagnostics-surface part proves the effective
  sidecar root is `375.333 x 240` before the Drawer shell is compared to the upstream
  `max-h-[80vh]` result (`192px`).

M4c report triage scoring:

- Generated reports now include derived `triage` metadata on every check and part.
- Report summaries include `triage_level_counts` and `top_findings`, so future sweeps can sort
  non-passing rows by status, layer, promotion target, axis, confidence, and measured pixel gap.
- `tools/parity-discovery/suites/shadcn_parity_discovery_v1.json` regenerates the current five
  report artifacts and writes
  `docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/shadcn_parity_suite_report_v1.json`.
- Re-running the current five reports with post-fix evidence keeps all current component reports
  at zero mismatches. `button_group_mismatch_report_v1.json` now uses the post-fix
  ButtonGroup Select evidence directory and the checked-in ButtonGroup upstream DOM snapshots, so
  stale seed sidecars no longer create false top findings.

## Second Sweep Audit Validation

The second proactive sweep audit is recorded in
`docs/workstreams/shadcn-parity-discovery-harness-v1/SECOND_SWEEP_AUDIT_2026-05-10.md`.

It maps the goal requirements to concrete evidence and records three non-user-reported findings:

- ButtonGroup SelectTrigger chrome sizing (`component_recipe`, promoted through report, render-flow,
  and diagnostics gates).
- Popover command shell sizing (`mechanism_core`, promoted to the recipe mechanism harness).
- Drawer bottom-sheet 80vh sizing (`mechanism_core`, promoted to the recipe mechanism harness).

Validation run on 2026-05-10:

```powershell
python -m json.tool docs/workstreams/shadcn-parity-discovery-harness-v1/WORKSTREAM.json > $null
python tools/check_workstream_catalog.py
python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py
git diff --check
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo test -p fret-ui-gallery --test combobox_diag_surface -- --nocapture
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo test -p fret-ui-shadcn --test web_vs_fret_layout mechanism_harness::mechanism_harness_recipe_layout_cases_match_oracles -- --exact --nocapture
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo test -p fret-ui-shadcn --lib popover_size_hint_reads -- --nocapture
$env:CARGO_BUILD_JOBS='1'; $env:RUSTFLAGS='-C debuginfo=0 -C codegen-units=1'; cargo test -p fret-ui-shadcn --lib drawer::tests::drawer_content_max_height_fraction_clamps_tall_content -- --exact --nocapture
```

## M3 Report Outcomes

- `dropdown_menu_mismatch_report_v1.json`: 3 parts, 3 pass, 0 mismatch. The
  `demo_content_matches_w_56_logical_width` comparison reports upstream DOM width `224`, Fret
  logical sidecar width `224`, and `logical_delta_px=0` at scale factor `1.5`; there is no
  `diagnostics_unit_contract` hint.
- `input_mismatch_report_v1.json`: 3 parts, 3 pass, 0 mismatch.
  The direct Input h-9 comparison reports upstream DOM height `36`, Fret logical sidecar height
  `36`, and `logical_delta_px=0` at scale factor `1.5`.

## Existing Fret Evidence Reused by M1

- UI Gallery seed script:
  `tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json`
- Render-flow invariant:
  `apps/fret-ui-gallery/src/driver/render_flow.rs`
  (`gallery_button_group_shadcn_parity_seed_layout_invariants`)
- Button Group snippets:
  `apps/fret-ui-gallery/src/ui/snippets/button_group/input.rs`,
  `dropdown_menu.rs`, and `text.rs`
- Seed proof lane:
  `docs/workstreams/shadcn-parity-harness-v1/README.md`
- Fret layout sidecars:
  `target/fret-diag/shadcn-parity-harness-v1/**/layout.taffy.v1.json`
- M3 Dropdown Menu and Input sidecars:
  `target/fret-diag/shadcn-parity-discovery-harness-v1-m3/**/layout.taffy.v1.json`
- Responsive Combobox sidecars:
  `target/fret-diag/shadcn-parity-discovery-harness-v1-responsive-desktop/**/layout.taffy.v1.json`,
  `target/fret-diag/shadcn-parity-discovery-harness-v1-responsive-mobile/**/layout.taffy.v1.json`

## Existing Runtime/Diag Gates

The discovery report does not rerun these gates automatically in M1, but it cites them as the
current Fret evidence:

```powershell
cargo nextest run -p fret-ui-gallery gallery_button_group_shadcn_parity_seed_layout_invariants
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/shadcn-parity/ui-gallery-shadcn-parity-seed-layout.json --pack --ai-packet --launch -- cargo run -p fret-ui-gallery
```

## Promotion Gate Rules

- `diag_script`: add or update a schema v2 script under `tools/diag-scripts/` when the check needs a
  running UI, scroll, viewport, overlay state, screenshot, or layout sidecar.
- `component_fixture`: add or update a fixture or test under `ecosystem/fret-ui-shadcn/tests/` when
  the diff is shadcn recipe chrome, slot sizing, token, or docs-path composition.
- `mechanism_harness`: add or update a mechanism-level case when the diff points at `crates/fret-ui`
  layout vocabulary, hit testing, focus routing, overlay routing, text measurement, clipping,
  semantics, or responsive query mechanisms.
- `needs_live_measurement`: keep the row in the report until a live extractor can turn it into one
  of the above owners with high confidence.
