# shadcn Menu/Select Policy Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-17

## M0 — Contract Triage

- [x] SMS-010 [owner=codex] [deps=none] [scope=ecosystem/fret-ui-shadcn/src/select.rs,ecosystem/fret-ui-kit/src/primitives/select.rs,repo-ref/primitives,repo-ref/base-ui,repo-ref/ui]
  Goal: Resolve the select pointer-open + ArrowDown contract using Radix/Base UI/shadcn source evidence.
  Validation: `cargo test -p fret-ui-shadcn --locked --test select_keyboard_navigation -j 1`.
  Evidence: `docs/workstreams/shadcn-menu-select-policy-followon-v1/EVIDENCE_AND_GATES.md#sms-010-select-pointer-open--arrowdown`.
  Result: shadcn v4 wraps Radix Select; Radix pointer-open focuses the selected item after content positioning, falling back to the first valid item. Base UI also treats the selected/first highlighted item as the open-time navigation anchor. Fret keeps focus on the listbox container for pointer-open, but initializes `active_descendant` to that same selected/first-enabled row so the first ArrowDown advances to the next enabled item.

## M1 — Shared Policy Owners

- [ ] SMS-020 [owner=unassigned] [status=deferred] [deps=SMS-010] [scope=ecosystem/fret-ui-headless,ecosystem/fret-ui-kit/src/primitives,ecosystem/fret-ui-shadcn/src/{select.rs,dropdown_menu.rs,context_menu.rs,menubar.rs}]
  Goal: Extract the next repeated policy only if SMS-010 shows drift across recipe surfaces.
  Validation: headless owner unit tests plus focused shadcn recipe gates.
  Evidence: Not executed in this lane; `SMS-010` resolved a Select-specific contract and did not provide enough fresh cross-surface evidence to justify another shared owner extraction.
  Handoff: Future candidate behaviors are roving/typeahead collection extraction, submenu grace/focus transfer, and dismissal/focus restore. Open a narrower follow-on with a concrete failing gate instead of reopening this lane broadly.

## M2 — Closeout

- [x] SMS-030 [owner=codex] [deps=SMS-010] [scope=docs/workstreams/shadcn-menu-select-policy-followon-v1]
  Goal: Close this follow-on or split a smaller lane if the remaining work becomes a visual/parity campaign.
  Validation: `WORKSTREAM.json`, `TODO.md`, and `EVIDENCE_AND_GATES.md` agree.
  Evidence: `docs/workstreams/shadcn-menu-select-policy-followon-v1/CLOSEOUT_AUDIT_2026-05-17.md`.
  Handoff: Closed on 2026-05-17. Future menu/select policy cleanup should start from fresh source-backed repro evidence and a narrower owner.
