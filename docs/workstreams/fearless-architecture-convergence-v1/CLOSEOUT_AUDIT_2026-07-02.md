# Fearless Architecture Convergence v1 - Closeout Audit - 2026-07-02

Status: closed
Last updated: 2026-07-02

## Objective

Close the coordinator after executing the 2026 UI framework convergence plan through the current
breakable refactor window. This audit records what landed, what remains deliberately retained, and
which follow-ons own the remaining work so future agents do not reopen the broad coordinator as an
implementation lane.

Primary plan:

- `docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md`

## Verdict

The plan is closed for the current implementation scope with explicit retained and deferred
follow-ons. The main architecture direction now has executable gates for:

- source responsibility policy,
- public app facade copyability,
- identity/dirty graph diagnostics,
- `ViewId` / boundary-owned frame products,
- runtime policy vocabulary demotion,
- retained scene chunks and guarded quad partial uploads,
- text/glyph/wasm cache budgets, and
- modular consumption profiles.

This does not mean every possible renderer or app-starter expansion is complete. The closeout
condition is satisfied because each remaining item has a named owner, reason, and gate path below.

## Unit Evidence

| Unit | Status | Evidence |
| --- | --- | --- |
| U1 - convergence contract and owner map | Closed | Commit `020bb34a37`; `docs/golden-architecture.md`; `docs/runtime-contract-matrix.md`; `docs/ui-closure-map.md`; `docs/adr/IMPLEMENTATION_ALIGNMENT.md`. |
| U2 - responsibility source-policy gates | Closed | Commit `84f60d8355`; `tools/check_surface_policy.py`; `tools/test_check_surface_policy.py`; `tools/pre_release.py`; gate `python3 tools/check_surface_policy.py`. |
| U3 - second-hour public app ladder | Closed for the first public ladder slice; broader behavioral ladder deferred | Commit `fe248df70b`; `crates/fretboard/src/scaffold/templates.rs`; `crates/fretboard/src/scaffold/mod.rs`; `docs/first-hour.md`; `docs/examples/README.md`; `docs/crate-usage-guide.md`; focused scaffold tests. |
| U4 - identity and dirty graph diagnostics | Closed for observability; stable-handle deletion deferred | Commits `890366ee74`, `df0d6620ff`; `UiDebugFrameStats` fields; `crates/fret-diag/src/perf_keys.rs`; `docs/workstreams/diag-perf-profiling-infra-v1/perf-key-registry.frame-stats.json`; focused identity/dispatch/dirty frontier tests. No `StableNodeHandle` code landed in this plan scope. |
| U5 - `ViewId` boundary ownership | Closed for staged dirty frontier and frame-product ownership | Commits `09debbceae` through `767a6b3b62`; `DirtyViewFrontier`; `BoundaryFrameProducts`; dispatch, command, hit-test, semantics, paint replay owner states; ADR 0327 alignment. The v1 boundary-node bridge remains explicit compatibility debt. |
| U6 - demote policy-coded runtime vocabulary | Closed with explicit mechanism retentions | Commits `3366af80ee` through `2729dab471`; source-policy root export checks; ADR 0066 alignment; `docs/action-hooks.md`; focused `fret-ui` and policy gate runs. Gate scope is default/root/public authoring surfaces, not a whole-repo vocabulary ban. |
| U7 - scene chunks and renderer dirty uploads | Closed for chunk bridge, diagnostics, and guarded quad partial uploads | Commits `0ae54c6a3a` through `6a45373eac`; `fret_core::SceneChunk`; `SceneChunkManifest`; renderer chunk input, payload cache, resident upload diagnostics, and quad partial write path. |
| U8 - text/glyph/wasm budgets | Closed for bounded cache/residency and web evidence | Commits `47b77fa7be` through `63dc08ce4f`; `tools/perf/diag_u8_text_budget_gate.py`; text shape cache budget, glyph atlas page budget, visible glyph residency, code-editor cache diagnostics, native and web budget evidence. |
| U9 - modular consumption and facade split | Closed for profile gates and `AppUi` split | Commits `17b1e55929` through `435df2240c`; `tools/check_consumption_profiles.py`; `ecosystem/fret/src/view.rs`; `ecosystem/fret/src/view/shell.rs`; profile checks for contracts-only, UI substrate, manual assembly, default `fret`, batteries, bootstrap, and launch. |

## Verification Contract Mapping

