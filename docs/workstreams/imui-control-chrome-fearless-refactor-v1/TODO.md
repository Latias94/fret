# ImUi Control Chrome Fearless Refactor v1 - TODO

Status: closed closeout lane
Last updated: 2026-04-20

## Lane setup

- [x] Start a narrow follow-on instead of widening `imui-editor-grade-product-closure-v1`.
- [x] Wire the lane into `docs/roadmap.md`, `docs/workstreams/README.md`, and
      `docs/todo-tracker.md`.
- [x] Keep the lane narrow enough that checkbox/selectable/disclosure follow-ons can split out
      later if they stop sharing the same control-chrome owner.

## M0 - Baseline and owner freeze

- [x] Record why this is a new lane rather than another umbrella-lane checklist item.
- [x] Write the assumptions-first baseline audit with code anchors and proof surfaces.
- [x] Freeze the first migration set:
      - `button_controls.rs`
      - `boolean_controls.rs`
      - `slider_controls.rs`
      - `combo_controls.rs`
      - `combo_model_controls.rs`
      - `text_controls.rs`
- [x] Record the current broken truths with explicit evidence:
      - text-like interactive defaults,
      - compact-rail overlap/clipping,
      - and combo trigger ownership drift.

## M1 - Shared control chrome owner

- [x] Introduce one shared IMUI control chrome owner in `ecosystem/fret-ui-kit::imui`.
- [x] Migrate button-like controls onto that shared owner.
- [x] Migrate switch/toggle-like controls onto that shared owner.
- [x] Migrate slider controls onto field-like chrome with explicit value/track ownership.
- [x] Migrate combo triggers onto field-like chrome without piggybacking on selectable-row visuals.
- [x] Migrate text inputs to the same field-width and compact-rail posture where appropriate.
- [x] Delete the old text-like default visuals instead of keeping compatibility shims.

## M2 - Proof surfaces and cleanup

- [x] Update `imui_interaction_showcase_demo` so it proves the new shared surface rather than
      compensating for old defaults.
- [x] Update `imui_shadcn_adapter_demo` so it remains a small proof that the IMUI helpers now read
      as real controls.
- [x] Keep `imui_response_signals_demo` focused on behavior proof, but make sure the migrated
      controls do not regress its readability.
- [x] Re-audit whether the showcase compact lab still needs a special-width workaround after the
      shared control chrome lands.
- [x] Record a component-family audit against `repo-ref/imgui` so the next slice is selected by
      real gap priority instead of memory.
- [x] Land the first Dear ImGui family catch-up slice that fits the shared chrome owner:
      `small_button`, `arrow_button`, `invisible_button`, and `radio`.
- [x] Land one owner-correct informational helper slice from Dear ImGui families:
      `bullet_text`, while keeping `separator_text` explicitly audited as already present.

## M3 - Gates

- [x] Keep `fret-imui` interaction tests green after the migration.
- [x] Add or extend focused tests for the migrated control families where the old visuals encoded
      implicit behavior assumptions.
- [x] Promote a showcase diag check that directly protects control discoverability / compact layout
      after the new chrome lands.
- [x] Leave one reviewable screenshot anchor for the migrated showcase surface.

## M4 - Closeout and routing

- [x] Close this lane explicitly once the shared control chrome owner, compact rail proof, and
      focused gate package are explicit enough to stop using this folder as an active execution
      queue.
      Result: `FINAL_STATUS.md` now closes the lane as the shared IMUI control-chrome rewrite
      record.
- [x] Route any future pressure to a narrower follow-on instead of reopening this lane as a generic
      IMUI parity bucket.
      Result: future field-width policy and checkbox/selectable/disclosure parity work now route to
      separate follow-ons.
