# shadcn Trigger/Policy Seams Audit — 2026-03-28

Status: focused surface audit note
Last updated: 2026-03-28

Related:

- `docs/shadcn-declarative-progress.md`
- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/SHADCN_RAW_MODEL_ALLOWLIST_AUDIT_2026-03-19.md`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`

## Why this note exists

After the `imui` fearless cleanup, one practical follow-up question remained:

> Which shadcn trigger/policy seams are still real concepts, and which ones are only old
> compatibility residue?

This note records the current answer so future surface work can keep deleting obvious residue
without reopening justified component-policy seams.

## Current conclusion

The remaining shadcn trigger/policy seams split into three buckets:

1. keep: component-owned trigger hooks such as `Button::toggle_model(...)`
2. keep: explicit advanced managed-open seams such as `Popover::from_open(...)`
3. delete: compatibility aliases that only duplicate one of the explicit advanced seams

The 2026-03-28 audit found one clear delete-now case:

- `ContextMenu::new(open)` was a compatibility alias over `ContextMenu::from_open(open)` and had
  no remaining in-tree callers, so it was removed.

## Bucket 1: keep component-owned trigger hooks

Representative seams:

- `Button::toggle_model(...)`
- `InputGroupButton::toggle_model(...)`
- `Calendar::close_on_select(...)`
- `CalendarMultiple::close_on_select(...)`
- `CalendarRange::close_on_select(...)`

Why these stay:

- they attach activation policy to a trigger-like or composite component,
- they are not generic form-field constructors,
- first-party callers still use them for explicit open/close choreography,
- they now accept the narrower `impl IntoBoolModel` bridge instead of forcing raw `Model<bool>`
  ownership at the public call site,
- and the verb communicates trigger behavior rather than widget value ownership.

Representative evidence:

- `ecosystem/fret-ui-shadcn/src/button.rs`
- `ecosystem/fret-ui-shadcn/src/input_group.rs`
- `ecosystem/fret-ui-shadcn/src/calendar.rs`
- `ecosystem/fret-ui-shadcn/src/calendar_multiple.rs`
- `ecosystem/fret-ui-shadcn/src/calendar_range.rs`
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dob.rs`
- `apps/fret-ui-gallery/src/ui/snippets/dialog/demo.rs`

## Bucket 2: keep explicit advanced managed-open seams

Representative seams:

- `Popover::from_open(...)`
- `DropdownMenu::from_open(...)`
- `ContextMenu::from_open(...)`
- `HoverCard::open(...)`
- `Sidebar::open(...)`
- `Sidebar::open_mobile(...)`

Why these stay:

- they are explicit “caller already owns state” lanes,
- they are intentionally not the first-contact constructor path,
- and source-policy tests already distinguish them from the default typed root constructors.

Representative evidence:

- `ecosystem/fret-ui-shadcn/src/popover.rs`
- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- `ecosystem/fret-ui-shadcn/src/hover_card.rs`
- `ecosystem/fret-ui-shadcn/src/sidebar.rs`

## Bucket 3: delete compatibility aliases

Rule:

- if a public API adds no new concept and only forwards to an explicit advanced seam, delete it
  instead of preserving an extra root name.

Landed result from this audit:

- deleted `ContextMenu::new(open)` because it only forwarded to `ContextMenu::from_open(open)`
  and had no remaining in-tree callers.

Evidence:

- `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`

## Next shrink candidates

Potential future work, but not auto-landed by this audit:

- keep checking for any further `Compatibility alias` comments or zero-caller forwarding roots in
  `fret-ui-shadcn`

These should be treated as separate evidence-backed batches, not as justification to rename the
component-policy verbs themselves.
