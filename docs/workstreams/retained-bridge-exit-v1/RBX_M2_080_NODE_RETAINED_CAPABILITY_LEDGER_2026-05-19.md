# RBX-M2-080 Node Retained Capability Ledger

Date: 2026-05-19

## Claim

`fret-node` still needs a retained compatibility island, but that island is now explicit enough to
use as a deletion oracle. The public authoring path remains declarative-first; retained code must
not grow outside the migration ledger.

This slice does **not** delete retained node graph behavior. It records the remaining retained
capability surface and adds a source-policy gate so later deletion slices can prove capability
parity before removing code.

## Current Boundary

Public/default node graph UI surface:

- `NodeGraphSurfaceBinding`
- `node_graph_surface(...)`
- `node_graph_surface_in(...)`
- controller/store-first viewport and transaction helpers
- declarative paint-only surface modules under `ecosystem/fret-node/src/ui/declarative/paint_only/`
- default-gated overlay/panel/screen-space policy modules
- default-gated controls overlay composition in
  `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
- default-gated blackboard overlay composition in
  `ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs`
- default-gated minimap overlay composition in
  `ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs`
- default-gated toolbar overlay composition in
  `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
- default-gated rename overlay composition in
  `ecosystem/fret-node/src/ui/overlays/rename_declarative.rs`
- default-gated rename command/session policy in
  `ecosystem/fret-node/src/ui/overlays/rename_command.rs`
- default-gated portal editor chrome

Retained compatibility island:

- `ecosystem/fret-node/Cargo.toml`
  - `compat-retained-canvas = ["fret-ui", "fret-ui/unstable-retained-bridge"]`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/**`
- `ecosystem/fret-node/src/ui/canvas/middleware.rs`
- `ecosystem/fret-node/src/ui/a11y.rs`
- `ecosystem/fret-node/src/ui/diag_anchors.rs`
- `ecosystem/fret-node/src/ui/editor.rs`
- `ecosystem/fret-node/src/ui/panel.rs`
- `ecosystem/fret-node/src/ui/portal.rs`
- `ecosystem/fret-node/src/ui/retained_event_tail.rs`
- `ecosystem/fret-node/src/ui/retained_submit.rs`
- `ecosystem/fret-node/src/ui/editors/portal_number.rs`
- `ecosystem/fret-node/src/ui/editors/portal_text.rs`
- retained-gated overlay widget/layout/paint hooks in:
  - `ecosystem/fret-node/src/ui/overlays/blackboard.rs`
  - `ecosystem/fret-node/src/ui/overlays/blackboard_paint.rs`
  - `ecosystem/fret-node/src/ui/overlays/controls.rs`
  - `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
  - `ecosystem/fret-node/src/ui/overlays/minimap.rs`
  - `ecosystem/fret-node/src/ui/overlays/mod.rs`
  - `ecosystem/fret-node/src/ui/overlays/panel_button_paint.rs`
  - `ecosystem/fret-node/src/ui/overlays/panel_pointer_policy.rs`
  - `ecosystem/fret-node/src/ui/overlays/toolbars.rs`
  - `ecosystem/fret-node/src/ui/overlays/toolbars_layout.rs`

## Capability Map

| Capability family | Retained oracle | Declarative/default coverage today | Deletion requirement |
| --- | --- | --- | --- |
| Large graph paint, culling, paint cache, skin/style/geometry overrides | `ui/canvas/widget/**` retained paint/layout tests | `node_graph_surface(...)` paint-only surface and default `fret-node` tests | Add default declarative tests for the retained conformance families before deleting the retained canvas leaf. |
| Pan/zoom, fit view, viewport helpers | retained canvas event/command/view queue tests | `NodeGraphSurfaceBinding` viewport helpers and gallery/demo declarative usage | Keep default binding tests as the contract; backfill any retained-only gesture semantics before deleting retained event code. |
| Selection, drag, resize, wire creation/reconnect, marquee, context/searcher menus | retained canvas event tests under `ui/canvas/widget/tests` | store/controller transaction helpers plus paint-only input modules | Move event arbitration onto declarative mechanisms or add a Canvas-style event leaf; every retained interaction family needs default-path tests. |
| Overlay panels: blackboard, controls, minimap, toolbars, rename | retained overlay widgets and retained overlay conformance tests | default overlay policy/layout tests from RBX-M2-060; default controls overlay composition tests from RBX-M2-100; default blackboard overlay composition/action-hook tests from RBX-M2-105; default minimap overlay composition/paint-plan tests from RBX-M2-106; default toolbar overlay composition/placement tests from RBX-M2-107; default rename overlay composition and submit/cancel command protocol tests from RBX-M2-108; default rename command/session application tests from RBX-M2-109 | Continue moving retained overlay conformance intent to default tests before deletion. Rename still needs seed-text ownership during layout, focus request/restore, focus-loss close integration, blackboard handoff parity, and retained paint/hit testing; minimap still needs keyboard/pointer/focus/viewport update parity; toolbars still need child measurement, child-root paint, hit testing, and target-resolution parity; blackboard and controls still need default interaction/focus/paint parity before deleting their retained widgets. |
| Portal editor chrome and command submission | `portal_text.rs`, `portal_number.rs`, retained portal lifecycle tests | default editor chrome tests from RBX-M2-070; default portal command protocol from RBX-M2-085; default text/number command policy from RBX-M2-090; default text/number command session adapter from RBX-M2-095 | Replace retained portal subtree rendering/model adapters with declarative portal hosting before deleting retained portal files. |
| Accessibility and diagnostics anchors | `a11y.rs`, `diag_anchors.rs`, retained semantics tests | declarative paint-only semantics/diagnostics modules exist | Add default declarative semantics/diagnostics anchor tests before deleting retained anchors. |
| Middleware extension points | retained `EventCx` / `CommandCx` based `NodeGraphCanvasMiddleware` | no public retained authoring surface; middleware is crate-private/test-only | Replace or delete retained middleware after event/command handling has a declarative host contract. |

## New Gate

`surface_policy_tests::retained_bridge_source_usage_stays_on_the_migration_ledger` scans
`ecosystem/fret-node/src/ui` and fails if code-level retained bridge usage appears outside the
explicit retained migration ledger.

The gate deliberately allows the current retained oracle files. Later migration slices should
shrink the allowed list as declarative coverage replaces retained behavior.

## Next Slices

Recommended order:

1. Continue `RBX-M2-100`/`RBX-M2-105`/`RBX-M2-106`/`RBX-M2-107`/`RBX-M2-108`/`RBX-M2-109`-style
   overlay slices: backfill default rename seed-text/focus ownership and focus-loss integration,
   minimap keyboard/pointer/focus/viewport update parity, toolbar child measurement/paint/hit-test
   parity, and blackboard/controls interaction/focus/paint parity before deleting their retained
   oracles.
2. `RBX-M2-110`: extract a declarative event/canvas leaf for retained canvas interaction families
   or split those policies behind controller/store-first APIs.
3. `RBX-M2-120`: remove `compat-retained-canvas` from `fret-node` only after the retained
   conformance families above have default declarative coverage.

## Verification

Fresh commands are recorded in `EVIDENCE_AND_GATES.md`.
