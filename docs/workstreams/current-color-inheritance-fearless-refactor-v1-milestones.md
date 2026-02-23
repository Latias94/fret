# Foreground inheritance (`currentColor`) (fearless refactor v1) — Milestones

Last updated: 2026-02-23

## M0 — Agreement on the surface (design)

Exit criteria:

- `currentColor` provider API is defined and documented.
- Resolution order is explicit: explicit color → inherited `currentColor` → theme fallback.
- Ownership/layering is confirmed (kit glue + shadcn policy).

## M1 — Foundation landed (provider + first adopters)

Exit criteria:

- Provider module exists and is tested.
- Icons inherit `currentColor` when available.
- `shadcn::Button` provides its computed `fg` to descendants.
- At least one minimal unit test proves inheritance in practice.

## M2 — Menu family parity (most visible UX win)

Exit criteria:

- Menu-like rows provide a consistent `fg` to both text and icons:
  - `DropdownMenuItem`
  - `SelectItem`
  - `CommandItem`
- At least one diag script captures a menu with leading icons and checks stable visibility.

## M3 — Text inheritance (remove boilerplate)

Exit criteria:

- A minimal, well-scoped text default inherits `currentColor` in the common authoring path.
- Button/menu examples no longer need to manually `.text_color(fg)` for basic cases.
- One unit test + one gallery/diag gate locks the behavior.

## M4 — Cleanup + evidence

Exit criteria:

- Gallery pages remove redundant token-threading where inheritance is sufficient.
- Workstream tracking table is updated to show closure status and evidence anchors.
- At least two regression artifacts exist:
  - one unit test (kit)
  - one diag script (gallery)

