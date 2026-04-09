# Default App Productization Fearless Refactor v1 (Milestones)

## Status summary

- `M0` Baseline and inherited-decision freeze: **Completed**
- `M1` Blessed-path convergence: **Completed**
- `M2` Rich-template productization: **Completed**
- `M3` Recipe promotion decision: **Completed**
- `M4` Teaching-surface alignment: **Completed**
- `M5` Gates and evidence: **Completed**

## M0 — Baseline and inherited-decision freeze

**Status:** Completed

**What closed**

- Added a dedicated release-facing productization lane for the default app path.
- Froze the inherited decisions this lane must not silently reopen:
  - `LocalState<T>` / `use_local*` remain the default local-state story,
  - grouped local bundles currently teach `*Locals::new(cx)`,
  - `todo` remains the third rung,
  - no universal `AppShell` is introduced here.
- Recorded the initial drift and recipe-pressure evidence.
- Froze the startup ADR posture: no new ADR is required unless the lane discovers real contract
  motion.

**Evidence**

- `docs/workstreams/default-app-productization-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/default-app-productization-fearless-refactor-v1/TODO.md`
- `docs/workstreams/default-app-productization-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/default-app-productization-fearless-refactor-v1/DRIFT_AUDIT_2026-04-02.md`
- `docs/workstreams/default-app-productization-fearless-refactor-v1/RECIPE_PROMOTION_AUDIT_2026-04-02.md`

## M1 — Blessed-path convergence

**Status:** Completed

**What closed**

- Audited the default-ladder evidence set and confirmed the inherited grouped-local target remains
  `*Locals::new(cx)`.
- Converged the live first-party Todo surfaces back to that frozen story:
  - `apps/fret-examples/src/todo_demo.rs`
  - `apps/fret-examples/src/simple_todo_demo.rs`
  - richer and simple scaffold templates in `apps/fretboard/src/scaffold/templates.rs`
- Removed the remaining split between view-owned grouped locals and `app.models_mut().insert(...)`
  style construction for these default-path surfaces.
- Refreshed source-policy and scaffold-template gates so the blessed path is protected against
  format-only drift and future reintroduction of `*Locals::new(app)`.
- Confirmed that no ADR follow-up is required for this slice because no public contract changed.

**Evidence**

- `ecosystem/fret/README.md`
- `docs/examples/todo-app-golden-path.md`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/simple_todo_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fretboard/src/scaffold/templates.rs`

**Validation**

- `cargo nextest run -p fret-examples todo_demo_prefers_default_app_surface simple_todo_demo_prefers_default_app_surface selected_view_runtime_examples_prefer_grouped_state_actions_and_effects`
- `cargo nextest run -p fretboard-dev todo_template_uses_default_authoring_dialect hello_template_uses_default_authoring_dialect simple_todo_template_has_low_adapter_noise_and_no_query_selector`
- `rustfmt --check apps/fret-examples/src/lib.rs apps/fret-examples/src/simple_todo_demo.rs apps/fret-examples/src/todo_demo.rs apps/fretboard/src/scaffold/templates.rs --edition 2024`

## M2 — Rich-template productization

**Status:** Completed

**What closed**

- Classified the richer `todo` scaffold into:
  - product baseline,
  - optional richer example material,
  - framework-showcase drift.
- Slimmed the generated Todo surface so it opens as a product starting point instead of a feature
  wall:
  - ordinary seed copy,
  - query demoted into a secondary focus-note callout,
  - simpler footer summary,
  - no in-card command palette button.
- Added explicit "first cuts" guidance to the generated README so app authors know how to remove
  the selector/query slices.
- Refreshed ladder wording so the third rung is described consistently as a richer product baseline
  with deletable selector/query seams.

**Evidence**

- richer todo template sections in `apps/fretboard/src/scaffold/templates.rs`
- generated template README/help text in `apps/fretboard/src/scaffold/templates.rs`
- `docs/examples/README.md`
- `ecosystem/fret/README.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/workstreams/default-app-productization-fearless-refactor-v1/RICH_TEMPLATE_PRODUCTIZATION_AUDIT_2026-04-02.md`

**Validation**

- `cargo nextest run -p fretboard-dev todo_template_uses_default_authoring_dialect template_readmes_capture_authoring_guidance todo_template_mounts_generated_assets_when_ui_assets_are_enabled hello_template_uses_default_authoring_dialect simple_todo_template_has_low_adapter_noise_and_no_query_selector`
- `rustfmt --check apps/fretboard/src/scaffold/templates.rs --edition 2024`
- `cargo run -p fretboard-dev -- new todo --path target/fretboard-scaffold-smoke.<tmp> --name codex-todo-m2-smoke`

## M3 — Recipe promotion decision

**Status:** Completed

**What closed**

- Audited the current recipe-pressure set across:
  - `apps/fret-examples/src/todo_demo.rs`
  - `apps/fret-examples/src/simple_todo_demo.rs`
  - richer scaffold sections in `apps/fretboard/src/scaffold/templates.rs`
  - existing cookbook/docs shell evidence where relevant.
- Closed on keep-local verdicts for:
  - responsive centered page wrapper,
  - Todo/card header composition,
  - hover-reveal destructive action row.
- Rejected shared recipe promotion because the current consumers remain either:
  - page-shell-shaped,
  - clustered in one Todo-only lane,
  - or not aligned on one stable interaction policy.
- Made the keep-local choice explicit in `todo_demo` by extracting file-local helpers rather than
  minting a shared owner.

**Evidence**

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/simple_todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`
- `docs/workstreams/default-app-productization-fearless-refactor-v1/RECIPE_PROMOTION_AUDIT_2026-04-02.md`
- `docs/workstreams/shell-composition-fearless-refactor-v1/PAGE_SHELL_AUDIT_2026-04-02.md`

