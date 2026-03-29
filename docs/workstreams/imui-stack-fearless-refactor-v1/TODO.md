# imui stack fearless refactor v1 - TODO

Tracking doc: `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`

Milestones: `docs/workstreams/imui-stack-fearless-refactor-v1/MILESTONES.md`

Historical references:

- `docs/workstreams/imui-authoring-facade-v2/imui-authoring-facade-v2.md`
- `docs/workstreams/imui-ecosystem-facade-v3/imui-ecosystem-facade-v3.md`

This board assumes a workspace-wide breaking migration.
Compatibility shims are explicitly out of scope.

## M0 - Scope freeze and deletion map

- [x] Freeze the ownership split across `fret-imui`, `fret-ui-kit::imui`, and `fret-ui-editor::imui`.
- [x] Confirm the minimal contract that survives unchanged in `fret-imui` (`ImUi`, `UiWriter`
      composition, identity helpers, minimal layout helpers, `Response` re-export).
- [x] Decide the canonical immediate layout vocabulary that survives the reset and delete alias-only
      names from the plan.
- [x] Write down the exact public names that will be deleted from `fret-ui-kit::imui`.
- [x] Record the in-tree demos/tests/proof surfaces that must still pass after the flag-day
      migration.

Verified proof surfaces for the current refactor batch:

- `cargo check -p fret-ui-editor --features imui`
- `cargo check -p fret-imui`
- `cargo nextest run -p fret-imui --lib`
- `cargo test -p fret-ui-editor --features imui --test imui_adapter_smoke`
- `cargo check -p fret-demo --bin imui_editor_proof_demo`

## M1 - Delete compatibility and redundant API surface

- [x] Delete the backward-compatible Cargo feature aliases from `ecosystem/fret-imui/Cargo.toml`
      (`query`, `selector`).
- [x] Delete `begin_disabled` and keep one disabled-scope API.
- [x] Delete alias-only layout helpers such as `same_line` and `items` if the canonical layout
      vocabulary already covers them.
- [x] Collapse `floating_area_show` and `floating_area_show_ex` into one typed canonical family.
- [x] Collapse `window_ex` and `window_open_ex` into one typed canonical family.
- [x] Delete `floating_window_impl_legacy`.
- [x] Delete any re-export or helper that only exists to preserve older naming rather than a distinct
      semantic concept.
- [x] Migrate all in-tree call sites in the same refactor batch instead of keeping compatibility
      wrappers.

## M2 - Split monolith files into reviewable modules

- [x] Shrink `ecosystem/fret-imui/src/lib.rs` into a small crate root and move implementation into
      focused modules.
- [x] Move `fret-imui` tests out of the crate root into a dedicated `src/tests/` module.
- [x] Split `fret-imui` tests into focused submodules by behavior family.
- [x] Split `ecosystem/fret-ui-kit/src/imui.rs` by concern so layout, popup, floating, response, and
      widget helpers stop living in one file.
- [x] Keep module boundaries aligned with ownership boundaries rather than old helper groupings.
- [x] Ensure new module names match the canonical API families chosen in M0.

## M3 - Close editor `imui` adapter coverage

- [x] Expand `ecosystem/fret-ui-editor/src/imui.rs` from a few thin helpers into a systematic thin
      adapter surface.
- [x] Add thin `imui` adapters for `ColorEdit`.
- [x] Add thin `imui` adapters for `NumericInput`.
- [x] Add thin `imui` adapters for `MiniSearchBox`.
- [x] Add thin `imui` adapters for `TextAssistField`.
- [x] Add thin `imui` adapters for `VecEdit`.
- [x] Add thin `imui` adapters for `TransformEdit`.
- [x] Add thin `imui` adapters for `AxisDragValue`.
- [x] Add thin `imui` adapters for `IconButton`.
- [x] Review whether the existing editor control inventory needs any declarative cleanup so the
      adapters stay thin.
- [x] Ensure no editor control gains a second implementation path during adapter expansion.

Current M3 closure evidence:

