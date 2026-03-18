# Child-Collection Audit — 2026-03-16

This audit records the first post-M1/M2 evidence pass for keyed lists, single-child late landing,
and default child-collection ergonomics.

Goal:

- determine whether keyed/list/default child-collection pressure is still a real shared-surface
  problem after the tracked-read and selector/query reductions,
- or whether the remaining examples are already best explained by the current taught path plus a
  few intentional advanced seams.

## Evidence surfaces used in this pass

Canonical compare set:

- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`

Non-Todo proof surfaces:

- `apps/fret-cookbook/examples/hello.rs`
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`
- `apps/fret-cookbook/examples/embedded_viewport_basics.rs`
- `apps/fret-cookbook/examples/chart_interactions_basics.rs`

Guard rails already in tree:

- `apps/fret-cookbook/src/lib.rs`
- `apps/fretboard/src/scaffold/templates.rs`

Why these surfaces:

- the canonical compare set is where a shared API would first need to justify itself,
- `hello.rs` verifies the smallest single-child page/helper story,
- `assets_reload_epoch_basics.rs` verifies a medium non-Todo app surface with ordinary
  heterogeneous child composition,
- `embedded_viewport_basics.rs` and `chart_interactions_basics.rs` verify whether the remaining
  awkward-looking seams are really default authoring gaps or intentional advanced/interop
  boundaries.

## Findings

### 1. The canonical compare set is already on the intended child-collection posture

Across `simple_todo`, `simple_todo_v2_target`, `todo_demo`, and the generated templates, the
default story is now consistent:

- `ui::for_each_keyed(...)` is the identity-bearing dynamic-list primitive,
- row helpers return `impl UiChild`,
- `ui::single(cx, child)` is the narrow late-landing helper for one typed child,
- ordinary layout composition stays on `ui::children![cx; ...]`.

The compare set no longer teaches:

- `todo_page(...).into_element(cx).into()` as the root wrapper story,
- `ui::children![cx; content]` for single-child late landing,
- or builder/bridge reopening just to express a normal keyed list.

Conclusion:

- the Todo lane no longer demonstrates a missing child-collection API.
- it demonstrates the already-settled default path.

### 2. Non-Todo app-facing surfaces do not show a new shared-surface gap either

`hello.rs` already uses the desired smallest shape:

- `render(...)` late-lands one typed page helper through `ui::single(cx, hello_page(...))`,
- the helper itself returns `impl UiChild`,
- and `ui::children![cx; ...]` remains the straightforward heterogeneous-child syntax inside that
  page.

`assets_reload_epoch_basics.rs` shows the same story at a medium surface:

- the page/card shell uses `ui::single(cx, content)` where exactly one typed child is being
  landed,
- ordinary multi-child body composition remains readable with `ui::children![cx; ...]`,
- the more explicit `v_flex_build(...)` use is tied to conditional sink composition, not to a
  missing default child-collection primitive.

Conclusion:

- outside Todo, the first-party app-facing examples already fit the current
  `ui::single` + `ui::children!` + `ui::for_each_keyed` teaching surface.
- there is no repeated non-Todo evidence that justifies a new shared helper.

### 3. The remaining awkward-looking seams are intentional advanced boundaries

The non-Todo surfaces that still look more manual are doing so for reasons unrelated to default
child-collection authoring.

`embedded_viewport_basics.rs` still ends with `vec![root].into()`, but that surface is on the
advanced `ui_app_with_hooks(...)` / `ViewElements` lane and owns an embedded renderer/viewport
boundary. The extra landing seam is not a normal app-authoring wrapper problem.

`chart_interactions_basics.rs` still keeps `chart_canvas(...) -> AnyElement`, but the file
explicitly documents why: it owns the retained-subtree bridge boundary for `ChartCanvas`.

`assets_reload_epoch_basics.rs` keeps a few helpers on `impl IntoUiElement<KernelApp> + use<>`,
but those are advanced helper surfaces for asset-backed image/SVG elements, not evidence that the
app-facing child-collection path needs another abstraction.

Conclusion:

- the remaining odd seams belong to advanced/runtime/interop boundaries,
- not to the default app-facing child-collection story,
- so they should not drive new public API on the `fret::app` lane.

### 4. Existing gates already protect the intended default path

Current source-policy tests already lock the highest-signal outcomes:

- `apps/fret-cookbook/src/lib.rs` asserts that the shared scaffold prefers `ui::single(cx, surface)`
  over `ui::children![cx; surface]`,
- `apps/fret-cookbook/src/lib.rs` asserts that the canonical compare set keeps the cookbook
  scaffold on `Ui`-returning helpers instead of old eager landing patterns,
- `apps/fretboard/src/scaffold/templates.rs` asserts that generated todo/simple-todo templates use
  `ui::for_each_keyed(...)`, `ui::single(cx, content)`, and avoid the displaced late-landing
  spellings.

Conclusion:

- M3 does not currently need another code-level helper or another public surface,
- it needs the verdict written down so future work does not reopen helper growth without new
  evidence.

## Decision from this audit

Treat M3 as:

- audit complete,
- verdict = docs/adoption only,
- no new shared public child-collection API justified.

The current child-collection baseline should be treated as settled:

- `ui::for_each_keyed(...)` for keyed dynamic lists,
- `ui::children![cx; ...]` for ordinary heterogeneous child groups,
- `ui::single(cx, child)` for single-child late landing,
- explicit `AnyElement` / `ViewElements` seams only where advanced retained/interop ownership is
  the point.

## Immediate execution consequence

Next work should move to M4:

1. keep deleting displaced wording from docs/templates/examples when it appears,
2. keep the existing gates aligned with the same taught path, and
3. refuse new child-collection helpers unless future evidence shows repeated pain on both:
   - the canonical compare set, and
   - at least one non-Todo app-facing surface that is not merely an advanced bridge boundary.