**Validation**

- `cargo nextest run -p fret-examples todo_demo_prefers_default_app_surface selected_view_runtime_examples_prefer_grouped_state_actions_and_effects todo_demo_registers_vendor_icons_used_by_layout todo_demo_responsive_layout_prefers_compact_footer_and_inline_actions_on_narrow_width todo_demo_responsive_layout_gives_roomy_shells_more_vertical_headroom todo_demo_responsive_layout_centers_card_once_viewport_is_large_enough todo_demo_responsive_layout_keeps_inline_row_actions_for_non_hover_pointers`
- `rustfmt --check apps/fret-examples/src/todo_demo.rs --edition 2024`

## M4 — Teaching-surface alignment

**Status:** Completed

**What closed**

- Aligned the main ingress docs so they now teach the same first-contact ladder:
  - `hello` as the smallest starter,
  - `simple-todo` as the first real local-state + typed-action app,
  - `todo` as the richer third-rung product baseline with deletable selector/query seams.
- Removed the remaining wording drift that still described `todo` primarily as a selector/query
  baseline or implied that it should replace the first two starters.
- Kept `docs/README.md` pointing at the active lane while the productization work remains in
  progress.
- Confirmed that generated template README guidance still matches the docs ingress story.

**Evidence**

- `docs/README.md`
- `ecosystem/fret/README.md`
- `docs/examples/README.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/first-hour.md`
- `docs/crate-usage-guide.md`
- generated README guidance in `apps/fretboard/src/scaffold/templates.rs`

**Validation**

- `git diff --check -- docs/README.md docs/examples/README.md docs/examples/todo-app-golden-path.md docs/first-hour.md docs/crate-usage-guide.md ecosystem/fret/README.md`
- `cargo nextest run -p fretboard-dev template_readmes_capture_authoring_guidance`

## M5 — Gates and evidence

**Status:** Completed

**What closed**

- Promoted the existing `todo_demo` resize roundtrip scripts into explicit proof artifacts by
  adding layout sidecars alongside bundles and screenshots.
- Replayed the two resize-sensitive proofs against the real native `todo_demo` surface and kept
  exact artifact paths for future triage.
- Tightened the row-list example surface so row-local subtree assembly now uses
  `ui::for_each_keyed_with_cx(...)`, matching the framework's keyed-list guidance.
- Improved diagnostics traceability for the default grouped-local path by preserving app callsites
  through `ecosystem/fret/src/view.rs::local_with(...)`.
- Added source-policy/template coverage that keeps default-ladder page shells local and prevents
  cookbook `centered_page_*` helpers from drifting into app/demo/template teaching surfaces.
- Recorded the remaining grouped-local warning noise as a residual authoring-diagnostics gap rather
  than burying it inside the proof run.

**Evidence**

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fretboard/src/scaffold/templates.rs`
- `ecosystem/fret/src/view.rs`
- `docs/first-hour.md`
- `tools/diag-scripts/tooling/todo/todo-resize-roundtrip-immediate-layout.json`
- `tools/diag-scripts/tooling/todo/todo-resize-roundtrip-footer-within-window.json`
- `docs/workstreams/default-app-productization-fearless-refactor-v1/RESIZE_LAYOUT_PROOF_2026-04-02.md`

**Validation**

- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/tooling/todo/todo-resize-roundtrip-immediate-layout.json --dir target/diag/todo-resize-roundtrip-immediate-layout-m5d --include-screenshots --exit-after-run --launch -- cargo run -p fret-demo --bin todo_demo`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/tooling/todo/todo-resize-roundtrip-footer-within-window.json --dir target/diag/todo-resize-roundtrip-footer-within-window-m5 --include-screenshots --exit-after-run --launch -- cargo run -p fret-demo --bin todo_demo`
- `cargo nextest run -p fret-examples todo_demo_prefers_default_app_surface simple_todo_demo_prefers_default_app_surface selected_view_runtime_examples_prefer_grouped_state_actions_and_effects todo_demo_registers_vendor_icons_used_by_layout todo_demo_responsive_layout_prefers_compact_footer_and_inline_actions_on_narrow_width todo_demo_responsive_layout_gives_roomy_shells_more_vertical_headroom todo_demo_responsive_layout_centers_card_once_viewport_is_large_enough todo_demo_responsive_layout_keeps_inline_row_actions_for_non_hover_pointers`
- `cargo nextest run -p fretboard-dev todo_template_uses_default_authoring_dialect simple_todo_template_has_low_adapter_noise_and_no_query_selector`
- `rustfmt --check apps/fret-examples/src/lib.rs apps/fretboard/src/scaffold/templates.rs ecosystem/fret/src/view.rs --edition 2024`
- `git diff --check -- apps/fret-examples/src/lib.rs apps/fretboard/src/scaffold/templates.rs apps/fret-examples/src/todo_demo.rs ecosystem/fret/src/view.rs docs/first-hour.md tools/diag-scripts/tooling/todo/todo-resize-roundtrip-immediate-layout.json tools/diag-scripts/tooling/todo/todo-resize-roundtrip-footer-within-window.json docs/README.md docs/workstreams/default-app-productization-fearless-refactor-v1`
