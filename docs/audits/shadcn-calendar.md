# shadcn/ui v4 Audit - Calendar


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Calendar` against the upstream shadcn/ui v4 docs and the
`base` example implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/calendar.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/calendar.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/calendar-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/calendar-basic.tsx`, `repo-ref/ui/apps/v4/examples/base/calendar-range.tsx`, `repo-ref/ui/apps/v4/examples/base/calendar-caption.tsx`, `repo-ref/ui/apps/v4/examples/base/calendar-presets.tsx`, `repo-ref/ui/apps/v4/examples/base/calendar-time.tsx`, `repo-ref/ui/apps/v4/examples/base/calendar-booked-dates.tsx`, `repo-ref/ui/apps/v4/examples/base/calendar-custom-days.tsx`, `repo-ref/ui/apps/v4/examples/base/calendar-week-numbers.tsx`, `repo-ref/ui/apps/v4/examples/base/calendar-rtl.tsx`, `repo-ref/ui/apps/v4/examples/base/calendar-hijri.tsx`
- Upstream foundation: `react-day-picker`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/calendar.rs`
- Related variants: `ecosystem/fret-ui-shadcn/src/calendar_range.rs`, `ecosystem/fret-ui-shadcn/src/calendar_multiple.rs`, `ecosystem/fret-ui-shadcn/src/calendar_hijri.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/calendar.rs`

## Audit checklist

### Authoring surface

- Pass: `Calendar::new(month, selected)` covers the common single-date authoring path with externally owned month state.
- Pass: `Calendar::new_controllable(cx, selected, default_selected)` covers the docs/gallery-style uncontrolled path without forcing callers to allocate a month model.
- Pass: `caption_layout(...)`, `number_of_months(...)`, `week_start(...)`, `show_week_number(...)`, `fixed_weeks(...)`, `locale(...)`, and disabled/hidden matchers cover the important recipe surface from the upstream docs/examples.
- Pass: `CalendarDayButton` exposes the day-cell customization seam that maps to upstream `components.DayButton`; richer per-day rendering should continue to land as a dedicated slot/day-button surface rather than a generic children API.
- Note: Range / multiple / Hijri variants intentionally live as dedicated components instead of overloading one generic builder, which keeps the contract surface explicit and typed.

### Layout & default-style ownership

- Pass: Recipe-owned defaults match the upstream component source: the calendar owns its inner chrome (`bg-background`, padding, day-cell chrome, caption/nav layout) and keeps the card/popover background transparent when hosted inside those slots.
- Pass: Example-level styling from the upstream docs stays caller-owned in Fret too: `rounded-lg border`, `p-0`, custom `--cell-size`, and field/popover width negotiation are applied in the gallery snippets rather than baked into `Calendar` defaults.
- Pass: The calendar root remains intrinsic-width (`w-fit` outcome) by default; the earlier gallery demo stretch came from page-level layout authored at the call site, not from a recipe bug.
- Pass: Multi-month responsive layout remains a mixed-ownership surface: the recipe owns month-row switching semantics, while host containers/popovers still own the outer width constraints.

### Gallery / docs parity

- Pass: The gallery now mirrors the upstream docs path more explicitly: `Demo`, `Usage`, `About`, `Date Picker`, `Persian / Hijri / Jalali Calendar`, `Selected Date (With TimeZone)`, then the upstream example sections through `RTL`, before appending Fret-only extensions.
- Pass: The `Selected Date (With TimeZone)` section is intentionally explanatory in Fret: the base calendar works with `time::Date`, so the JS `Date` offset issue described upstream does not require a calendar-level `timeZone` prop for date-only selection.
- Pass: The `Demo` and example snippets keep caller-owned styling (`rounded-lg border`, `p-0`, custom cell size, field/popover sizing) at the page/snippet layer instead of baking those constraints into `Calendar` defaults.
- Pass: Fret-only additions (`Date of Birth Picker`, `Natural Language Picker`, locale experiments, responsive semantics) remain after the upstream-aligned path so the page stays source-comparable.

## Validation

- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout web_vs_fret_layout_calendar_demo_day_grid_geometry_and_a11y_labels_match_web --status-level fail`
- `cargo nextest run -p fret-ui-gallery --lib gallery_calendar_core_examples_keep_upstream_aligned_targets_present --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --lib calendar_root_width_is_intrinsic_unless_caller_overrides_it --status-level fail`
