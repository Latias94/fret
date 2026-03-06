# Foreground Style Context (Fearless Refactor v1) — TODO

Status: In progress
Last updated: 2026-03-06

Related:

- Design: `docs/workstreams/foreground-style-context-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/foreground-style-context-fearless-refactor-v1/MILESTONES.md`
- General guidance: `docs/fearless-refactoring.md`
- Provider guidance: `docs/service-injection-and-overrides.md`

Tracking legend:

- `[ ]` open
- `[~]` in progress
- `[x]` done
- `[!]` blocked

ID format:

- `FSC-{area}-{nnn}`

---

## A. Decision + Contract Audit

### Component Review Matrix

Use this table to track call-site review and migration priority. The goal is not to prove every
entry is already wrong; the goal is to make the review surface explicit and keep audit progress
visible.

| Family | Component / Area | Primary anchor | Owner | Current inheritance shape | Primary risk hypothesis | Priority | Review status | Evidence / Gate | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Menu / Overlay | `dropdown_menu` | `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` | `fret-ui-shadcn` | Mixed `scope_element(...)` and `scope_children(...)` | Multi-sibling wrapper semantics inside menu content/rows can drift layout ownership | P0 | `[ ]` | Menu-family unit tests + overlay/text-wrap regression gate | High-value first audit target |
| Form / Overlay | `select` | `ecosystem/fret-ui-shadcn/src/select.rs` | `fret-ui-shadcn` | `scope_children(...)` in recipe composition | Trigger/content/value layout may change when color inheritance inserts a wrapper | P0 | `[ ]` | Select/content layout regression + wrapped-text gate | Related to wrapped text and menu-like surfaces |
| Navigation | `tabs` | `ecosystem/fret-ui-shadcn/src/tabs.rs` | `fret-ui-shadcn` | `scope_children(...)` in part composition | Trigger/content alignment and shrink/fill expectations may become wrapper-sensitive | P1 | `[ ]` | Trigger-row geometry test + shrink/fill regression | Review with focus on trigger row geometry |
| Form / Layout | `input_group` | `ecosystem/fret-ui-shadcn/src/input_group.rs` | `fret-ui-shadcn` | Mix of direct `foreground_scope(...)` and `scope_children(...)` | Addons and input rows can hide layout ownership changes behind muted foreground inheritance | P0 | `[ ]` | Row/addon layout test + explicit-vs-inherited foreground gate | Likely to expose row/slot edge cases |
| AI / Content | `fret-ui-ai/message` | `ecosystem/fret-ui-ai/src/elements/message.rs` | `fret-ui-ai` | `scope_children(...)` around composed stack | Message body/content roots may pick up hidden wrapper semantics under text-heavy layouts | P1 | `[ ]` | Text-wrap/content-root regression test | Good non-shadcn ecosystem case |
| Surface | `card` | `ecosystem/fret-ui-shadcn/src/card.rs` | `fret-ui-shadcn` | Direct `foreground_scope(...)` | Wrapper may be safe today but still teaches the wrong authoring model | P2 | `[ ]` | Simple unit test documenting preferred migration target | Useful for migration examples |
| Surface / Content | `alert` | `ecosystem/fret-ui-shadcn/src/alert.rs` | `fret-ui-shadcn` | Direct `foreground_scope(...)` around content blocks | Description/content composition may drift when text and icons share inherited foreground | P1 | `[ ]` | Text-heavy alert layout regression | Check text-heavy variants |
| Menu / Overlay | `context_menu` | `ecosystem/fret-ui-shadcn/src/context_menu.rs` | `fret-ui-shadcn` | Direct `foreground_scope(...)` | Menu item/icon composition may still rely on wrapper-shaped inheritance | P1 | `[ ]` | Menu-item icon/text inheritance test | Review alongside `dropdown_menu` |
| Menu / Navigation | `menubar` | `ecosystem/fret-ui-shadcn/src/menubar.rs` | `fret-ui-shadcn` | Direct `foreground_scope(...)` | Similar to `context_menu`; risk is lower but shape is still legacy | P2 | `[ ]` | Menubar item inheritance regression | Can likely migrate with menu-family pass |
| AI / Content | `task` | `ecosystem/fret-ui-ai/src/elements/task.rs` | `fret-ui-ai` | Direct `foreground_scope(...)` | AI task rows may hide wrapper-induced layout drift in rich content blocks | P2 | `[ ]` | Rich-content row regression test | Review after message/task surfaces are grouped |

- Suggested review states:
  - `[ ]` not reviewed
  - `[~]` reviewed, migration or gate still needed
  - `[x]` reviewed and outcome recorded
  - `[!]` blocked by larger mechanism decision

