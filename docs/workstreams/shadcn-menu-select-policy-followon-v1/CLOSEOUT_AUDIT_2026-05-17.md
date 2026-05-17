# shadcn Menu/Select Policy Follow-on v1 — Closeout Audit

Status: Closed
Last updated: 2026-05-17

## Scope

This closeout covers the narrow follow-on split from `architecture-surface-fearless-refactor-v1`
after ASF-060. The lane existed to resolve the shadcn Select pointer-open + ArrowDown contract with
Radix/Base UI/shadcn source evidence and decide whether a broader shared menu/select policy cleanup
should continue here.

## Findings

### 1. The Select pointer-open contract is resolved

`SMS-010` aligned Fret Select with the Radix/shadcn behavior axis:

- shadcn v4 Select wraps Radix Select,
- Radix pointer-open focuses the selected item after content positioning,
- with no selected value, Radix falls back to the first valid item,
- and Base UI corroborates the same headless direction by using selected/highlighted item state as
  the navigation anchor.

Fret keeps tree focus on the listbox container for pointer-open, but initializes
`active_descendant` to the selected/first-enabled row. With no selected value, the open state starts
at Apple and the first ArrowDown advances to Banana.

Evidence:

- `ecosystem/fret-ui-shadcn/src/select.rs`
- `ecosystem/fret-ui-shadcn/tests/select_keyboard_navigation.rs`
- `docs/workstreams/shadcn-menu-select-policy-followon-v1/JOURNAL/2026-05-17-sms-010.md`
- `docs/workstreams/shadcn-menu-select-policy-followon-v1/EVIDENCE_AND_GATES.md`

### 2. No broader shared policy extraction is justified inside this lane

`SMS-020` remains intentionally deferred. The lane now has one source-backed Select fix, but it does
not have a second concrete failing surface proving that roving/typeahead extraction, submenu intent,
or dismissal/focus restore should be handled in this same folder.

Keeping this lane open would make the scope drift from a failing Select contract into an open-ended
menu/select parity campaign. Future work should start from a narrower repro and a named owner.

### 3. The shipped evidence is sufficient for closeout

The final gate set proves the code behavior, the headless entry-focus boundary from ASF-060, and the
workstream metadata:

```bash
cargo fmt --package fret-ui-shadcn
cargo test -p fret-ui-shadcn --locked --test select_keyboard_navigation -j 1
cargo test -p fret-ui-shadcn --locked --lib select_pointer_open_arrow_down_moves_active_descendant -j 1
cargo test -p fret-ui-shadcn --locked --lib select_grouped_pointer_open_arrow_down_moves_active_descendant -j 1
cargo test -p fret-ui-headless --locked --lib entry_focus -j 1
cargo test -p fret-ui-kit --locked --lib initial_focus -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```

## Follow-ons

Open a new narrow workstream only when fresh source-backed evidence exists for one of these owners:

- roving/typeahead collection extraction across at least two shadcn menu/select surfaces,
- submenu grace or focus transfer with a failing dropdown/context/menubar gate,
- dismissal/focus restore behavior that is shared across recipes and not already owned by overlay
  policy lanes.

Do not reopen this lane for broad visual parity work or recipe redesign. Visual or structural
Select/Combobox redesign remains separate from this pointer-open navigation contract.

## Closure Decision

Close `shadcn-menu-select-policy-followon-v1` as complete. `SMS-010` and `SMS-030` are done;
`SMS-020` is deferred to future narrower follow-ons if fresh cross-surface evidence appears.
