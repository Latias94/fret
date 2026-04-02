# Recipe Promotion Audit — 2026-04-02

Status: Closed with keep-local verdicts on 2026-04-02

## Scope

This audit answers one narrower question than the shell-composition lane:

- which repeated app-level helpers on the default app path should stay app-owned,
- and which might eventually deserve a shared recipe owner?

This audit is intentionally narrower than shell promotion.

It does **not** authorize a new `AppShell`.

## Candidate set

Current detection surface:

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/simple_todo_demo.rs`
- richer todo scaffold sections in `apps/fretboard/src/scaffold/templates.rs`
- app-owned cookbook scaffolds where they overlap semantically

## Current decisions

### 1) Responsive centered page wrapper

**Decision:** keep app-owned.

Reason:

- this pattern is page-shell-shaped,
- the recent shell audit already rejected promoting a shared page shell in the current state,
- the known first-party consumers still differ materially in semantics:
  - cookbook lesson shell,
  - UI Gallery docs scaffold,
  - `todo_demo` responsive product shell.

If promotion is ever reconsidered, it should happen through a fresh shell-aware audit rather than
through this recipe note.

Evidence:

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/simple_todo_demo.rs`
- `apps/fret-cookbook/src/scaffold.rs`
- `apps/fret-ui-gallery/src/ui/doc_layout.rs`
- `docs/workstreams/shell-composition-fearless-refactor-v1/PAGE_SHELL_AUDIT_2026-04-02.md`

Follow-up landed:

- `todo_demo` now makes this choice explicit with file-local shell helpers:
  - `todo_page(...)`
  - `todo_card_section(...)`
  - `todo_card_footer_section(...)`
- this is deliberate local extraction, not the start of a shared shell API.

### 2) Todo/card header recipe

**Decision:** keep local for now.

Reason:

- the current composition is still tightly coupled to Todo semantics:
  - title,
  - remaining-count status line,
  - completion-progress block,
  - iconography and copy choices,
- current evidence is still clustered in one Todo-shaped lane (`todo_demo`, `simple_todo_demo`,
  and richer starter variants),
- this is not yet strong proof of a reusable first-party recipe.

Why the current consumers are not aligned enough:

- `todo_demo` uses stronger product chrome:
  - icon badge,
  - success sparkle state,
  - progress block,
  - responsive footer and card shell constraints.
- `simple_todo_demo` uses a lighter starter header:
  - title,
  - status line,
  - one compact progress badge.
- richer scaffold `todo` now uses:
  - title,
  - summary,
  - one compact progress badge,
  - secondary focus-note callout below the main content rather than a richer header band.

These are related compositions, but not one stable reusable recipe yet.

Potential future owner if promotion ever becomes justified:

- `ecosystem/fret-ui-shadcn` or another explicit recipe owner,
- not `fret::app::prelude::*`,
- not the `fret` root facade.

Evidence:

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/simple_todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`

### 3) Hover-reveal destructive action row

**Decision:** keep local for now; still the most plausible future recipe candidate.

Reason:

- the interaction pattern is potentially broader than Todo:
  - file rows,
  - settings rows,
  - command/history rows,
  - other list/table-like surfaces with optional destructive affordances,
- but current explicit proof is still too small,
- the row currently also carries Todo-specific policy and styling decisions.

Why promotion is still rejected now:

- `todo_demo` is the only audited default-ladder consumer using true hover-reveal destructive
  actions tied to pointer capability and viewport width.
- `simple_todo_demo` keeps its remove action always visible.
- the richer scaffold `todo` currently omits row removal entirely.

So the repo does not yet have three aligned first-party consumers for the same interaction rule.

Follow-up landed:

- `todo_demo` now keeps the destructive styling local through
  `subtle_destructive_button_style(...)`, shared only inside the demo between:
  - footer maintenance action,
  - row remove affordance.
- that helper remains file-local because the surrounding visibility policy is still demo-specific.

Promotion gate:

- require at least three aligned first-party consumers,
- require stable keyboard/pointer visibility policy,
- choose an explicit recipe/component owner,
- leave a behavior gate behind before calling the promotion done.

## Promotion rule retained for this lane

Do not promote an app-level helper into a shared recipe unless all of the following are true:

1. at least three aligned first-party consumers exist,
2. the helper is not just one Todo-shaped app surface repeated twice,
3. the behavior and styling policy can be named without product-specific vocabulary,
4. one explicit owning crate is chosen,
5. the promoted surface stays off the default `fret` root and prelude,
6. a source-policy, behavior, or diagnostics gate proves the new owner is real.

## Outcome

Current answer:

- no new shared recipe is introduced from this audit,
- the default app lane now makes the keep-local choice explicit in `todo_demo` via file-local
  helpers rather than a new shared owner,
- reopen promotion only if cross-surface evidence becomes stronger than the current Todo-clustered
  proof set.
