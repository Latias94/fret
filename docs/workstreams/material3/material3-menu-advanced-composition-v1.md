# Material 3 Menu Advanced Composition v1

Status: Complete
Last updated: 2026-06-01

This note scopes the bounded fearless-refactor batch for Material 3 Menu advanced composition.
The goal is to close concrete recipe gaps while keeping generic menu/submenu policy in ecosystem
infrastructure instead of `crates/fret-ui`.

## Refactor Brief

- Intent: move Material 3 Menu closer to the mature shadcn/Radix authoring bar by adding explicit
  structural groups and long-menu viewport behavior instead of relying on labels and oversized
  popover panels.
- Scope: `ecosystem/fret-ui-material3::{menu,dropdown_menu}`, `menu_state` tests, Material gallery
  menu snippet, and the Material completeness matrix.
- Deletion plan: replace ad-hoc top-level-only entry walking with recursive helpers for collection
  labels, disabled flags, rendering, close-on-select wrapping, and size estimation.
- Boundary plan: Material owns styling, tokens, test ids, and recipe API; existing `fret-ui-kit`
  menu submenu primitives remain the correct home for submenu policy. Core does not change.
- Testing plan: add grouped menu semantics/collection tests and a long `DropdownMenu` viewport
  scroll/clamp test; keep existing motion/focus gates.
- Risk plan: recursive groups must not skew `pos_in_set`/`set_size`; scroll wrapping must preserve
  initial menu focus and existing `.chrome`/item test ids.
- Workflow plan: direct bounded goal with proof doc, targeted tests, docs/gallery updates, and one
  Conventional Commit.
- Scale plan: medium direct task. Full Material submenu wiring is a follow-up because the reusable
  kit/shadcn submenu machinery exists but the Material recipe has not adopted it yet.

## Truth

- `MenuGroup` is a structural grouping primitive, not a selectable row.
- Group children participate in the parent menu collection metadata exactly as top-level items do.
- Labels and separators remain non-roving structural entries.
- Dropdown menus with content taller than the available or configured viewport mount a scrollable
  viewport part instead of sizing the popover to all rows.
- Submenu behavior is not moved into core; the existing `fret-ui-kit::primitives::menu::sub*`
  machinery remains the reference for a later Material submenu recipe pass.

## Proof Plan

- `menu_group_wraps_entries_without_skewing_collection_metadata`
- `dropdown_menu_long_content_uses_scrollable_material_viewport`
- Existing `menu_state` focus/motion/RTL tests remain green.

## Validation

- `cargo test -p fret-ui-material3 --features diagnostics --test menu_state menu_group_wraps_entries_without_skewing_collection_metadata -- --exact`
- `cargo test -p fret-ui-material3 --features diagnostics --test menu_state dropdown_menu_long_content_uses_scrollable_material_viewport -- --exact`
- `cargo test -p fret-ui-material3 --features diagnostics --test menu_state`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test menu_state`
- `cargo test -p fret-ui-material3 --lib dropdown_menu::tests::estimated_panel_size_uses_material_menu_intrinsic_bounds_and_padding -- --exact`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo check -p fret-ui-gallery`
- `cargo clippy -p fret-ui-material3 --features diagnostics --test menu_state --no-deps -- -D warnings`
- `python tools/check_layering.py`
- `python tools/check_workstream_catalog.py`
- `git diff --check`

## Residual Risk

- Material submenus remain a recipe wiring gap after this batch. The next dedicated pass should port
  the shadcn/Radix submenu model into Material row styling and overlay placement rather than
  inventing new core behavior.
