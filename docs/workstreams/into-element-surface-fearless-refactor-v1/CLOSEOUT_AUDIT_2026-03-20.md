# Closeout Audit — 2026-03-20

This audit records the final closeout read for the into-element surface v1 lane.

Goal:

- verify whether the repo still has an active conversion-surface migration backlog,
- separate intentional raw `AnyElement` seams from already-migrated default teaching surfaces,
- and decide whether this lane should remain active or become historical maintenance evidence.

## Audited evidence

Core workstream docs:

- `docs/workstreams/into-element-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/TODO.md`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/MIGRATION_MATRIX.md`

Representative closure gates:

- `ecosystem/fret/tests/reusable_component_helper_surface.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_internal_previews.rs`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`

Representative source anchors:

- `ecosystem/fret-ui-kit/src/lib.rs`
- `ecosystem/fret/src/lib.rs`
- `apps/fretboard/src/scaffold/templates.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-cookbook/src/lib.rs`
- `apps/fret-ui-gallery/src/ui/doc_layout.rs`

Validation run used for closeout:

- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app copyable_ui_gallery_snippet_lane_has_no_top_level_raw_render_roots direct_recipe_root_pages_mark_their_default_lane_without_inventing_compose navigation_menu_and_pagination_pages_keep_their_dual_lane_story gallery_doc_layout_retains_only_intentional_raw_boundaries`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews wrap_preview_page_callers_land_the_typed_preview_shell_explicitly render_doc_page_callers_land_the_typed_doc_page_explicitly internal_preview_scaffold_retains_only_the_audited_vec_anyelement_seams gallery_overlay_preview_retains_intentional_raw_boundaries`
- `cargo nextest run -p fret --test reusable_component_helper_surface`
- `git diff --check -- docs/README.md docs/roadmap.md docs/workstreams/README.md docs/workstreams/into-element-surface-fearless-refactor-v1/DESIGN.md docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md docs/workstreams/into-element-surface-fearless-refactor-v1/MILESTONES.md docs/workstreams/into-element-surface-fearless-refactor-v1/TODO.md docs/workstreams/into-element-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-20.md`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`

## Findings

### 1. The public conversion taxonomy collapse is already landed

The repo no longer has an unresolved choice between multiple public conversion concepts.

What is now true on the shipped surface:

- app-facing docs/templates/examples teach `Ui`, `UiChild`, and `.into_element(cx)` as an
  operation instead of a trait taxonomy;
- reusable component-facing code converges on `IntoUiElement<H>`;
- the old public-looking split names (`UiIntoElement`, `UiHostBoundIntoElement`,
  `UiChildIntoElement`, `UiBuilderHostBoundIntoElementExt`) survive only in historical docs and
  negative source-policy assertions.

Conclusion:

- the central conversion-vocabulary goal of this lane is satisfied.

### 2. The remaining `AnyElement` seams are classified raw boundaries, not default-lane migration debt

The current tree still contains `AnyElement`, but the audited evidence shows that these usages are
not the same kind of problem this lane opened to solve.

Current remaining categories are intentional:

- preview registry seams that still dispatch `Vec<AnyElement>`,
- diagnostics or torture surfaces that explicitly keep raw boundaries,
- advanced/manual composition or interop leaves,
- component-family root landing seams that are already documented as direct-root or dual-lane
  families rather than default app-lane helpers.

At the same time, the negative evidence is stronger:

- the representative app/cookbook/scaffold lanes no longer teach `.into_element(cx).into()`,
- default UI Gallery snippet/page surfaces are source-gated away from top-level raw render roots,
- internal preview wrappers are source-gated so typed page shells stay typed while registry seams
  remain explicit.

Conclusion:

- the remaining raw seams are now inventory work, not a standing migration queue.

### 3. The lane's proof surfaces are already locked by tests instead of only by prose

This lane is closed for the right reason: its claims are backed by gates, not just by workstream
notes.

Representative closure protection now exists on:

- curated reusable helper posture,
- default UI Gallery snippet/page authoring,
- internal preview shell ownership,
- shadcn family/root-lane policy.

That means future drift can be caught without reopening the broad conversion-surface redesign
question.

Conclusion:

- this folder should now be read as closeout evidence plus maintenance guardrails.

### 4. The remaining useful work is closeout maintenance only

What still belongs here after closeout is narrow:

1. keep the intentional raw-seam inventory accurate,
2. keep top-level docs/indices synchronized with the already-closed state,
3. only reopen if a new cross-surface regression shows that the public conversion vocabulary has
   fragmented again.

What does **not** belong here anymore:

- new trait-budget exploration,
- broad Gallery migration sweeps,
- reopening family taxonomy without fresh evidence,
- inventing new helper families only to remove already-audited explicit landing seams.

Conclusion:

- this lane no longer owns an active implementation backlog.

## Decision from this audit

Treat `into-element-surface-fearless-refactor-v1` as:

- closed for the pre-release conversion-surface cleanup goal,
- maintenance/historical evidence by default,
- and reopenable only through fresh cross-surface evidence that the settled `IntoUiElement<H>` /
  `UiChild` posture has regressed.

## Immediate execution consequence

From this point forward:

1. keep app-facing teaching surfaces on `Ui` / `UiChild`,
2. keep reusable component-facing conversion on `IntoUiElement<H>`,
3. keep `AnyElement` explicit only at intentional raw seams,
4. do not treat isolated raw preview/diagnostics boundaries as proof that this lane is open again,
5. prefer targeted seam-inventory updates or source-gate fixes over another broad migration pass.
