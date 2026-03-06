# Foreground Style Context (Fearless Refactor v1) — Milestones

Status: In progress

Related:

- Design: `docs/workstreams/foreground-style-context-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/foreground-style-context-fearless-refactor-v1/TODO.md`

Current snapshot (2026-03-06):

- `M2` is effectively landed for foreground-only v1 through an attached inherited-foreground carrier
  plus compatibility support for legacy `ForegroundScope`.
- `M3` is in progress: `input_group`, `card`, `alert`, `badge`, `menubar`, `context_menu`, and
  selected `fret-ui-ai` surfaces have been migrated off synthetic foreground wrappers.
- `dropdown_menu`, `select`, and `tabs` have now been audited and locked with targeted regression
  tests that prove inherited foreground attaches to real subtree roots without synthetic wrapper
  nodes.
- `context_menu` and `menubar` are now also covered by targeted menu-row regression tests, so the
  menu-family audit has explicit evidence for wrapper-free inherited icon foreground on existing
  subtree nodes.
- `message`, `task`, and `alert` now have explicit regression coverage for wrapper-free inherited
  foreground, and `message` no longer lowers a single-root content stack through transitional
  `scope_children(...)`.
- `input_group` is now fully covered for the v1 foreground slice: addon rows and button-content
  roots both attach inherited foreground directly to existing subtree roots without synthetic
  wrappers.
- Follow-up layout audit work has now locked `radio_group` long-label behavior with a targeted
  regression test: the default label text keeps word wrapping while shrinking inside the item row
  via explicit grow + `min-w-0`, preventing the one-line overflow drift seen during the audit.
- The same wrapped-label audit has now been extended into `fret-ui-ai` header surfaces:
  `agent`, `sandbox`, and `tool` all had icon/label/badge rows tightened with explicit shrink-
  friendly constraints, and each surface now has a targeted regression test that locks the label
  text to `wrap: Word` plus `flex_grow(1.0)` + `min_w_0()`.
- The audit has also reached `fret-ui-ai/sources_block`: both per-source title/link rows and the
  collapsible trigger label now keep their truncation semantics while shrinking within horizontal
  rows, with targeted regression tests covering the item-row and trigger-row paths.
- `fret-ui-ai/web_preview` console rows are now also covered: timestamp + message log rows keep
  word-wrap behavior without overflowing beside the timestamp, and the fix is locked with a
  targeted regression test.
- The Material 3 chip audit now also covers natural-width chip chrome: `chip`,
  `suggestion_chip`, `filter_chip`, and `input_chip` all keep one-line ellipsis semantics while
  remaining shrinkable within icon-bearing chip rows via explicit `min-w-0` + `flex-shrink: 1`,
  without forcing fill width.
- The Material 3 button audit now also covers natural-width button chrome: icon-bearing
  buttons keep one-line clip semantics while remaining shrinkable between icon slots via explicit
  `min-w-0` + `flex-shrink: 1`, without forcing fill width on the label.
- The Material 3 extended FAB audit now also covers natural-width FAB chrome:
  icon-bearing extended FABs keep one-line clip semantics while remaining shrinkable beside the
  icon via explicit `min-w-0` + `flex-shrink: 1`, without forcing fill width on the label.
- The Material 3 select listbox audit now also covers one-line and two-line option
  rows: item labels and supporting text keep clip semantics while shrinking inside icon-bearing
  listbox rows via explicit fill-width body slots plus `min-w-0`.
- The Material 3 date picker audit now also covers the modal panel headline:
  `Select date` keeps ellipsis semantics within the fixed dialog width via explicit fill width
  plus `min-w-0`, alongside the already-covered month header title.
- `fret-ui-ai/inline_citation` hover-card content is now covered too: fixed-width cards keep title
  / URL truncation and quote wrapping stable because the content stacks and text/link surfaces now
  explicitly fill available width and opt into `min-w-0`.
- `fret-ui-material3/top_app_bar` is now covered too: single-row and two-row titles keep
  ellipsis semantics within their horizontal slots because the title text and its flex wrappers
  now explicitly fill available width and opt into `min-w-0`.
- `fret-ui-material3/select` trigger values are now covered too: selected text keeps ellipsis
  semantics within the leading-icon + chevron row because the value text and left content slot now
  explicitly fill available width and opt into `min-w-0`.
- `fret-ui-material3/list` item labels are now covered too: one-line labels keep stable clip
  semantics between leading/trailing icon slots because the label text now explicitly fills the
  available row width and opts into `min-w-0`.
- `fret-ui-material3/navigation_drawer` labels are now covered too: labels shrink beside optional
  badges because the left content slot and label text both explicitly fill available width and opt
  into `min-w-0`.
- `fret-ui-material3/tabs` labels are now covered too: non-scrollable primary tabs keep stable
  one-line clip semantics because the label text explicitly fills the equal-width tab slot and opts
  into `min-w-0`.
