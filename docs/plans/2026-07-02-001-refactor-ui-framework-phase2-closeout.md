# UI Framework Phase 2 Fearless Refactor - Closeout

Status: closed with retained bridges
Last updated: 2026-07-02

## Objective

Close `docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md` after the
U1-U14 implementation run. This closeout records the shipped units, the current proof set, and the
bridges that deliberately remain because deleting them still needs a narrower follow-on gate.

This document is a closeout note, not a new implementation plan. Future work should start from one
of the retained-bridge rows below and open a narrow owner lane or plan instead of reopening the
broad Phase 2 plan.

## Verdict

Phase 2 is complete for this goal: every unit U1-U14 has a commit and evidence, and the remaining
compatibility paths are explicit retained bridges with an owner, reason, and deletion gate.

The strongest Definition of Done statements are not all fully satisfied as unconditional deletion
claims. In particular, flat `Scene` is still the default launch input, parent-pointer repair remains
on a normal repair path, GC reachability remains part of retained-tree liveness, text shaping
closure is not complete for all script and decoration cases, and non-quad partial upload support is
limited to the proven `VertexColor` viewport slice. Those are follow-ons, not hidden success
conditions.

## Unit Evidence

| Unit | Status | Evidence |
| --- | --- | --- |
| U1 - Phase 2 identity contract | Closed | Commit `295afd5470`; `docs/adr/0165-dirty-views-and-notify-gpui-aligned.md`; `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md`; `docs/adr/IMPLEMENTATION_ALIGNMENT.md`. |
| U2 - stable handles and element index | Closed | Commit `db0de99ac0`; fixes `65e31accb2` and `02b987b9f6`; `crates/fret-ui/src/tree/identity.rs`; `ElementNodeIndex`; identity debug stats and focused identity tests. |
| U3 - scan-based live resolution deletion | Closed | Commits `f18008447a` and `54fd844cd8`; static search has no normal-path matches for legacy live element scan bridge names; semantics relation resolution now uses the live index. |
| U4 - entity-first `ViewId` and boundary store | Closed | Commits `f0ecbd5227`, `62883b5582`, and `2921133499`; `crates/fret-core/src/ids.rs`; `crates/fret-ui/src/tree/view_boundary.rs`; boundary store migration tests. |
| U5 - boundary bridge and observation cleanup | Closed with retained repair/liveness bridges | Commits `8d4d52a4ce` and `236f0e58cb`; dirty boundary candidates are consumed through boundary records; observation fanout is subscriber-driven. Parent repair and GC reachability remain follow-ons. |
| U6 - chunk closure metadata | Closed for resource-free quad payloads; broader stream closure retained | Commit `3859918538`; `SceneChunk` closure metadata and payload cache evidence. Text, mask, path, material, and effect parity remain follow-ons. |
| U7 - chunk-local text resource key | Closed for normal retained chunk keys | Commit `ff70cc3754`; retained chunk keys use visible glyph residency. Full-blob helper calls remain in test/debug contexts and some renderer tests. |
| U8 - explicit render scene source | Closed as an explicit source split; flat launch input retained | Commit `00b5130cca`; `RenderSceneSource::ResourceFreeQuadChunks` is explicit and unsupported authoritative chunks fail instead of silently falling back. Native and web launch still pass flat scenes with diagnostic chunks. |
| U9 - non-quad partial upload slice | Closed for `VertexColor` viewport vertices | Commit `20a62f5b03`; partial upload gates admit the proven side-table-free viewport vertex stream. Image, text, path, mask, material, and effect streams stay full-upload fallback. |
| U10 - workbench-lite settings diagnostics | Closed | Commit `5e21b99a20`; `tools/diag-scripts/public-app/workbench-lite-settings-dialog.json`; scaffold and public diagnostic evidence. |
| U11 - app-facing mutation and toast wrappers | Closed | Commit `64ed4c4134`; `ecosystem/fret/src/view/actions.rs`; `ecosystem/fret/src/view/effects.rs`; mutation/toast cookbook path no longer names raw action-host seams. |
| U12 - mutation-workbench starter | Closed | Commit `f89cae1499`; generated `mutation-workbench` starter; `tools/diag-scripts/public-app/mutation-workbench-flow.json`; forbidden-import and behavior gates. |
| U13 - retiring quarantine records | Closed | Commit `e481363274`; `tools/check_surface_policy.py`; advanced/manual exceptions now carry owner, category, allowed seams, reason, and retirement metadata. |
| U14 - narrowed `AppUi` facade internals | Closed | Commit `7c608356dd`; `ecosystem/fret/src/view/local_state/bridges.rs`; `ecosystem/fret/src/view/local_state/adapters.rs`; `ecosystem/fret/src/view/data/render.rs`; default prelude omits raw bridge traits. |

## Static Closeout Searches

The closeout audit used these searches to separate deleted bridges from retained bridges:

- Identity bridge search across `crates`, `ecosystem`, and `apps` returned no matches for
  `ViewId(pub NodeId)`, `impl From<ViewId> for NodeId`, `BoundaryId(NodeId)`,
  `iter_boundary_nodes_v1`, `mark_boundary_node_v1`, `clear_boundary_node_v1`,
  `live_nodes_for_element`, `element_id_map_for_window`, or legacy element scan helpers.
