---
title: UI Kit Gap Analysis (shadcn/ui + gpui-component)
---

# UI Kit Gap Analysis (shadcn/ui + gpui-component)

This document tracks:

- Current UI-kit state in Fret (primitives + components).
- Known issues observed while dogfooding the UI kit (including VirtualList highlighting).
- A concrete gap checklist vs **shadcn/ui (v4)** and **gpui-component `crates/ui`**.

Scope: this is a pragmatic “what’s missing / what’s broken / what’s next” note, not an ADR.

## References (pinned in `repo-ref/`)

### shadcn/ui (non-deprecated)

- Component docs: `repo-ref/ui/apps/v4/content/docs/components/`
- Component implementations (v4): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/`
- Command palette styles (v4 + cmdk): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/command.tsx`

### gpui-component

- UI crate root: `repo-ref/gpui-component/crates/ui/src/lib.rs`
- Virtual list implementation: `repo-ref/gpui-component/crates/ui/src/virtual_list.rs`

## Current Fret inventory

### Framework primitives (`crates/fret-ui`)

- `VirtualList` (virtualization + selection + keyboard nav)
- `Scroll` (scroll container) and scrollbar rendering
- `DialogOverlay` / `TooltipOverlay` / `ContextMenu` / `Popover` overlay primitives
- `TreeView`, `TextArea`, `ResizableSplit`, `Dock` (docking + multi-window)

### Component kit (`crates/fret-components-ui`)

- Buttons: `Button`, `IconButton`, `Toolbar`
- Inputs: `TextField`, `Checkbox`, `Switch`, `Select`, `Slider`
- Overlays: `TooltipArea`, `DropdownMenuButton`, `Dialog`
- Data: `ListView` (virtualized), `ScrollArea`, `ProgressBar`, `Tabs`, `Separator`, `Frame`
- Command palette: `CommandList` + `command_palette::install_command_palette` (cmdk-style shell wiring)
- Combobox: `Combobox` (typeahead input + anchored popover list)

### Tailwind-like primitives (typed)

- `Space` + `Radius` enums (component-layer vocabulary; no runtime class parser).
- Default themes ship a minimal Tailwind-like metric scale via extension keys:
  - `component.space.*` (`0`, `0p5`, `1`, `1p5`, `2`, `2p5`, `3`, `3p5`, `4`, `5`, `6`, `8`, `10`, `11`)
  - `component.radius.*` (`sm`, `md`, `lg`)

Demo: `cargo run -p fret-demo --bin ui_kit`

## Known issues (current)

### VirtualList row highlighting feels “too tight” (UX)

Observed behavior:

- For dense rows (two-line primary/secondary), the selection/hover background can visually “hug” the
  neighboring row text (especially around descenders like `g`), making it look like the highlight is
  overlapping adjacent rows.

Notes:

- In shadcn/ui (DOM), each item is its own element box, and background is naturally clipped to that
  box. In a canvas/scene renderer, we must implement row-local clipping explicitly.
- Fret now clips *each row* when painting `VirtualList` to prevent any text/background bleeding.
- Remaining work is primarily UX tuning (row padding, highlight inset, density presets), not correctness.

Workarounds:

- Use `VirtualListRowHeight::Measured { min: ... }` for multi-line items.
- Prefer list-specific tokens (`metric.list.*`) to avoid forcing global padding changes.

Tracking:

- This should be solved by promoting sizing/density to a single component-level contract (MVP 47):
  `Size` (xs/sm/md/lg) + derived control metrics (list padding/row gaps/input heights), inspired by
  gpui-component’s `Size` + `StyleSized` helpers (`repo-ref/gpui-component/crates/ui/src/styled.rs`).
  - Fret now uses a shared `fret-components-ui` list style helper to keep multi-line spacing consistent
    across list-like components (`ListView`, `CommandList`).

### VirtualList hover not showing on first pointer move (fixed)

Symptom:

- Hover highlight did not appear until after scrolling.

Root cause:

- Pointer events could arrive before the first layout/paint pass; `VirtualList` used stale bounds for
  hover hit-testing.

Fix:

- `VirtualList::event` now updates `last_bounds/last_viewport_height` from `EventCx.bounds`.
- Covered by a regression test: `hover_updates_before_first_layout_pass`.

## Gap checklist vs shadcn/ui (v4)

shadcn/ui is not a virtualization library; for long lists it typically relies on DOM scrolling and
optional integration with `@tanstack/react-virtual` / `react-window`. For Fret, the goal is:

- Provide a **framework-level virtual list primitive** (`VirtualList`) for performance.
- Provide **component-level API parity** with shadcn/ui for common app UIs.

### P0 (core app UI)

- `Command` (command UI): searchable list + groups + shortcuts + keyboard navigation
- `Popover` + `HoverCard` equivalents: anchored overlays and hover previews
- `Toast`/`Sonner`: transient notifications + stacking + timers
- `Menubar`: application menus (native integration later; custom first is ok)
- `Combobox`: typeahead + filtering + virtualization for large option sets

### P1 (layout + data)

- `Table` / `DataTable`: virtualized rows, column resizing, selection
- `Sidebar` + navigation patterns (tree + sections)
- `Accordion` / `Collapsible` / `NavigationMenu`

### P2 (nice-to-have for polish)

- `Skeleton`, `Badge`, `Card`, `Breadcrumb`, `Pagination`, `Carousel`, `Calendar/DatePicker`

## Gap checklist vs gpui-component `crates/ui`

gpui-component is a useful “desktop app” reference for:

- Virtual list and scroll ergonomics (`virtual_list.rs`).
- A broader suite of components and styling conventions (`button`, `menu`, `select`, `table`, `tree`).

Key differences to keep in mind:

- GPUI’s `VirtualList` composes rows as separate elements; DOM-like clipping is implicit.
- Fret records draw ops into a single ordered scene; we must be explicit about clip regions to match
  DOM semantics and avoid paint bleed.

Practical gaps to prioritize for parity:

- `menu` + `popover` + `select` behavior parity (focus, dismissal, keyboard)
- `tree` and `table` patterns (selection models, row actions, virtualized rendering)

## Next steps (recommended)

1. Expand `Command` beyond the list: add a palette-style shell (overlay/backdrop + open/close + focus policy) and
   keyboard navigation parity with shadcn/cmdk (Down/Up/Enter) while keeping the list virtualized.
2. Expand `ListView` API beyond `Vec<String>` (leading/secondary/trailing + group headers/separators).

Update:

- The palette-style shell is now prototype implemented via `fret-ui` `CommandPaletteOverlay` +
  `WindowOverlays` commands (`command_palette.open/close/toggle`) and wired in `fret-demo --bin ui_kit`.
