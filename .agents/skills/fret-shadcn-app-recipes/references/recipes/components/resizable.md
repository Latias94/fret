# Component recipe: Resizable panels

Goal: a shadcn-style resizable split layout that feels editor-grade (drag capture, predictable constraints, no layout thrash).

## Upstream references

- shadcn docs: https://ui.shadcn.com/docs/components/resizable
- shadcn source (v4 New York registry): https://github.com/shadcn-ui/ui/blob/main/apps/v4/registry/new-york-v4/ui/resizable.tsx
- `react-resizable-panels` (upstream library used by shadcn): https://github.com/bvaughn/react-resizable-panels
- Local pinned snapshot (optional; not necessarily present on GitHub checkouts):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/resizable.tsx`

## Fret building blocks

- Component surface: `fret-ui-shadcn::ResizablePanelGroup` (+ `ResizablePanel`, `ResizableHandle`).
- Model/state:
  - sizes should be app-owned if you want persistence (settings/layout files)
- Layout:
  - resizable surfaces often behave as layout barriers; verify flex integration in complex containers.

## Checklist (what to verify)

- Drag interaction:
  - handle captures pointer; release always ends drag even if cursor leaves the window
  - double-click reset policy (if supported) is explicit
- Constraints:
  - min/max sizes are enforced under small viewports
  - nested resizable groups don’t produce jitter
- Performance:
  - dragging does not trigger expensive subtree rebuilds outside the group
- Persistence (if desired):
  - sizes round-trip through settings/layout storage cleanly

## `test_id` suggestions

- `resize-group-<name>`
- `resize-handle-<name>`
- `resize-panel-<name>`

## See also

- `references/mind-models/mm-layout-and-sizing.md`
- `fret-docking-and-viewports` (for editor tab/panel docking, which is a different problem)
