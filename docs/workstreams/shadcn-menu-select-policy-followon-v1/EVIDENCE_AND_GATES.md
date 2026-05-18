# shadcn Menu/Select Policy Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-17

## Baseline Observation

Observed during ASF-060:

```bash
cargo test -p fret-ui-shadcn --locked --test select_keyboard_navigation -j 1
```

Result:

- Failed after running the test.
- Failure: pointer-open + ArrowDown + Enter selected `"apple"`; the test expected `"banana"`.
- Interpretation: this is an unresolved select interaction contract, not a layering failure.

## SMS-010 Select Pointer-Open + ArrowDown

Status: Done

Decision:

- shadcn v4 Select follows Radix Select semantics because the `new-york-v4` component wraps
  `@radix-ui/react-select`.
- Radix pointer-open focuses the selected item after the content is positioned. When no value is
  selected, Radix's item callback records the first valid item as the selected-item fallback.
- Fret keeps tree focus on the listbox container for pointer-open so it can preserve its
  active-descendant model, but the active descendant should initialize to the same selected/first
  enabled row. With no selected value, pointer-open starts at Apple and the first ArrowDown advances
  to Banana.
- Base UI is not the primary shadcn truth source, but it corroborates the same headless direction:
  selected/highlighted item state is the navigation anchor, and pointer-open with no value highlights
  the first item instead of leaving navigation unanchored.

Source anchors:

- `F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/registry/new-york-v4/ui/select.tsx`
  imports and composes `@radix-ui/react-select`.
- `F:/SourceCodes/Rust/fret/repo-ref/primitives/packages/react/select/src/select.tsx`
  opens mouse Select on trigger pointer-down through `handleOpen(event)`.
- `F:/SourceCodes/Rust/fret/repo-ref/primitives/packages/react/select/src/select.tsx`
  calls `focusSelectedItem()` after content positioning.
- `F:/SourceCodes/Rust/fret/repo-ref/primitives/packages/react/select/src/select.tsx`
  sets `selectedItem` from the selected value or first valid item in `itemRefCallback`.
- `F:/SourceCodes/Rust/fret/repo-ref/primitives/packages/react/select/src/select.tsx`
  handles ArrowDown by starting after the currently focused item.
- `F:/SourceCodes/Rust/fret/repo-ref/base-ui/packages/react/src/select/root/SelectRoot.tsx`
  wires list navigation with `activeIndex` and `selectedIndex`.
- `F:/SourceCodes/Rust/fret/repo-ref/base-ui/packages/react/src/select/item/SelectItem.tsx`
  syncs the registered selected item into `selectedIndex`.
- `F:/SourceCodes/Rust/fret/repo-ref/base-ui/packages/react/src/select/root/SelectRoot.test.tsx`
  covers pointer-open with no selected value highlighting the first item when item-trigger alignment
  is active.

In-tree artifacts:

- `ecosystem/fret-ui-shadcn/src/select.rs`: no longer clears the initial active row only because the
  trigger was opened by pointer.
- `ecosystem/fret-ui-shadcn/src/select.rs`: unit coverage asserts pointer-open active descendant is
  Apple before ArrowDown and Banana after ArrowDown.
- `ecosystem/fret-ui-shadcn/tests/select_keyboard_navigation.rs`: integration coverage preserves
  the original repro, ArrowDown + Enter after pointer-open selects Banana and closes.

Validated gates:

```bash
cargo test -p fret-ui-shadcn --locked --test select_keyboard_navigation -j 1
cargo test -p fret-ui-shadcn --locked --lib select_pointer_open_arrow_down_moves_active_descendant -j 1
cargo test -p fret-ui-shadcn --locked --lib select_grouped_pointer_open_arrow_down_moves_active_descendant -j 1
cargo test -p fret-ui-headless --locked --lib entry_focus -j 1
cargo test -p fret-ui-kit --locked --lib initial_focus -j 1
```

Residual risk:

- This slice resolves Select's initial pointer-open active descendant. It does not yet extract shared
  roving/typeahead or submenu intent policy across dropdown/context/menubar surfaces; that remains
  a separate SMS-020 decision.

## Gate Set

```bash
cargo test -p fret-ui-headless --locked --lib entry_focus -j 1
cargo test -p fret-ui-kit --locked --lib initial_focus -j 1
cargo test -p fret-ui-shadcn --locked --test select_keyboard_navigation -j 1
cargo test -p fret-ui-shadcn --locked --test dropdown_menu_keyboard_navigation -j 1
cargo test -p fret-ui-shadcn --locked --test context_menu_keyboard_navigation -j 1
python tools/check_layering.py
```

Use narrower gates while iterating, but record exact commands next to each shipped slice.

## Closeout

Status: Done

Closeout decision:

- `SMS-010` resolves the only concrete failing contract that opened this follow-on.
- `SMS-020` is intentionally deferred because no additional shared policy owner was proven by fresh
  cross-surface evidence in this lane.
- Future roving/typeahead, submenu intent, or dismissal/focus-restore cleanup should open a narrower
  follow-on with one source-backed repro and one focused gate.

Final closeout gates:

```bash
cargo fmt --package fret-ui-shadcn
cargo test -p fret-ui-shadcn --locked --test select_keyboard_navigation -j 1
cargo test -p fret-ui-shadcn --locked --lib select_pointer_open_arrow_down_moves_active_descendant -j 1
cargo test -p fret-ui-shadcn --locked --lib select_grouped_pointer_open_arrow_down_moves_active_descendant -j 1
cargo test -p fret-ui-headless --locked --lib entry_focus -j 1
cargo test -p fret-ui-kit --locked --lib initial_focus -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```

## Evidence Anchors

- `docs/workstreams/architecture-surface-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- `ecosystem/fret-ui-headless/src/entry_focus.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/root.rs`
- `ecosystem/fret-ui-kit/src/primitives/select.rs`
- `ecosystem/fret-ui-shadcn/src/select.rs`
- `ecosystem/fret-ui-shadcn/tests/select_keyboard_navigation.rs`
- `docs/workstreams/shadcn-menu-select-policy-followon-v1/JOURNAL/2026-05-17-sms-010.md`
- `docs/workstreams/shadcn-menu-select-policy-followon-v1/CLOSEOUT_AUDIT_2026-05-17.md`