| Contract item | Closeout evidence |
| --- | --- |
| Formatting | `cargo fmt --all --check` passed during U8 web closeout and after U9 split work. |
| Layering | `python3 tools/check_layering.py` passed across U2-U9 slices and is part of this closeout gate set. |
| Consumption profiles | `python3 tools/check_consumption_profiles.py` passed and now covers default `fret`, `fret --features batteries`, backend-free app, bootstrap, and launch profiles. |
| Source-policy checker | `python3 tools/check_surface_policy.py` passed; unit tests prove negative default-import and root-policy-vocabulary cases. The checker protects default/root/public authoring surfaces and classified mechanism exports, not every prose or ecosystem policy occurrence. |
| Focused `fret-ui` gates | U4/U5/U6 slices ran focused identity, dirty frontier, dispatch snapshot, command routing, focus, outside press, modal barrier, hit-test, paint-cache, and semantics tests. |
| `fret-ui-kit` / `fret-ui-shadcn` policy gates | U4/U6/U8 included focused `fret-ui-kit`, shadcn wasm compile, and source-policy coverage for policy-vs-mechanism vocabulary. |
| Perf baseline audit | `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict` passed during U4/U7/U8 closeout work. |
| Identity/dirty graph diagnostics | Frame stats now include identity fallback, seeded hit/stale, parent repair, GC reachability, dispatch snapshot cache, dirty frontier breadth, and observation churn signals. |
| Scene/text/upload diagnostics | Perf keys and diagnostics now include scene chunk input/cache/payload/reassembly fields, resident upload fallback and partial-write fields, text shape/cache budgets, glyph atlas budgets, and wasm bundle text resource snapshots. |
| Starter/scaffold compile | `cargo nextest run -p fretboard scaffold` passed for the `workbench-lite` scaffold slice; generated source uses `use fret::app::prelude::*` and source-policy protects default surfaces. Dedicated settings-dialog and real async/mutation diagnostics remain a public app ladder follow-on. |

## Retained And Deferred Items

