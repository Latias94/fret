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

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/resizable.mdx`

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
- Note: Because this component is already expressed as explicit parts, Fret does not need an additional
  generic `compose()` builder here.

### Interaction & layout parity

- Pass: The core resize interaction lives in the dedicated resizable recipe rather than leaking policy into
  general layout primitives.
- Pass: Horizontal and vertical groups are both supported.
- Pass: Gallery examples already cover nested groups, handle visuals, and RTL hit-testing smoke checks.

## Conclusion

- Result: This component does not currently point to a missing mechanism-layer gap.
- Result: The main missing piece was gallery/docs alignment with the upstream `Usage` section.
- Result: Follow-up work should focus on concrete keyboard or hit-testing regressions only if they appear.

## Validation

- `cargo check -p fret-ui-gallery --message-format short`
