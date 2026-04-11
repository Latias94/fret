# M0 Branch-Site Audit — 2026-04-11

Status: active audit note

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

## Verdict

M0 can now be treated as closed.

The branch-site inventory is explicit enough to stop asking whether this follow-on is real.
There is one real higher-level gap, but it is narrower than the closed adaptive lane:

- repeated app-level desktop/mobile shell branching still exists,
- one recipe family already exposes explicit device-shell naming successfully,
- one app-shell provider already owns mobile inference explicitly,
- and several viewport reads remain recipe-internal alignment decisions rather than shared
  strategy-surface candidates.

That gives this lane a clear owner split.

## Inventory by bucket

### 1) App-local raw branch sites

These still hand-roll desktop/mobile shell swaps directly in app/gallery code:

- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
  - raw `viewport_width_at_least(...)`
  - desktop: `Popover`
  - mobile: `Drawer`
  - page already labels this as a gallery-only extra rather than the default docs lane
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
  - raw `viewport_width_at_least(...)`
  - desktop: `DropdownMenu`
  - mobile: `Drawer`
  - page already labels this as a focused Fret follow-up rather than the core docs path
- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
  - explicit paired `Dialog` vs `Drawer`
  - currently serves as docs-path proof rather than a shared abstraction

Interpretation:

- these are the strongest evidence that a higher-level device-shell strategy surface may be worth
  extracting above raw query reads
- today, however, they still live at app/gallery level and are not yet one shared contract

### 2) Recipe-owned explicit device-shell API

One family already uses explicit device-shell naming successfully:

- `ecosystem/fret-ui-shadcn/src/combobox.rs`
  - `Combobox::device_shell_responsive(bool)`
  - `Combobox::device_shell_md_breakpoint(Px)`
  - implementation stays explicitly viewport-driven and branches to `Drawer` only for the mobile
    device-shell path
- `apps/fret-ui-gallery/src/ui/pages/combobox.rs`
  - docs already teach this as an explicit follow-up lane instead of widening the default path

Interpretation:

- this is the strongest in-repo proof that explicit device-shell naming can work without reopening
  panel/container adaptive work
- it also implies that the next shared surface should probably look more like explicit
  recipe/policy naming than like a generic `responsive(...)` helper

### 3) Provider-owned app-shell device inference

One family should remain app-shell-only:

- `ecosystem/fret-ui-shadcn/src/sidebar.rs`
  - `SidebarProvider::is_mobile(bool)`
  - `SidebarProvider::is_mobile_breakpoint(Px)`
  - `width_mobile`, `open_mobile`, and the `Sheet` branch stay on the provider-owned app-shell lane
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
  - docs explicitly say this is app-shell/device-shell control, not generic panel adaptation
- `docs/audits/shadcn-sidebar.md`
  - same owner split is already source-aligned

Interpretation:

- sidebar is not the first extraction target for this lane
- keep it as the "do not widen this into editor rails" boundary anchor

### 4) Recipe-internal viewport alignment, not shared strategy

Some viewport reads exist, but they are internal layout/chrome decisions rather than reusable
desktop/mobile shell strategy:

- `ecosystem/fret-ui-shadcn/src/dialog.rs`
  - header/footer alignment and action layout follow upstream `sm:` semantics
- `ecosystem/fret-ui-shadcn/src/sheet.rs`
  - width/max-size policy follows viewport breakpoints inside the recipe
- related alert-dialog internal viewport reads follow the same class of concern

Interpretation:

- these do not justify a new higher-level helper by themselves
- keep them out of the first extraction unless the contract needs to wrap them intentionally

## Owner split recorded for M0

### Keep app-local for now

- `Breadcrumb responsive`
- `Date picker dropdowns`
- `Drawer responsive dialog`

Reason:

- they are still proof/inventory surfaces and do not yet prove one stable shared API shape

### Keep recipe-owned

- `Combobox::device_shell_responsive(...)`
- `Combobox::device_shell_md_breakpoint(...)`

Reason:

- this family already has explicit device-shell naming and a clear owned implementation path

### Keep provider/app-shell-owned

- `SidebarProvider::is_mobile(...)`
- `SidebarProvider::is_mobile_breakpoint(...)`
- `width_mobile` / `open_mobile`

Reason:

- these are app-shell controls, not generic overlay/device strategy helpers

### Keep recipe-internal

- `Dialog`/`Sheet`/`AlertDialog` viewport alignment reads that only implement upstream layout/chrome
  details

Reason:

- they are internal parity details, not current public strategy-surface candidates

## First extraction ranking

### Priority 1: `Popover` / `DropdownMenu` / `Drawer`-style device-shell switcher

Why first:

- repeated at least twice in app/gallery code today
- naturally sits above raw viewport queries
- does not force `Sidebar` or editor rail ownership questions back open

Likely owner:

- shared policy/helper in `fret-ui-kit`, with recipe-specific wrappers in `fret-ui-shadcn` only if
  the source-aligned API proves stable

### Priority 2: `Dialog` / `Drawer` pairing helper

Why second:

- there is good proof surface value, but less repetition today than the popover/dropdown family
- it may still be best kept explicit while the public naming rule is frozen

### Not a priority-1 target

- `Sidebar`
- raw recipe-internal viewport alignment in `Dialog` / `Sheet`

## What M0 resolves

- `DSS-010` current branch sites are inventoried
- `DSS-011` branch patterns are now classified into app-local vs recipe-owned vs provider-owned vs
  recipe-internal
- `DSS-012` owner split is explicit enough to move to M1 contract-freeze decisions