| Item | Owner | Reason | Gate or next proof |
| --- | --- | --- | --- |
| Stable handle deletion after U4 | Future identity/dirty graph follow-on | U4 intentionally made fallback scans observable before replacing compatibility paths. Introducing and deleting through `StableNodeHandle` is riskier without warmup pressure evidence, and no `StableNodeHandle { index, generation }` source type exists yet. | Use identity fallback metrics, add a 10k keyed-reorder stress gate, and gate fallback scan count plus stale-handle repair after warmup. |
| Entity-first `ViewId` ownership | Future ViewId boundary follow-on | U5 moved dirty frontier and many frame products to `ViewId` / `ViewBoundary` vocabulary, but `DirtyViewFrontier` still uses v1 boundary-node bridge methods and retained NodeId compatibility paths where required. | Remove or shrink `iter_boundary_nodes_v1`-style bridges only after entity-first ViewId lookup and focused dirty/layout/dispatch gates prove parity. |
| Window/layer-forest frame products | Frame pipeline follow-on where proven useful | Dispatch snapshot, final semantics snapshot, command routing, hit-test path routing, and paint recording remain window/layer-forest owners because their correctness spans active layer roots, modal barriers, focus/capture, and command registries. | Promote only after a proof shows a per-boundary owner can preserve cross-layer behavior. Keep ADR 0327 alignment honest meanwhile. |
| Retained mechanism vocabulary | Source-policy maintenance | `Roving*` remains a composite focus mechanism inside `crates/fret-ui`; `ResizablePanelGroupStyle` remains on an explicit mechanism module path; Radix/shadcn policy vocabulary remains in ecosystem/recipe crates. | Keep source-policy protecting root/default exports while allowing explicit mechanism or ecosystem-policy locations. |
| Full public second-hour ladder beyond `workbench-lite` | New narrow follow-on using `docs/workstreams/fretboard-public-app-author-surface-v1/` as upstream evidence | The current plan proved the first copyable second-hour slice. Data-admin, workspace-lite, and canvas/node starters remain separate product surfaces and should not be smuggled into this coordinator. The existing `fretboard-public-app-author-surface-v1` lane is closed, so create a new focused follow-on rather than reopening it. | Extend `crates/fretboard` templates plus diag scripts one starter at a time; keep `tools/check_surface_policy.py` on default app surfaces. |
| `workbench-lite` behavioral depth | New public app ladder follow-on | The shipped template includes a settings dialog, command entrypoints, status/content panes, and stable test IDs, but submit is intentionally simulated and there is no dedicated diag for open/edit/cancel focus restore/save/Escape. | Add a generated-app diag that drives settings dialog behavior and a real async/mutation submit recipe without importing raw runtime seams. |
| Advanced/manual allowlist cleanup | Source-policy maintenance follow-on | U2 intentionally allowed classified advanced surfaces such as `api_workbench_lite_demo`, workspace shell, canvas pan/zoom, and node graph while public wrappers mature. Those allowlist entries must shrink as default wrappers land. | For each promoted wrapper, remove or reclassify the corresponding `ADVANCED_MANUAL_SURFACES` entry in `tools/check_surface_policy.py` and add a negative default-surface fixture. |
| Flat `Scene` bridge | Renderer scene/output follow-on | U7 uses `SceneChunkManifest` for retained chunk identity and upload planning, but flat `Scene` remains the semantic render output source. This is an intentional compatibility bridge until chunk replay is output-equivalent for more stream classes. | Renderer conformance plus scene chunk parity gates for quads, text, paths, clips, masks, and effects before replacing flat-scene output. |
| Non-quad resident partial uploads | Renderer resident upload follow-on | Real partial writes are safe only for quad instances today. Text/path/material/side-table streams still fall back to full upload because dependency closure is not proven. | Start with a side-table-free non-quad subset and require negative tests for clip masks, text paint closure gaps, resource generation changes, and coverage fallback. |
| Full-blob text helper paths | Text/glyph residency follow-on | Runtime frame prepare now uses visible glyph residency, but compatibility helpers remain for chunk/test paths and should not be deleted until chunk-local text closure is complete. | Add per-chunk text-resource closure gates, then remove full-blob helper dependence where no longer needed. |
| U8 web evidence under `target/` | Evidence note, not committed artifact | The bundle and summaries are large generated outputs. This audit preserves exact metrics and paths without committing target artifacts. | Re-run `tools/perf/diag_u8_text_budget_gate.py` for fresh bundle evidence before future release gates. |
| `tools/pre_release.py` full-chain smoke | ADR numbering follow-on | Pre-release currently stops before the consumption-profile step because `tools/check_adr_numbers.py` finds duplicate ADR ID `0324`. U9 did wire the profile gate into pre-release, but the aggregate gate cannot run through until the ADR duplicate is resolved. | Resolve duplicate ADR ID `0324`, then run the pre-release skip-heavy policy chain through the consumption-profile gate. |
| `ecosystem/fret/src/view.rs` line count | U9 facade maintenance | `view.rs` is now mostly re-export and source-shape test host, but remains large because tests aggregate split modules. `view/data.rs` and `view/local_state.rs` are still about 1000 lines each. | Future U9 follow-on may split source-shape tests and data/local-state internals without changing the public facade. |

## U8 Evidence Snapshot

Native budget summary:

- `target/fret-diag-u8-text-budget-gate-native-r1/summary.json`
- text-heavy atlas live bytes `20971520 <= 50331648`
- text-heavy shape entries `1 <= 4096`
- text-heavy shape bytes `415952 <= 33554432`
- code-editor atlas live bytes `4194304 <= 16777216`
- code-editor shape entries `635 <= 4096`
- code-editor shape bytes `6491112 <= 16777216`

Web/wasm budget summary:

- bundle: `target/fret-diag-u8-web-export-code-editor-r3/1782959381479-bundle/bundle.json`
- summary: `target/fret-diag-u8-web-budget-r3/summary.json`
- page: `code_editor_torture`
- `render_text_shape_cache_entries=544`
- `render_text_shape_cache_entry_limit=1024`
- `render_text_shape_cache_bytes_estimate_total=3514264`
- `render_text_atlas_bytes_live_estimate_total=4194304`
- `render_text_atlas_bytes_budget_estimate_total=37748736`
- mask/color/subpixel atlas `max_pages=1`
- `renderer_text_atlas_evicted_pages=0`

## Known Blockers Outside This Closeout

- `python3 tools/check_adr_numbers.py` fails on duplicate ADR ID `0324`:
  `0324-a11y-state-description-semantics-v1.md` and
  `0324-window-input-hit-testing-and-passthrough-v1.md`.
- The blocker predates this closeout and prevents `tools/pre_release.py` from reaching later policy
  gates in a single aggregate run. The individual layering, source-policy, consumption-profile,
  perf-baseline, and U8 text-budget gates have passed independently.

## Final State

This coordinator is closed. Future work should start from the retained/deferred table above or an
existing owner lane, not by reopening this broad workstream.
