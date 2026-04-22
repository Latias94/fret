# M1 Target Surface Freeze — 2026-04-22

Purpose: freeze which `BeginChild()`-scale concerns are still credible generic `child_region`
candidates after re-reading the current helper, the current pane-first proof surfaces, the shell
resize owner, and the local Dear ImGui reference.

## Evidence reviewed

- `docs/workstreams/imui-child-region-depth-v1/DESIGN.md`
- `docs/workstreams/imui-child-region-depth-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0217-scroll-offset-children-transform-and-scrollhandle-invalidation-v2.md`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
- `ecosystem/fret-ui-kit/src/style/layout.rs`
- `ecosystem/fret-imui/src/tests/composition.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `ecosystem/fret-workspace/src/panes.rs`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`

## Findings

### 1) Fret already has explicit layout sizing; this lane should not clone Dear ImGui `size_arg`

The current generic helper already accepts `ChildRegionOptions.layout`, and Fret's layout surface
already exposes:

- explicit width/height sizing,
- fill sizing,
- min/max constraints,
- and flex grow/shrink/basis posture.

The current first-party pane proofs already use that surface directly:

- `workspace_shell_demo` sets explicit `w_full`, `h_full`, and fixed `h_px` / `w_px` values on
  nested child regions,
- and no first-party evidence currently argues that the repo needs a second child-specific size
  grammar layered on top of `LayoutRefinement`.

Conclusion:

- do not clone Dear ImGui's `size_arg` grammar into generic IMUI.
- Keep sizing on `LayoutRefinement`.

### 2) Frame/padding posture is the first credible generic candidate

The current helper always renders one framed card-like surface:

- `.p_2()`
- `.rounded_md()`
- `.border_1()`
- `bg(card)`
- `border_color(border)`

That means the current generic helper has only one visual posture.
Dear ImGui distinguishes default child windows from the optional `FrameStyle` posture, and this
difference maps more cleanly to helper-local presentation policy than resize or shell-level pane
ownership does.

Conclusion:

- frame/padding posture is the first credible generic child-region candidate for M2.
- If M2 lands a slice, prefer one bounded style/chrome option over a large flag bag.

### 3) Axis-specific resize should stay shell-owned for now

Dear ImGui exposes `ResizeX` / `ResizeY`, but the current first-party Fret evidence points to a
different owner:

- pane resizing in the editor/workbench story is already owned by
  `workspace_pane_tree_element_with_resize(...)` in `ecosystem/fret-workspace/src/panes.rs`,
- the workbench shell closeout already froze explicit shell assembly over the existing workspace
  starter set,
- and the current `child_region` proofs only show embedded pane content, not a second generic
  resizable split/layout owner.

Conclusion:

- axis-specific resize should stay out of generic `child_region` for now.
- Keep resize in shell/product owners unless another non-workbench first-party consumer appears.

### 4) Axis-specific auto-resize should stay deferred for now

Dear ImGui exposes `AutoResizeX`, `AutoResizeY`, and `AlwaysAutoResize`, but those flags carry a
deeper measurement/clipping contract.
Fret's current scroll contract already emphasizes a different baseline:

- scroll offset should behave like a children-only translation,
- scroll invalidation should stay fine-grained,
- and the current helper already forwards scroll handles / viewport test ids without introducing a
  second measurement contract.

The current first-party child-region consumers do not yet show a second real need for
content-measured child auto-resize beyond explicit `LayoutRefinement`.

Conclusion:

- auto-resize / always-auto-resize should stay deferred for now.
- Reopen only with stronger first-party evidence than the current shell-mounted pane proofs.

### 5) Focus-boundary posture should stay deferred for now

Dear ImGui exposes `NavFlattened`, but the current Fret evidence is still too thin to admit a
generic equivalent:

- the current child-region composition proof covers embedded menu composition, not cross-boundary
  keyboard navigation policy,
- workbench-shell focus/command scope already has stronger shell owners,
- and the lane still lacks a focused child-region keyboard proof that would justify a shared
  contract.

Conclusion:

- do not admit a generic child-region focus-flattening surface in this cycle.

### 6) Visibility/clipping posture should not become a `BeginChild() -> bool` clone

Dear ImGui `BeginChild()` has an imperative begin/end return contract.
That does not map cleanly to Fret's declarative-on-retained posture:

- `child_region` is a closure-scoped declarative helper,
- clipping is already handled by the scroll/container substrate,
- and the current lane has no first-party evidence that a bool-return early-out surface would be a
  better fit than the existing retained subtree + clipping model.

Conclusion:

- do not admit a `BeginChild() -> bool`-style return contract in generic IMUI.

## Target-surface freeze

Current M1 result:

1. Keep generic sizing on `LayoutRefinement`; do not clone Dear ImGui `size_arg`.
2. Keep frame/padding posture as the first credible generic candidate for M2.
3. Defer axis-specific resize to shell/product owners.
4. Defer auto-resize / always-auto-resize until stronger first-party proof exists.
5. Defer focus-boundary flattening until a focused keyboard proof exists.
6. Reject a `BeginChild() -> bool` clone for this declarative helper surface.

## First defer list

The lane now explicitly keeps these out of scope for M1/M2 unless stronger evidence appears:

- collection marquee / lasso breadth,
- key-owner / collection keyboard-owner depth,
- menu/tab policy,
- shell-helper promotion,
- shell-owned pane resize,
- runner/backend multi-window parity,
- and an imperative begin/end return contract clone.

## Immediate execution consequence

From this point forward:

1. treat frame/padding posture as the only current credible M2 candidate,
2. keep axis resize, auto-resize, nav flattening, and begin-return posture out of generic
   `child_region` by default,
3. keep the current pane-first proof pair unchanged,
4. and prefer a no-new-generic-surface verdict over a wide `ChildRegionFlags` clone if M2 cannot
   justify even the bounded style/chrome slice.
