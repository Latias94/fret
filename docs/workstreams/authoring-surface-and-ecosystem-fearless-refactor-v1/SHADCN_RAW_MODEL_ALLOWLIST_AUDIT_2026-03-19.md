# shadcn Raw Model Allowlist Audit — 2026-03-19

Status: source-policy audit note
Last updated: 2026-03-19

Related:

- `docs/shadcn-declarative-progress.md`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`
- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/REMAINING_SURFACE_SHRINK_AUDIT_2026-03-17.md`

## Why this note exists

Recent fearless-shrink batches converted many default shadcn entry points away from raw
`Model<_>` requirements and onto narrower bridges such as:

- `IntoBoolModel`
- `IntoOptionalBoolModel`
- `IntoTextValueModel`
- `IntoOptionalTextValueModel`
- `IntoFloatValueModel`
- `IntoOptionalFloatValueModel`
- `IntoFloatVecModel`
- `IntoSolarHijriMonthModel`
- `IntoU8ValueModel`

That creates a maintenance question:

> Which public `Model<_>` call sites in `fret-ui-shadcn` still represent intentional seams, and
> which ones would now be accidental surface creep?

This note records the selected allowlist that remains explicit today so future refactors do not
re-open the default lane by accident.

## Current conclusion

- Default first-contact surfaces should keep preferring typed wrappers and narrow conversion
  bridges.
- A smaller set of public `Model<_>` seams is still justified for controlled/uncontrolled roots,
  managed overlay ownership, source-aligned menu primitives, and controller/output handoff.
- New raw `Model<_>` entry points should update both this note and the source-policy gate in
  `surface_policy_tests.rs`; otherwise treat them as regressions.

## Audited seam classes

### 1. Managed overlay and controllable root seams

These remain explicit because the caller may already own open state, or because the type exposes a
controlled/uncontrolled root helper rather than the default compact constructor.

Audited files:

- `alert_dialog.rs`
- `calendar.rs`
- `calendar_multiple.rs`
- `calendar_range.rs`
- `context_menu.rs`
- `dialog.rs`
- `drawer.rs`
- `dropdown_menu.rs`
- `hover_card.rs`
- `popover.rs`
- `sheet.rs`
- `sidebar.rs`

Representative seams:

- `from_open(open: Model<bool>)`
- `new_controllable(..., open: Option<Model<bool>>, default_open: bool, ...)`
- `close_on_select(open: Model<bool>)`
- `open(...)` / `open_mobile(...)`

Rule:

- keep these explicit when the caller is intentionally managing root state,
- do not copy this pattern onto compact/default constructors when a bridge trait is sufficient.

### 2. Source-aligned menu, command, and part-level control seams

These remain model-backed because they mirror source-aligned checked/radio/query/value ownership or
because they are explicit builder-level escape hatches.

Audited files:

- `button.rs`
- `command.rs`
- `context_menu.rs`
- `dropdown_menu.rs`
- `menubar.rs`
- `tabs.rs`

Representative seams:

- menu checkbox/radio item constructors
- command query/highlight/value models
- `toggle_model(...)`
- `TabsRoot::new(model: Model<Option<Arc<str>>>)`

Rule:

- keep these seams explicit when they truly represent owned control state,
- prefer shrinking first-contact helpers before adding more raw part-level constructors.

### 3. Output/controller handoff seams

These seams exist because the component exports runtime snapshots, API handles, or owned state that
another controller/recipe consumes.

Audited files:

- `carousel.rs`
- `chart.rs`
- `data_grid_canvas.rs`
- `data_table.rs`
- `data_table_recipes.rs`
- `date_picker_with_presets.rs`
- `extras/banner.rs`

Representative seams:

- `output_model(...)`
- `api_snapshot_model(...)`
- `api_handle_model(...)`
- `faceted_filter_counts(...)`
- `preset_value_model(...)`

Rule:

- keep model-backed handoff where another controller genuinely reads or mutates shared runtime
  state,
- avoid promoting these controller seams into default cookbook-style discovery lanes.

### 4. Asset/media and explicit state-carrier seams

These are intentionally explicit because the caller is supplying a state carrier rather than a
static asset/value.

Audited files:

- `avatar.rs`
- `media_image.rs`

Representative seams:

- `AvatarImage::model(...)`
- `AvatarFallback::when_image_missing_model(...)`
- `MediaImage::model(...)`

Rule:

- keep the static constructor as the default lane,
- keep model-backed media constructors explicit and clearly named.

### 5. Narrow specialized surfaces that should reopen only with evidence

This bucket is now smaller than when the note was first written:

- `InputGroup::new(...)`
- `SidebarInput::new(...)`
- `CalendarHijri::new(...)`
- `Rating::new(...)`
- `DataTableGlobalFilterInput::new(...)`
- `DataTableViewOptionItem::new(...)`
- `DataTableViewOptions::new(...)`

have already moved onto narrow bridge traits and are no longer part of the raw-model allowlist.

What remains in this class should still not be treated as proof that more raw `Model<_>` surface is
desirable.

Rule:

- reopen these only with fresh authoring evidence, not because a raw model noun merely still
  exists in source.

## Guardrail

`ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs` now carries a selected-file allowlist gate
for the public function signatures in this note.

That gate is intentionally selected-file scoped:

- default-lane constructors/wrappers are already protected by the existing typed-surface tests,
- this note focuses on the remaining raw-model seams that are still intentional,
- future changes should update the note and the gate in the same patch.

## Practical maintainer rule

Before adding another public `Model<_>` signature to `fret-ui-shadcn`, ask:

1. Is this truly a controlled/managed/controller handoff seam?
2. Would a narrow bridge trait preserve the same capability on the default lane?
3. Will first-party docs/examples benefit from seeing this raw model surface directly?

If the answer to (2) is "yes" or the answer to (3) is "no", do not widen the public raw-model
budget.
