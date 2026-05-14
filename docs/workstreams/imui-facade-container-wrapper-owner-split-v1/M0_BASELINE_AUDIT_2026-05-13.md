# M0 Baseline Audit - 2026-05-13

Status: baseline captured

## Current Source Shape

Before M1:

| File | Baseline |
| --- | ---: |
| `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1275 lines before M1 |
| `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers.rs` | n/a |

The structural container wrapper cluster still lived in `facade_writer.rs`:

- `horizontal(...)` / `horizontal_with_options(...)`
- `menu_bar(...)` / `menu_bar_with_options(...)`
- `tab_bar(...)` / `tab_bar_with_options(...)`
- `vertical(...)` / `vertical_with_options(...)`
- `grid(...)` / `grid_with_options(...)`
- `table(...)` / `table_with_options(...)`
- `virtual_list(...)` / `virtual_list_with_options(...)`
- `scroll(...)` / `scroll_with_options(...)`
- `child_region(...)` / `child_region_with_options(...)`

## Decision

Move only those inherent wrappers to a private `facade_writer/container_wrappers.rs` owner. Leave
trait methods, container behavior, table/tab/child-region policy, and public paths unchanged.

## Non-Goals

- No public method renames.
- No `fret-imui` dependency or public surface changes.
- No `crates/fret-ui` runtime contract changes.
- No new table, tab, virtual-list, scroll, or child-region behavior.
