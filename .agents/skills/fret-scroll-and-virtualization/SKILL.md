---
name: fret-scroll-and-virtualization
description: Scrolling and large lists in Fret (`ScrollHandle`, `VirtualList`, `VirtualListScrollHandle`). Use when building fast tables/trees/palettes, implementing scroll-to-item, or debugging keyed identity / offset regressions.
---

# Fret scrolling and virtualization

## When to use

- Rendering a large collection (tables, trees, palettes, timelines) where “render everything” is too slow.
- You need **scroll-to-item**, “keep selection visible”, or “scroll to bottom”.
- You see state bugs like “caret/hover sticks to the wrong row after reorder”.
- You’re chasing regressions in scroll clamping, overscan, or item measurement.

## Core concepts (what matters)

- **`ScrollHandle` is state + revision**: viewport size, content size, offset. Mutations bump a revision so layout/paint can react.
- **`VirtualList` renders a window**: the runtime decides which items are visible (+ overscan) and asks you to render only those.
- **Stable identity is non-negotiable**:
  - Use `ElementContext::virtual_list_keyed(...)` and provide a stable `ItemKey` from your data model (not the row index).
  - This prevents element-local state (selection/caret/hover/scroll handles) from “sticking to positions”.
- **Measurement modes** (`VirtualListOptions.measure_mode`):
  - `Fixed`: best perf when row height is constant.
  - `Measured`: supports variable heights but requires careful invalidation when content changes.
  - `Known`: you provide per-row heights via `known_row_height_at`.
- **`items_revision` is your “I changed something structural” knob**:
  - Bump it when the list’s ordering/keys/heights meaningfully change, so metrics/key caches can reset safely.

## Quick start

### Virtual list with stable keys

```rust
use fret_core::Px;
use fret_ui::element::{AnyElement, VirtualListOptions};
use fret_ui::{ElementContext, UiHost, VirtualListScrollHandle};
use fret_ui_kit::prelude::*;

pub struct Row {
    pub id: u64,
    pub title: String,
}

pub fn rows_list<H: UiHost>(cx: &mut ElementContext<'_, H>, rows: &[Row]) -> AnyElement {
    // Keep the scroll handle stable across frames (element-local state).
    let scroll = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

    // Fixed height is the fast path for editor-style lists.
    let options = VirtualListOptions::fixed(Px(28.0), 8).keep_alive(16);

    cx.virtual_list_keyed(
        rows.len(),
        options,
        &scroll,
        |i| rows[i].id, // stable key (NOT i)
        |cx, i| ui::text(cx, rows[i].title.clone()).into_element(cx),
    )
}
```

### Scroll-to-item (programmatic)

- Call `VirtualListScrollHandle::scroll_to_item(index, strategy)` (or `scroll_to_bottom()`).
- The request is deferred and consumed by the runtime; avoid reissuing the same request every frame.

## Common pitfalls

- **Using the row index as identity** (`key_at: |i| i as u64`) for reorderable data.
- **Variable-height rows with `Fixed` mode** (breaks visibility/culling assumptions).
- **Forgetting to bump `items_revision`** when the meaning of indices/keys changes.
- **Overusing `keep_alive`** (can retain too much subtree state, increasing memory and invalidation cost).

## Regression gates (recommended defaults)

- Add a small invariant test for:
  - “scroll offset clamps to content bounds” (baseline already exists in `crates/fret-ui/src/scroll.rs`).
  - “duplicate keys are detected / warned” (key collisions are catastrophic for state).
- Add a `tools/diag-scripts/*.json` scenario that:
  - scrolls, selects an item, reorders data, then asserts the selected row is still the same `test_id`.
- Capture diagnostics with `FRET_DIAG=1` and inspect `virtual_list_*` records in the bundle.

## References (start here)

- ADRs:
  - `docs/adr/0042-virtualization-and-large-lists.md`
  - `docs/adr/0047-virtual-list-data-source-and-stable-item-keys.md`
- Code entry points:
  - `crates/fret-ui/src/scroll.rs` (`ScrollHandle`, `VirtualListScrollHandle`)
  - `crates/fret-ui/src/virtual_list.rs` (metrics/windowing)
  - `crates/fret-ui/src/elements/cx.rs` (`virtual_list_keyed*` helpers)
  - `ecosystem/fret-ui-shadcn/src/data_table.rs` (real usage)
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (virtual list diagnostics surface)
