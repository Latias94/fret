# P0 Next Follow-On Priority Audit - 2026-04-23

Status: landed maintenance audit
Last updated: 2026-04-23

Related:

- `TODO.md`
- `MILESTONES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-editor-proof-collection-modularization-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-inline-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/DESIGN.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `repo-ref/imgui/imgui.cpp`
- `repo-ref/imgui/imgui_demo.cpp`

## Why this note exists

`imui-editor-proof-collection-modularization-v1` closed the structural follow-on that kept
collection implementation from continuing to accrete inside one host file, and
`imui-collection-command-package-v1` then closed the bounded duplicate-selected plus explicit
rename-trigger command-package follow-on.

That changes the next default non-multi-window question again.

The next mistake would now be either:

- reopening the closed command-package lane to add a third verb,
- widening `fret-ui-kit::imui` from one proof surface,
- or skipping the second real proof surface that the proof-budget rule still requires before any
  shared collection helper growth can reopen.

This note records the post-command-package priority order and the later no-helper-widening verdict
after the second proof surface landed.

## Assumptions

1. `fret-imui` should stay policy-light.
   Evidence: `ecosystem/fret-imui/src/lib.rs` still documents the crate as the minimal immediate
   authoring facade over `fret-ui`, and `tools/audit_crate.py --crate fret-imui` still shows a
   thin public/dependency surface.
   Confidence: Confident.
   Consequence if wrong: we would start pushing editor/product policy into the wrong crate.

2. The proof-budget rule still blocks shared collection helper growth from one proof surface.
   Evidence: `P0_PROOF_BUDGET_RULE_2026-04-12.md` still requires two real first-party proof
   surfaces before widening `fret-ui-kit::imui`, while
   `docs/workstreams/imui-editor-proof-collection-modularization-v1/CLOSEOUT_AUDIT_2026-04-23.md`
   keeps the structural cleanup explicitly demo-local.
   Confidence: Confident.
   Consequence if wrong: we would misread one advanced demo as enough justification for generic API
   growth.

3. The structural maintenance hazard that previously distorted priorities is now materially reduced.
   Evidence: `apps/fret-examples/src/imui_editor_proof_demo.rs` now routes collection rendering
   through `collection::render_collection_first_asset_browser_proof(ui)`, while
   `apps/fret-examples/src/imui_editor_proof_demo/collection.rs` now owns collection assets,
   state, render logic, and unit tests.
   Confidence: Confident.
   Consequence if wrong: we would still need more local cleanup before adding product breadth.

4. Dear ImGui-class collection maturity is now more about a second real proof surface than about
   missing basic collection mechanics on the current proof.
   Evidence: `repo-ref/imgui/imgui_demo.cpp` groups multi-select, delete, context menus, child
   scrolling, and dynamic list maintenance around application-owned selection state, while Fret now
   already has the app-owned selection, box-select, keyboard owner, delete, context menu, zoom,
   rename, inline rename, modularized owner surface, duplicate-selected command, and explicit
   rename trigger on the first proof surface.
   Confidence: Likely.
   Consequence if wrong: we would prioritize the wrong next slice and leave the editor proof
   feeling fragmented.

## Findings

### 1. Just-landed structural priority: proof-local collection modularization

Owner:

- `apps/fret-examples`

What shipped:

- `imui_editor_proof_demo.rs` now keeps the collection boundary explicit instead of carrying the
  whole proof inline.
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs` now owns the collection data,
  models, render logic, and unit tests.
- The new modularization surface/source-policy gates now freeze that host/module split.

Decision:

- treat this as landed and closed,
- keep it demo-local,
- and do not reinterpret the structural cleanup as justification for shared helper growth.

### 2. Just-landed product-depth priority: a bounded app-owned collection command package

Owner:

- `apps/fret-examples`

What shipped:

- The collection proof now routes duplicate-selected through `Primary+D`, one explicit button, and
  the collection context menu.
- The existing inline rename activation now also has an explicit `Rename active asset` button.
- Both slices stay app-owned in `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`.
- No public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` surface widened.

Decision:

- treat `docs/workstreams/imui-collection-command-package-v1/` as a closed closeout record,
- do not reopen it for a third command verb,
- and use it as evidence that the next non-multi-window priority is now the second proof surface.

Non-goal:

- do not turn the closed package into a generic `collection_commands(...)` helper in
  `fret-ui-kit::imui`.

### 3. Closed product-depth priority: landed second real collection proof surface before shared helper growth

Closed proof surfaces:

- closed primary surface: `apps/fret-examples/src/editor_notes_demo.rs`,
- supporting evidence: `apps/fret-examples/src/workspace_shell_demo.rs`.

Why this now closes without shared helper growth:

- no-helper-widening verdict:

- the current problem is no longer missing command-package breadth on the first proof,
- the first shell-mounted second proof surface now exists as `editor_notes_demo`'s `Scene
  collection` left rail,
- the two collection proof surfaces do not yet demand the same reusable helper shape,
- and the proof-budget rule still blocks shared helper widening until a future, separate
  helper-readiness follow-on can name the exact helper and prove both surfaces need it.

Decision:

- close `docs/workstreams/imui-collection-second-proof-surface-v1/` on a no-helper-widening
  verdict,
- prefer an existing shell-mounted demo instead of a new dedicated asset-grid/file-browser demo,
- and do not reopen shared collection helpers directly from this lane.

### 4. Lower-priority generic follow-ons remain explicitly deferred

Still not the default next move:

- reopening `imui-key-owner-surface-v1` for a generic `SetNextItemShortcut()` /
  `SetItemKeyOwner()`-scale facade,
- reopening `imui-child-region-depth-v1` for Dear ImGui-style axis resize or auto-resize,
- widening generic menu/tab IMUI to absorb shell-owned tabstrip/workspace behavior,
- or widening `crates/fret-ui`.

Those may become valid only after stronger first-party proof changes the owner story.

## Recommended execution order

1. keep `imui-collection-second-proof-surface-v1` closed
2. only start a separate helper-readiness follow-on if future evidence names the exact shared helper and proves both collection surfaces need it

Repo-wide note:

- `docking-multiwindow-imgui-parity` still remains the active global parity lane when real
  backend/runner acceptance is available.
- This audit only changes the default order for the next non-multi-window IMUI work.

## Decision from this audit

From this audit forward, the current non-multi-window IMUI order is:

1. keep `fret-imui` and `fret-ui-kit::imui` frozen at the current generic collection floor,
2. treat the bounded command-package lane as closed on duplicate-selected plus explicit rename
   trigger breadth,
3. treat the `editor_notes_demo` `Scene collection` as the landed second shell-mounted proof
   surface and close this cycle on a no-helper-widening verdict,
4. and continue treating structural cleanup as a demo-local responsibility unless stronger evidence
   says otherwise.