- `docs/workstreams/imui-stack-fearless-refactor-v1/EDITOR_IMUI_ADAPTER_AUDIT_2026-03-29.md`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`

## M4 - Tests, proof surfaces, and documentation closure

- [x] Update or replace tests that currently exercise deleted compatibility names.
- [x] Add focused regression coverage for the consolidated window/floating/popup API families.
- [x] Add focused regression coverage for the promoted editor `imui` adapters.
- [x] Keep at least one editor-oriented proof/demo surface runnable after the refactor.
- [x] Update workstream references and code comments that still teach deleted API names.
- [x] Update any crate-usage or authoring guidance that would otherwise point users at removed entry
      points.
- [x] If `UiWriter` or `Response` contract shape changes materially, update the relevant ADR or
      contract-tracking notes instead of leaving the change implicit.
  - Closed on 2026-03-29:
    - no material shared-contract change landed in this workstream,
    - `UiWriter` / `Response` closeout evidence is now explicit in
      `UIWRITER_RESPONSE_CONTRACT_CLOSEOUT_2026-03-29.md`,
    - and `ecosystem/fret-authoring/tests/contract_surface_policy.rs` plus
      `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs` now lock the boundary.

Current evidence anchors for M4 closure:

- `ecosystem/fret-authoring/src/lib.rs`
- `ecosystem/fret-authoring/tests/contract_surface_policy.rs`
- `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`
- `docs/workstreams/imui-stack-fearless-refactor-v1/UIWRITER_RESPONSE_CONTRACT_CLOSEOUT_2026-03-29.md`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-demo/src/bin/imui_editor_proof_demo.rs`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- `docs/workstreams/imui-ecosystem-facade-v3/imui-ecosystem-facade-v3.md`
- `docs/workstreams/imui-ecosystem-facade-v3/imui-ecosystem-facade-v3-todo.md`
- `docs/workstreams/imui-ecosystem-facade-v1/imui-ecosystem-facade-v1.md`

## M5 - Closeout review

- [x] Verify there is one canonical entry point per concept on the public `imui` surface.
- [x] Verify `fret-imui` is still policy-light and dependency-light after the cleanup.
- [x] Verify `fret-ui-editor::imui` reads like a thin coverage layer over existing declarative
      controls.
- [x] Verify no legacy/compatibility symbols remain in public docs or public APIs.
- [x] Capture the final "what survived / what was deleted / what became canonical" summary in the
      workstream notes.

Current M5 evidence:

- Evidence for policy-light `fret-imui`:
  - `ecosystem/fret-imui/Cargo.toml`
  - `ecosystem/fret-imui/src/lib.rs`
  - `ecosystem/fret-imui/src/frontend.rs`
  - `ecosystem/fret-authoring/src/lib.rs`
  - `ecosystem/fret-authoring/tests/contract_surface_policy.rs`
  - `docs/workstreams/imui-stack-fearless-refactor-v1/UIWRITER_RESPONSE_CONTRACT_CLOSEOUT_2026-03-29.md`
- Evidence for thin editor adapter coverage:
  - `ecosystem/fret-ui-editor/src/imui.rs`
  - `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
  - `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
  - `docs/workstreams/imui-stack-fearless-refactor-v1/EDITOR_IMUI_ADAPTER_AUDIT_2026-03-29.md`
- Evidence for the final canonical/deleted/survived summary:
  - `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
- Evidence for canonical public floating/select naming:
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/floating_window.rs`
  - `ecosystem/fret-ui-kit/src/imui/select_controls.rs`
- Evidence for canonical public `*_with_options` naming:
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- Evidence for explicit overlay-root options naming:
  - `crates/fret-ui/src/tree/layers/types.rs`
  - `crates/fret-ui/src/tree/layers/impls.rs`
  - `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Evidence for zero live `*_ex` code naming:
  - `crates/fret-ui/src/overlay_placement/solver.rs`
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome/support/overlay_chrome/overlay.rs`
- Evidence for historical-doc isolation:
  - `docs/README.md`
  - `docs/roadmap.md`
  - `docs/workstreams/README.md`
  - `docs/workstreams/standalone/README.md`
  - `docs/workstreams/imui-authoring-facade-v2/imui-authoring-facade-v2.md`
  - `docs/workstreams/imui-authoring-facade-v2/imui-authoring-facade-v2-todo.md`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
  - `docs/workstreams/imui-ecosystem-facade-v1/imui-ecosystem-facade-v1.md`
  - `docs/workstreams/imui-ecosystem-facade-v1/imui-ecosystem-facade-v1-todo.md`
  - `docs/workstreams/imui-ecosystem-facade-v2/imui-ecosystem-facade-v2.md`
  - `docs/workstreams/imui-ecosystem-facade-v2/imui-ecosystem-facade-v2-todo.md`
  - `docs/workstreams/imui-ecosystem-facade-v2/imui-ecosystem-facade-v2-m3-popup-floating-polish.md`
  - `docs/workstreams/imui-ecosystem-facade-v2/imui-ecosystem-facade-v2-m4-perf-gates.md`
  - `docs/workstreams/imui-ecosystem-facade-v3/imui-ecosystem-facade-v3.md`
  - `docs/workstreams/imui-ecosystem-facade-v3/imui-ecosystem-facade-v3-todo.md`
