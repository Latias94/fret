---
title: UI Kit Gap Analysis (shadcn/ui + gpui-component)
---

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- gpui-component: https://github.com/longbridge/gpui-component
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

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
- Overlay mechanism: multi-root layers, hit-testing, focus/capture routing (`UiTree` layers; see ADR 0011)
- Menu request store: `ContextMenuService` + `ContextMenuRequest` (surface moved to component kit)
- `TreeView` (legacy, gated), `TextArea`, `ResizableSplit`, `Dock` (docking + multi-window)

### Component kit (`ecosystem/fret-ui-shadcn` on top of `ecosystem/fret-ui-kit`)

- Buttons: `Button`, `IconButton`, `Toolbar`
- Inputs: `TextField`, `Checkbox`, `Switch`, `Select`, `Slider`
- Overlays: `ContextMenu`, `Popover`, `DialogOverlay`, `CommandPaletteOverlay`, `AppMenuBar`, `TooltipArea`, `ToastOverlay`, `DropdownMenuButton`
- Data: `ListView` (virtualized), `ScrollArea`, `ProgressBar`, `Tabs`, `Separator`, `Frame`
- Command palette: `CommandList` + `command_palette::install_command_palette` (cmdk-style shell wiring)
- Combobox: `Combobox` (typeahead input + anchored popover list)

### Tailwind-like primitives (typed)

- `Space` + `Radius` enums (component-layer vocabulary; no runtime class parser).
- Tailwind primitive parity backlog (gpui-component alignment): `docs/archive/backlog/tailwind-primitive-parity-todo.md`
- Default themes ship a minimal Tailwind-like metric scale via extension keys:
  - `component.space.*` (`0`, `0p5`, `1`, `1p5`, `2`, `2p5`, `3`, `3p5`, `4`, `5`, `6`, `8`, `10`, `11`)
  - `component.radius.*` (`sm`, `md`, `lg`)
- Fallback rule: when `component.space.*` / `component.radius.*` tokens are missing, `Space`/`Radius` fall back to baseline `metric.*` tokens (and a scale derived from `metric.padding.sm`) to avoid theme drift.

Demo: `cargo run -p fret-demo --bin components_gallery`

### Theme sanity check (baseline-only)

To verify that component spacing/radius stays consistent even when a theme omits `component.*` tokens,
copy `themes/fret-baseline-only.json` to `.fret/theme.json` and run the UI kit demo:

- `cp themes/fret-baseline-only.json .fret/theme.json`
- `cargo run -p fret-demo --bin components_gallery`

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
- Default mitigation: when `metric.list.row_highlight_inset_y` is not provided by the theme,
  `fret-ui::VirtualList` now falls back to `metric.padding.sm / 4` (equivalent to the component-layer
  `Space::N0p5` fallback), so the highlight doesn’t visually “hug” adjacent rows as tightly.

Workarounds:

- Use `VirtualListRowHeight::Measured { min: ... }` for multi-line items.
- Prefer list-specific tokens (`metric.list.*`) for fine-tuning without forcing global padding changes.

Tracking:

- This should be solved by promoting sizing/density to a single component-level contract (MVP 47):
  `Size` (xs/sm/md/lg) + derived control metrics (list padding/row gaps/input heights), inspired by
  gpui-component’s `Size` + `StyleSized` helpers (`repo-ref/gpui-component/crates/ui/src/styled.rs`).
  - Fret now uses a shared `fret-ui-kit` list style helper to keep multi-line spacing consistent
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
- Track shadcn taxonomy parity separately: `docs/shadcn-declarative-progress.md`

### P0 (core app UI)

- `Command` (command UI): searchable list + groups + shortcuts + keyboard navigation
- `Popover` + `HoverCard` equivalents: anchored overlays and hover previews
- `Toast`/`Sonner`: transient notifications + stacking + timers (prototype implemented in `ecosystem/fret-ui-kit` today; intended shadcn surface lives in `ecosystem/fret-ui-shadcn`)
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
- Declarative layout semantics (fit-content vs fill, Flex sizing, min/max, wrap) so shadcn-style
  component composition does not require bespoke per-component layout code.

## Next steps (recommended)

1. Land the declarative authoring model (ADR 0028 + ADR 0039) as a real component composition path.
   - Rationale: shadcn/ui-style component parity is primarily about composition and consistent recipes; retained widgets
     alone will keep pushing “component needs” into runtime types.
2. Evolve virtualization toward GPUI-style composability:
   - keep a framework-level virtualized list primitive for performance and correctness,
   - but make row content composable (no fixed `VirtualListRow { text/secondary/trailing... }` schema),
   - migrate `Command` and at least one “tree-like” surface to prove the model.
3. Complete the runtime/components boundary tightening:
   - keep `fret-ui` as the runtime substrate + perf primitives,
   - ensure shadcn-like surfaces and policies stay in `ecosystem/fret-ui-shadcn` (built on `ecosystem/fret-ui-kit`),
   - remove remaining UI-kit-shaped runtime widgets once composition exists.

Update:

- The palette-style shell is prototype implemented and wired in `fret-demo --bin components_gallery`.
- The command palette list body is now rendered via **declarative composition** (rows are arbitrary
  element subtrees, not `VirtualListRow { text/secondary/trailing... }`), as an MVP 50 step toward a
  fully composable virtualization contract.
- The UI kit declarative panel now also demonstrates a composable, “rich row” virtualized list
  (leading + two-line label + trailing) built purely from declarative primitives, without relying on
  fixed row schemas.
