# shadcn/ui v4 Audit - Calendar


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Calendar` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/calendar.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/calendar.tsx`
- Upstream foundation: `react-day-picker`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/calendar.rs`
- Related variants: `ecosystem/fret-ui-shadcn/src/calendar_range.rs`, `ecosystem/fret-ui-shadcn/src/calendar_multiple.rs`, `ecosystem/fret-ui-shadcn/src/calendar_hijri.rs`

## Audit checklist

### Authoring surface

- Pass: `Calendar::new(month, selected)` covers the common single-date authoring path.
- Pass: `caption_layout(...)`, `number_of_months(...)`, `week_start(...)`, `show_week_number(...)`, `locale(...)`, and disabled matchers cover the important recipe surface from the upstream docs.
- Pass: Range / multiple / Hijri variants live as dedicated components instead of overloading one generic builder, which keeps the contract surface explicit.
- Note: `Calendar` already exposes the knobs it needs, so Fret intentionally does not add a generic `compose()` builder here.

### Contract notes vs upstream

- Note: Upstream is built on `react-day-picker` and typically uses JS `Date`; Fret uses `time::Date`, which avoids timezone-offset selection drift.
- Note: Fret's calendar contract is intentionally more explicit around date typing and variant separation than the upstream web composition.

### Visual / interaction parity

- Pass: The gallery already covers the major shadcn examples: single-date, range, month/year selector, date-of-birth picker, presets, custom cell size, and Hijri.
- Pass: Multi-month, week-number, locale, and responsive semantics examples extend the upstream docs with framework-specific regression coverage.

## Validation

- `cargo test -p fret-ui-shadcn --lib calendar`
- `cargo check -p fret-ui-gallery`