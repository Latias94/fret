# Foreground inheritance (late binding) v2 - TODO

Last updated: 2026-02-24

This TODO is scoped to the v2 “fearless refactor” workstream:
paint-time inherited foreground (`currentColor` / `IconTheme`) via `ForegroundScope`.

See:
- Workstream overview: `docs/workstreams/foreground-inheritance-late-binding-v2/foreground-inheritance-late-binding-v2.md`
- Milestones: `docs/workstreams/foreground-inheritance-late-binding-v2/foreground-inheritance-late-binding-v2-milestones.md`

## Tracking table

| Item | Layer | Priority | Status | Evidence anchors |
| --- | --- | --- | --- | --- |
| Add paint-time regression test (icon + text inherit fg) | `crates/fret-ui` | P0 | In progress | `crates/fret-ui/src/declarative/tests/foreground_inheritance.rs` |
| Make paint-cache key depend on inherited fg | `crates/fret-ui` | P0 | Landed | `crates/fret-ui/src/tree/paint_cache.rs` |
| Migrate shadcn hosts to `ForegroundScope` wrappers | `ecosystem/fret-ui-shadcn` | P0 | In progress | `ecosystem/fret-ui-shadcn/src/*` |
| Remove build-time `inherited_current_color` reads from ecosystem leaves | `ecosystem/fret-ui-kit` | P0 | In progress | `ecosystem/fret-ui-kit/src/ui.rs` |
| Audit remaining “dark background + icon” surfaces in gallery | `apps/fret-ui-gallery` | P1 | Not started | (add diag scripts + anchors) |
| Deprecate/phase out v1 provider APIs in authoring docs | `docs/` | P2 | Not started | `docs/workstreams/current-color-inheritance-fearless-refactor-v1/current-color-inheritance-fearless-refactor-v1.md` |

## Checklist (by area)

### Mechanism (`crates/fret-ui`)

- [ ] Ensure `ForegroundScope` is layout/measure/semantics transparent in all declarative passes.
- [ ] Add small unit tests for `PaintCacheKey` (different fg => different key).
- [ ] Consider extending `PaintStyleState` beyond foreground (text-style inheritance) as a follow-up workstream, not as part of this v2 landing.

### Ecosystem leaves (`ecosystem/fret-ui-kit`)

- [ ] Ensure icon/spinner/text helpers default to late-binding (do not bake inherited color during `into_element(cx)`).
- [ ] Update/replace any tests that asserted build-time inheritance (switch to “late-binding” assertions or move the assertion to `crates/fret-ui` paint-time tests).
- [ ] Expose `current_color::scope_children(...)` (or an equivalent helper) in the authoring prelude for easy migration.

### shadcn hosts (`ecosystem/fret-ui-shadcn`)

- [ ] Ensure every host that previously installed “current color” now installs a `ForegroundScope` boundary.
- [ ] Prefer installing scope at the narrowest correct boundary (e.g. inside a menu item row, not around the entire menu content).
- [ ] Keep explicit per-leaf overrides working (explicit color wins).

### Regression gates (`tools/diag-scripts`)

- [ ] Add/update screenshot scripts covering:
  - [ ] Button variants on dark backgrounds with leading/trailing icons.
  - [ ] Menubar/context-menu rows on dark backgrounds.
  - [ ] Loading states (spinner inherits fg in destructive/secondary/etc).
- [ ] Run scripts with fixed frame delta when motion is involved.