- `fret-ui-material3/menu` item labels are now covered too: one-line menu rows keep stable
  clip semantics because the label text explicitly fills the available row width and opts into
  `min-w-0`.
- `fret-ui-material3/navigation_bar` labels are now covered too: bottom-nav labels keep one-line
  clip semantics within equal-width item slots because the label text explicitly fills the slot and
  opts into `min-w-0`.
- `fret-ui-material3/navigation_rail` labels are now covered too: rail labels keep one-line clip
  semantics within fixed-width item slots because the label text explicitly fills the slot and opts
  into `min-w-0`.
- `fret-ui-material3/date_picker` month titles are now covered too: header titles keep
  centered ellipsis semantics between the previous/next buttons because the title text explicitly
  fills the remaining header width and opts into `min-w-0`.
- `fret-ui-material3/time_picker` titles are now covered too: dialog titles keep ellipsis
  semantics beside the mode-toggle button because the title text explicitly fills the remaining
  header width and opts into `min-w-0`.
- `fret-ui-material3/segmented_button` labels are now covered too: equal-width segment
  labels keep centered one-line clip semantics because the label text explicitly fills the segment
  slot and opts into `min-w-0`.
- The `fret-ui-material3` chip family is now covered too: `chip`, `suggestion_chip`,
  `filter_chip`, and `input_chip` all keep one-line ellipsis semantics while remaining shrinkable
  inside natural-width chip chrome because the label text now explicitly opts into `min-w-0` plus
  `flex-shrink: 1` without being forced to fill width.
- `fret-ui-material3/button` is now covered too: icon-bearing buttons keep one-line clip
  semantics while remaining shrinkable inside natural-width button chrome because the label text
  now explicitly opts into `min-w-0` plus `flex-shrink: 1` without being forced to fill width.
- `fret-ui-material3/fab` is now covered too: icon-bearing extended FABs keep one-line
  clip semantics while remaining shrinkable inside natural-width floating-action chrome because the
  label text now explicitly opts into `min-w-0` plus `flex-shrink: 1` without being forced to fill width.
- `fret-ui-material3/select` listbox items are now covered too: one-line and two-line
  option rows keep label/supporting-text clip semantics while shrinking inside icon-bearing rows
  because the text surfaces and body slots now explicitly opt into fill width plus `min-w-0`.
- `fret-ui-material3/date_picker` now also covers the modal dialog headline: `Select date`
  keeps ellipsis semantics within the fixed dialog width because the headline text now explicitly
  opts into fill width plus `min-w-0`, complementing the month-title header coverage.

---

## M0 — Problem locked and risk surface audited

Exit criteria:

- The current `ForegroundScope` contract is described clearly enough that reviewers can explain why
  it is risky.
- All `scope_children(...)` and direct `foreground_scope(...)` call sites are inventoried.
- High-risk migration candidates are identified and grouped by area.

---

## M1 — Design accepted for inherited foreground v1

Exit criteria:

- The preferred design is explicit: inherited foreground is context, not a synthetic layout
  fragment.
- Ownership is clear across `crates/fret-ui`, `fret-ui-kit`, and `fret-ui-shadcn`.
- The compatibility stance for `ForegroundScope` is explicit.
- It is clear whether v1 is foreground-only or also includes broader text-style inheritance.

---

## M2 — Mechanism path landed behind compatibility

Exit criteria:

- `crates/fret-ui` has a mechanism path for inherited foreground that does not require an
  author-facing wrapper node.
- Real subtree roots can install inherited foreground.
- Core consumers resolve foreground by the documented precedence contract.
- Legacy `ForegroundScope` still works during migration.

---

## M3 — First migration slice landed on high-risk surfaces

Exit criteria:

- At least the first high-risk recipe surfaces no longer depend on `scope_children(...)`.
- Wrapped-text and overlay-content regressions are gated.
- The migration path is demonstrated in one or more representative recipe families.

Recommended first slice:

- `dropdown_menu`
- `select`
- `tabs`
- `input_group`

---

## M4 — Public authoring story simplified

Exit criteria:

- New docs/examples teach a single preferred path for inherited foreground.
- `scope_children(...)` is clearly transitional, deprecated, or removed from guidance.
- Reviewers can identify wrapper-shaped inheritance as legacy rather than normal authoring.

---

## M5 — Cleanup decision on `ForegroundScope`

Exit criteria:

- The public fate of `ForegroundScope` is decided:
  - supported compatibility surface,
  - deprecated public surface,
  - or internal-only bridge.
- Any required ADR has been added or updated.
- Compatibility tests and migration notes match the final decision.

---

## M6 — Optional v2 decision: full text-style cascade

Exit criteria:

- A conscious decision exists on whether Fret should support a broader inherited text-style context.
- If yes, scope and risks are documented separately from v1 foreground inheritance.
- If no, docs explicitly say that v1 stops at foreground inheritance.

This milestone is intentionally separate because it is a broader contract than the v1 problem.
