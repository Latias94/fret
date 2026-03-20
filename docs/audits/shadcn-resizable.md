# shadcn/ui v4 Audit - Resizable

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui
- react-resizable-panels: https://github.com/bvaughn/react-resizable-panels

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Resizable` recipe against the upstream docs and the
`react-resizable-panels`-inspired interaction surface.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/resizable.mdx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/resizable.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/resizable.rs`
- Copyable usage snippet: `apps/fret-ui-gallery/src/ui/snippets/resizable/usage.rs`

## Audit checklist

### Authoring surface

- Pass: `ResizablePanelGroup::new(model)` plus `.entries([...])` already cover the common panel-group
  authoring model.
- Pass: `ResizablePanel`, `ResizableHandle`, and `ResizablePanelGroup::axis(...)` match the upstream
  parts surface closely enough for copyable shadcn-style examples.
- Pass: `ResizableHandle::with_handle(true)` and layout refinement hooks cover the important recipe outcomes.
- Pass: `resizable_panel_group(cx, model, |cx| ..)` is already the first-party composable root lane for
  UI Gallery snippets; it preserves ordered panel/handle composition while keeping root-level builder knobs.
- Note: Because this component is already expressed as explicit parts, Fret does not need an additional
  generic `compose()` or broader children API here.

### Interaction & layout parity

- Pass: The core resize interaction lives in the dedicated resizable recipe rather than leaking policy into
  general layout primitives.
- Pass: Horizontal and vertical groups are both supported.
- Pass: `ResizablePanelGroup` owns the upstream `w-full h-full` + orientation flex behavior, while `ResizablePanel` stays a runtime-sized wrapper and outer bordered demo shells remain caller-owned.
- Pass: Gallery examples already cover nested groups, handle visuals, and RTL hit-testing smoke checks.

### UI Gallery docs surface

- Fix landed: diag scripts and the layout sweep now anchor to concrete resizable surfaces
  (`ui-gallery-resizable-demo`, `ui-gallery-resizable-handle`, `ui-gallery-resizable-panels`)
  instead of depending on a page-wrapper contract.
- Fix landed: the launch-smoke script now treats `ui-gallery-resizable-demo-title` as the
  visibility anchor instead of requiring the whole `ui-gallery-resizable-demo` section wrapper to
  fit inside the window; the wrapper is legitimately taller than the viewport in the default gallery shell.
- Fix landed: section ordering now follows the upstream docs flow (`Demo`, `Usage`, `Vertical`, `Handle`,
  `RTL`) instead of presenting `Handle` before `Vertical`.
- Pass: the remaining notes now explicitly document the authoring-surface conclusion that no extra generic
  composable children API is needed.

## Conclusion

- Result: This component does not currently point to a missing mechanism-layer gap.
- Result: Default-style ownership looks correct: group fill sizing + handle chrome are recipe-owned, while surrounding card/border shells remain caller-owned.
- Result: The main drift was on the UI Gallery teaching/automation surface, not the mechanism layer.
- Result: The main missing piece was regression coverage for the gallery/docs-aligned example set.
- Result: Follow-up work should focus on concrete keyboard or hit-testing regressions only if they appear.

## Validation

- `cargo check -p fret-ui-gallery --message-format short`
