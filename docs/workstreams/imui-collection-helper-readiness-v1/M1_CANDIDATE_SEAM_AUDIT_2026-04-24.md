# ImUi Collection Helper Readiness v1 - M1 Candidate Seam Audit

Date: 2026-04-24
Status: landed audit

## Verdict

No shared collection helper is helper-ready yet.

The two current proof surfaces both present collection-shaped UI, but their reusable pressure does
not match:

1. `imui_editor_proof_demo` owns a dense asset-browser grid with app-owned multi-select,
   keyboard owner, box-select marquee, zoom/layout metrics, context menu, duplicate/delete/select-all,
   inline rename, and command status.
2. `editor_notes_demo` owns a compact shell-mounted `Scene collection` outline with three
   single-selection rows, app-owned labels, and existing editor actions.
3. The shared part is currently vocabulary and test-id discipline, not a reusable helper contract.

## Candidate Seam Table

| Candidate seam | Classification | Reason |
| --- | --- | --- |
| `collection(...)` container helper | Not helper-ready | Too broad: the first proof needs pointer capture, scroll, context menu, commands, and layout; the second proof needs a shadcn card/outline wrapper. A shared container would either be empty sugar or absorb app policy. |
| `collection_list(...)` / `collection_rows(...)` | Not helper-ready | The second proof is row/list-shaped, but the first proof is tile/grid-shaped with visible-order, zoom-derived columns, and box-select hit testing. The common row abstraction would not serve the grid. |
| `collection_commands(...)` | App-owned policy | Duplicate, delete, select-all, rename, status strings, and shortcut routing are product semantics. The first proof owns them locally; the second proof has no matching command package. |
| `selection_summary(...)` text helper | Not worth extracting yet | Both surfaces expose summary/status copy, but the strings encode different product stories and are clearer as local app text. |
| Stable collection test-id convention | Helper-adjacent documentation | Both surfaces benefit from root/summary/list/action test IDs, but this is a naming convention or recipe guidance, not a runtime/helper API. |
| Selection state model | Already covered / not this lane | `ImUiMultiSelectState` already serves the first proof. The second proof uses a simple enum selection and does not need multi-select state. |

## Boundary Decision

Keep shared helper widening closed for M1.

The only reusable guidance worth carrying forward is a documentation-level convention for stable
collection test IDs and proof-surface evidence. Do not turn that into `fret-ui-kit::imui` API until
another first-party surface proves the same row/list or command helper shape.

## Evidence

- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_command_package_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`

## Next Move

Close this lane in M2 unless fresh evidence arrives with a narrower helper candidate than:

- generic collection container,
- generic collection rows,
- generic collection commands,
- or generic selection summary text.

If future work wants to improve reuse now, prefer docs/recipe naming guidance over public helper implementation.