- [x] FSC-audit-001 Inventory all current author-facing foreground inheritance entry points.
  - Minimum scope:
    - `cx.foreground_scope(...)`
    - `current_color::scope_element(...)`
    - `current_color::scope_children(...)`
- [x] FSC-audit-002 Audit all in-tree `scope_children(...)` call sites and classify them:
  - safe temporary use,
  - migration candidate,
  - immediate correctness risk.
- [x] FSC-audit-003 Audit direct `cx.foreground_scope(...)` call sites outside `fret-ui-kit`.
- [x] FSC-audit-004 Write down the current runtime contract of `ForegroundScope` in one place:
  - mount shape,
  - measurement behavior,
  - paint inheritance behavior,
  - overlay/root boundary behavior.
- [ ] FSC-audit-005 Decide the intended public stance for v1:
  - keep public during migration,
  - mark transitional,
  - or plan deprecation.

---

## B. Mechanism Design Closure

- [x] FSC-design-010 Decide the carrier for inherited foreground in `crates/fret-ui`.
  - Preferred direction: traversal-owned paint/text context, not `LayoutStyle`.
- [x] FSC-design-011 Define the precedence contract:
  - explicit foreground,
  - inherited foreground,
  - theme fallback.
- [ ] FSC-design-012 Decide whether overlay roots inherit foreground automatically or must be
  threaded explicitly.
- [x] FSC-design-013 Decide the initial v1 consumer set.
  - Minimum expected set:
    - `Text`
    - `StyledText`
    - `SelectableText`
    - icon-like surfaces
    - spinner/loading glyphs if applicable
- [x] FSC-design-014 Decide whether full text-style cascade is in scope for v1 or deferred to v2.

---

## C. Mechanism Prototype (`crates/fret-ui`)

- [x] FSC-mech-020 Add a minimal inherited foreground context path that does not require a
  synthetic author-facing wrapper node.
- [x] FSC-mech-021 Teach real subtree roots to install inherited foreground.
  - Candidate roots:
    - `Container`
    - `Pressable`
    - flex/row/column roots
- [x] FSC-mech-022 Teach paint consumers to read the new inherited foreground carrier.
- [x] FSC-mech-023 Keep `ForegroundScope` working through a compatibility bridge while migration is
  in flight.
- [x] FSC-mech-024 Add comments/docs at the mechanism boundary clarifying that inherited foreground
  is context, not a layout fragment.

---

## D. Migration (`ecosystem/*`)

- [~] FSC-migrate-030 Migrate high-risk shadcn surfaces first.
  - Initial candidates:
    - `dropdown_menu`
    - `select`
    - `tabs`
    - `input_group`
- [~] FSC-migrate-031 Migrate nearby ecosystem surfaces that compose layout-heavy content under
  inherited foreground.
  - Initial candidates:
    - `fret-ui-ai/message`
    - other menu/list/overlay content roots found by the audit
- [ ] FSC-migrate-032 Stop introducing new `scope_children(...)` usages in new code.
- [x] FSC-migrate-033 Convert helper APIs/doc examples to prefer real layout roots carrying
  inherited foreground.

---

## E. Regression Gates

- [x] FSC-gates-040 Add a unit/integration test proving that inherited foreground does not change
  sibling flow ownership when attached to a real layout root.
- [ ] FSC-gates-041 Add a regression test for wrapped text under migrated menu/overlay content.
- [ ] FSC-gates-042 Add a regression test proving explicit color still overrides inherited
  foreground.
- [x] FSC-gates-043 Add a compatibility test for legacy `ForegroundScope` during the migration
  window.
- [ ] FSC-gates-044 Consider a grep/lint-style guard that flags new `scope_children(...)` usage once
  the replacement path is stable.

---

## F. Docs + ADRs

- [x] FSC-docs-050 Keep this workstream doc set updated as decisions close.
- [ ] FSC-docs-051 Update user-facing authoring guidance once the replacement path exists.
- [ ] FSC-docs-052 Add or update an ADR when any hard contract changes are approved.
  - Trigger examples:
    - public `ForegroundScope` contract change,
    - generic inherited foreground contract in `crates/fret-ui`,
    - full inherited text-style cascade.
- [ ] FSC-docs-053 If an ADR is added, update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` with evidence
  anchors.

---

## G. Cleanup / Exit

- [ ] FSC-cleanup-060 Decide the final public fate of `scope_children(...)`.
- [ ] FSC-cleanup-061 Decide the final public fate of `ForegroundScope`.
- [ ] FSC-cleanup-062 Remove or quarantine transitional helper paths once migration is complete.
- [ ] FSC-cleanup-063 Make sure the final docs teach one boring golden path for inherited
  foreground.