- `text_resource_snapshot_for_blobs` still exists under `crates/fret-render-wgpu/src/text` and
  renderer tests. This is not a normal retained-chunk key dependency, but it is still a debug/test
  and parity helper until full shaping-aware chunk closure lands.
- `RenderSceneSource::flat` and `RenderSceneSource::flat_with_diagnostic_chunks` still appear in
  renderer tests and the native/web launch paths. Flat `Scene` is therefore retained as a normal
  launch input, not deleted.
- `repair_parent_pointers_from_layer_roots` is still called from declarative mount and subtree
  layout-dirty repair paths. Parent repair is retained normal-path debt.
- `layout_collapse_layout_observations_time_us` and `paint_collapse_observations_time_us` remain in
  diagnostics aggregation and the perf-key registry for historical bundle compatibility.

## Verification Summary

The final U14 slice passed:

- `cargo check -p fret --all-targets`
- `cargo nextest run -p fret --lib --no-fail-fast`
- focused `fret` facade and source-shape tests for advanced/default prelude separation
- `cargo check -p fret-examples-imui --all-targets`
- `cargo check -p fret-cookbook --all-targets`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_layering.py`
- `cargo fmt --all`
- `cargo fmt --all --check`
- `git diff --check`

Earlier Phase 2 units also recorded focused `fret-ui`, `fret-render-wgpu`, `fretboard`, public app
diagnostic, renderer parity, and perf-matrix gates in their engineering memory entries.

The closeout tail passed:

- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_workstream_catalog.py`
- `python3 tools/check_adr_numbers.py`
- `python3 ~/.codex/skills/engineering-wiki-memory/scripts/wiki_memory.py validate --root docs/knowledge/engineering`
- `git diff --check`

## Retained Bridges And Deletion Gates

| Retained bridge | Owner | Reason | Deletion gate |
| --- | --- | --- | --- |
| Parent-pointer repair in normal paths | Identity and retained-tree follow-on | The retained tree still needs repair after some mount, layer-root, and dirty-layout transitions. Deleting it without pressure evidence can break focus, layout, and cached subtree routing. | Add a keyed reorder and retained-cache pressure gate that records zero parent repairs after warmup, then move repair to debug/assert-only or delete it. |
| GC reachability and retained-tree liveness bridge | Identity and retained-tree follow-on | Retained trees still need explicit liveness cleanup while the repo has not moved to a full per-frame rebuild model. | Prove stale retained nodes cannot satisfy live identity, focus, scroll, or semantics lookups; then shrink reachability to retained-cache cleanup only. |
| Flat `Scene` as native/web launch input | Renderer chunk-native follow-on | Resource-free quad chunks are supported, but text, paths, masks, materials, effects, side tables, and inherited scopes do not yet have complete chunk-native parity. | Add stream, side-table, command, and targeted pixel parity for supported classes, then remove `flat_with_diagnostic_chunks` from normal launch. |
| Full-blob text helper in tests/debug paths | Text closure follow-on | Normal retained chunk keys are visible-glyph local, but full shaping-run/cluster closure is not proven for ligatures, RTL, combining marks, fallback fonts, decorations, selection, and caret. | Add shaping-aware chunk closure gates and delete or test-hide full-blob helper uses that are no longer parity scaffolding. |
| Full-upload fallback for non-quad streams | Renderer upload follow-on | Only quad streams and resource-free `VertexColor` viewport vertices have proven closure. Other streams have side-table and resource dependencies that can silently corrupt output if partially uploaded too early. | For each stream, name closure owner, side-table/resource dependency coverage, fallback reason, write-count metrics, and negative coverage-gap tests before enabling partial writes. |
| Advanced/manual source-policy quarantine records | Public facade maintenance | Some advanced examples are legitimate proof surfaces until public wrappers mature. Bare allowlists were replaced, but the records are still migration state. | Remove or reclassify each record when its public wrapper or generated starter lands; keep negative default-surface fixtures. |
| `LocalState::new_in` and explicit raw bridge traits | App facade follow-on | U14 removed raw `LocalState` bridge methods from inherent/default autocomplete, but manual/hybrid code still needs explicit advanced access to a `ModelStore` owner. | Add public app-facing constructors for remaining cookbook/starter needs, then shrink advanced bridge exports further if no manual lane needs them. |
| Historical observation-collapse perf keys | Diagnostics compatibility follow-on | Old bundles and reports still consume `layout_collapse_layout_observations_time_us` and `paint_collapse_observations_time_us`. | Keep compatibility until bundle readers can map historical keys to replacement observation subscriber metrics, then remove registry/report fields with migration notes. |

## ADR Alignment

No new ADR is needed for this closeout. The implementation alignment matrix already records the
Phase 2 posture for ADR 0066, ADR 0165, and ADR 0327. The important closeout constraint is not to
upgrade those rows to "fully aligned" while the retained bridges above remain in normal paths.

## Post-Deploy Monitoring And Validation

No production deployment monitoring is required for this closeout commit because the tail is
documentation and engineering memory only. Runtime-impact validation for the Phase 2 code changes
is represented by the per-unit gates above; future runtime follow-ons should add diagnostics or perf
gate monitoring tied to their specific bridge deletion.
