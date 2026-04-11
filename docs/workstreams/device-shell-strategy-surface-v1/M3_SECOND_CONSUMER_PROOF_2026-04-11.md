# M3 Second Consumer Proof — 2026-04-11

Status: accepted proof note

Related:

- `TARGET_INTERFACE_STATE.md`
- `M2_FIRST_EXTRACTION_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## Question

Does the crate-local `fret-ui-kit` device-shell helper now have enough real consumer evidence to
close this lane without reopening facade promotion or wrapper growth?

## Short answer

Yes.

The current shipped helper surface is now proven across two materially different app-facing
consumers while the lane still keeps the explicit non-helper boundaries visible.

## Current proof set

### 1) `Date Picker` proves `Popover` vs `Drawer`

`apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs` now uses
`device_shell_switch(...)` while keeping:

- desktop `Popover`
- mobile `Drawer`

visible at the call site.

### 2) `Breadcrumb` proves `DropdownMenu` vs `Drawer`

`apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs` now uses:

- `device_shell_mode(...)` for the device-shell-derived tail-width choice
- `device_shell_switch(...)` for the actual desktop/mobile shell branch

while keeping:

- desktop `DropdownMenu`
- mobile `Drawer`

visible at the call site.

### 3) `Dialog` vs `Drawer` remains intentionally explicit

`apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs` still stays outside the shared
helper and remains the explicit proof surface for a docs-first paired dialog/drawer example.

### 4) Recipe-owned and app-shell-owned boundaries remain intact

The lane still keeps:

- `Combobox::device_shell_responsive(...)` as the explicit recipe-owned wrapper example
- `SidebarProvider::is_mobile(...)` / `is_mobile_breakpoint(...)` as the app-shell-owned boundary

instead of collapsing everything into one generic adaptive surface.

## Decision

Treat the current evidence as sufficient for this lane's narrow question.

This means:

1. two real app-facing helper consumers are now enough to prove the `fret-ui-kit` owner split,
2. the lane does not need a third demo before closeout,
3. facade promotion to `fret::adaptive` remains deferred,
4. recipe-owned wrapper growth remains a separate future question,
5. and the next reopen trigger should be a new bounded follow-on rather than more proof
   duplication inside this lane.
