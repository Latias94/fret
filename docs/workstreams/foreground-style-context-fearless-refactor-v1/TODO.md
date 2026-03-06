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
| Menu / Overlay | `dropdown_menu` | `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` | `fret-ui-shadcn` | Mixed `scope_element(...)` and `scope_children(...)` | Multi-sibling wrapper semantics inside menu content/rows can drift layout ownership | P0 | `[x]` | `dropdown_menu_checkable_row_attaches_foreground_to_existing_root` | Audited: current row path stamps inherited foreground on the existing content root container |
| Form / Overlay | `select` | `ecosystem/fret-ui-shadcn/src/select.rs` | `fret-ui-shadcn` | `scope_children(...)` in recipe composition | Trigger/content/value layout may change when color inheritance inserts a wrapper | P0 | `[x]` | `select_scroll_buttons_attach_foreground_to_icon_without_wrapper` | Audited: scroll-arrow recipe remains single-root and no longer relies on a synthetic wrapper |
| Navigation | `tabs` | `ecosystem/fret-ui-shadcn/src/tabs.rs` | `fret-ui-shadcn` | `scope_children(...)` in part composition | Trigger/content alignment and shrink/fill expectations may become wrapper-sensitive | P1 | `[x]` | `tabs_trigger_content_attaches_foreground_without_wrapper` + `tabs_content_defaults_to_flex_grow_fill_like_shadcn` | Audited: trigger content stamps inherited foreground on the real flex root while fill/min-w-0 behavior stays covered |
| Form / Selection | `radio_group` | `ecosystem/fret-ui-shadcn/src/radio_group.rs` | `fret-ui-shadcn` | Direct label/icon composition on the item row root | Word-wrapped labels in horizontal rows can still overflow unless the label text explicitly opts into grow + `min-w-0` | P1 | `[x]` | `radio_group_word_wrapped_label_can_shrink_within_row` | Audited: default label text now keeps `wrap: Word` while shrinking inside the row via `flex_grow(1.0)` + `min_w_0()` |
| Form / Layout | `input_group` | `ecosystem/fret-ui-shadcn/src/input_group.rs` | `fret-ui-shadcn` | Direct subtree stamping on addon rows and button-content roots | Addons and input rows can hide layout ownership changes behind muted foreground inheritance | P0 | `[x]` | `input_group_addons_scope_muted_foreground_for_current_color_parity` + `input_group_button_content_attaches_foreground_without_wrapper` | Audited: addon rows and button content now both stamp inherited foreground on existing roots without synthetic wrappers |
| AI / Content | `fret-ui-ai/message` | `ecosystem/fret-ui-ai/src/elements/message.rs` | `fret-ui-ai` | Direct subtree stamping on the message content stack | Message body/content roots may pick up hidden wrapper semantics under text-heavy layouts | P1 | `[x]` | `message_content_user_bubble_attaches_foreground_without_wrapper` + `message_content_assistant_defaults_to_fill_width_for_stable_wrap` | Audited: user bubble now stamps inherited foreground on the existing stack root, and assistant content keeps full-width flow for stable wrapping |
| AI / Header | `fret-ui-ai/agent` | `ecosystem/fret-ui-ai/src/elements/agent.rs` | `fret-ui-ai` | Icon + word-wrapped label + badge inside a header row | Long agent names can overflow unless the label opts into grow + `min-w-0` and the inner row fills the card width | P1 | `[x]` | `agent_header_label_can_shrink_within_row` | Audited: agent header labels now keep `wrap: Word` while shrinking inside the header row via `flex_grow(1.0)` + `min_w_0()` |
| AI / Header | `fret-ui-ai/sandbox` | `ecosystem/fret-ui-ai/src/elements/sandbox.rs` | `fret-ui-ai` | Icon + word-wrapped label + badge inside a trigger row with trailing chevron | Long sandbox titles can overflow when the left cluster does not grow/shrink against the chevron slot | P1 | `[x]` | `sandbox_header_label_can_shrink_within_trigger_row` | Audited: sandbox header labels now shrink within the trigger row and the left cluster explicitly opts into grow + `min_w_0()` |
| AI / Header | `fret-ui-ai/tool` | `ecosystem/fret-ui-ai/src/elements/tool.rs` | `fret-ui-ai` | Icon + word-wrapped label + badge inside a trigger row with trailing chevron | Tool titles can overflow unless both the left cluster and the label text opt into shrink-friendly row constraints | P1 | `[x]` | `tool_header_label_can_shrink_within_trigger_row` | Audited: tool header labels now keep word wrap while shrinking within the trigger row via explicit grow + `min_w_0()` |
| AI / Content | `fret-ui-ai/sources_block` | `ecosystem/fret-ui-ai/src/elements/sources_block.rs` | `fret-ui-ai` | Truncating title/link rows plus trigger rows inside a collapsible sources list | Source titles and trigger labels can overflow unless both the text and the row-level link/container opt into shrink-friendly constraints | P1 | `[x]` | `sources_block_item_title_can_shrink_within_row` + `sources_block_trigger_label_can_shrink_within_row` | Audited: source title rows and trigger labels now keep truncation semantics while shrinking within their rows via explicit grow + `min_w_0()` |
| AI / Content | `fret-ui-ai/web_preview` | `ecosystem/fret-ui-ai/src/elements/web_preview.rs` | `fret-ui-ai` | Timestamp + word-wrapped message rows inside the console collapsible | Console messages can overflow beside the timestamp unless the message text explicitly opts into grow + `min-w-0` | P1 | `[x]` | `web_preview_console_messages_can_shrink_within_log_rows` | Audited: console log messages now keep `wrap: Word` while shrinking beside timestamps via explicit grow + `min_w_0()` |
| AI / Content | `fret-ui-ai/inline_citation` | `ecosystem/fret-ui-ai/src/elements/inline_citation.rs` | `fret-ui-ai` | Fixed-width hover-card content with truncating title/URL and wrapped quote text | Hover-card content can overflow unless title/URL/quote blocks and their parent stacks opt into width fill + `min-w-0` | P1 | `[x]` | `inline_citation_title_and_url_can_truncate_within_card_width` + `inline_citation_quote_can_wrap_within_card_width` | Audited: inline citation hover-card content now preserves ellipsis/wrap semantics within the fixed-width card via explicit `w_full()` + `min_w_0()` |
| Navigation / App Bar | `fret-ui-material3/top_app_bar` | `ecosystem/fret-ui-material3/src/top_app_bar.rs` | `fret-ui-material3` | Title text inside single-row and two-row action-bar slots | Long titles can overflow action/title slots unless the title text and its flex wrappers opt into fill width + `min-w-0` | P1 | `[x]` | `top_app_bar_titles_can_truncate_within_horizontal_slots` | Audited: top app bar titles now keep ellipsis semantics while shrinking within single-row and two-row title slots via explicit fill width + `min_w_0()` |
| Form / Overlay | `fret-ui-material3/select` | `ecosystem/fret-ui-material3/src/select.rs` | `fret-ui-material3` | Trigger value text inside the leading-icon + chevron row | Long selected values can overflow the trigger row unless the value text and the left content slot opt into fill width + `min-w-0` | P1 | `[x]` | `select_trigger_value_can_truncate_within_trigger_row` | Audited: select trigger value text now keeps ellipsis semantics within the trigger row via explicit fill width + `min_w_0()` on the value slot and left content wrapper |
| Surface | `card` | `ecosystem/fret-ui-shadcn/src/card.rs` | `fret-ui-shadcn` | Direct subtree stamping via `.inherit_foreground(...)` | Wrapper may be safe today but still teaches the wrong authoring model | P2 | `[x]` | `card_root_has_default_vertical_padding_and_visible_overflow` | Audited: card root already carries inherited foreground on the existing container root |
| Surface / Content | `alert` | `ecosystem/fret-ui-shadcn/src/alert.rs` | `fret-ui-shadcn` | Direct subtree stamping via `.inherit_foreground(...)` | Description/content composition may drift when text and icons share inherited foreground | P1 | `[x]` | `alert_attaches_foreground_to_main_content_without_wrapper` + `alert_forces_icon_to_inherit_current_color` | Audited: main alert content stamps inherited foreground on the existing root and icons still follow current color |
| Menu / Overlay | `context_menu` | `ecosystem/fret-ui-shadcn/src/context_menu.rs` | `fret-ui-shadcn` | Direct subtree stamping via `.inherit_foreground(...)` | Menu item/icon composition may still rely on wrapper-shaped inheritance if legacy surfaces regress | P1 | `[x]` | `context_menu_row_attaches_inherited_foreground_without_wrapper` | Audited: menu-row leading icon now carries inherited foreground directly on the icon node |
| Menu / Navigation | `menubar` | `ecosystem/fret-ui-shadcn/src/menubar.rs` | `fret-ui-shadcn` | Direct subtree stamping via `.inherit_foreground(...)` | Similar to `context_menu`; ensure menu rows stay wrapper-free while icon color still inherits | P2 | `[x]` | `menubar_row_attaches_inherited_foreground_without_wrapper` | Audited: menu-row leading icon inherits foreground without adding a layout wrapper |
| AI / Content | `task` | `ecosystem/fret-ui-ai/src/elements/task.rs` | `fret-ui-ai` | Direct subtree stamping via `.inherit_foreground(...)` | AI task rows may hide wrapper-induced layout drift in rich content blocks | P2 | `[x]` | `task_trigger_default_row_attaches_foreground_without_wrapper` | Audited: default task trigger row now carries inherited foreground on the existing row root without a synthetic wrapper |

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
