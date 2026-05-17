# shadcn Menu/Select Policy Follow-on v1 — TODO

Status: Active
Last updated: 2026-05-17

## M0 — Contract Triage

- [ ] SMS-010 [owner=unassigned] [deps=none] [scope=ecosystem/fret-ui-shadcn/src/select.rs,ecosystem/fret-ui-kit/src/primitives/select.rs,repo-ref/primitives,repo-ref/base-ui,repo-ref/ui]
  Goal: Resolve the select pointer-open + ArrowDown contract using Radix/Base UI/shadcn source evidence.
  Validation: `cargo test -p fret-ui-shadcn --locked --test select_keyboard_navigation -j 1`.
  Evidence: Update this lane's evidence doc with the chosen behavior and source anchors.
  Handoff: Do not blindly change the test or implementation; first decide whether pointer-open should leave no active row, make ArrowDown land on the first option, or preserve a selected/current active row before navigation.

## M1 — Shared Policy Owners

- [ ] SMS-020 [owner=unassigned] [deps=SMS-010] [scope=ecosystem/fret-ui-headless,ecosystem/fret-ui-kit/src/primitives,ecosystem/fret-ui-shadcn/src/{select.rs,dropdown_menu.rs,context_menu.rs,menubar.rs}]
  Goal: Extract the next repeated policy only if SMS-010 shows drift across recipe surfaces.
  Validation: headless owner unit tests plus focused shadcn recipe gates.
  Evidence: One shared behavior consumed by at least two surfaces.
  Handoff: Candidate behaviors are roving/typeahead collection extraction, submenu grace/focus transfer, and dismissal/focus restore.

## M2 — Closeout

- [ ] SMS-030 [owner=planner] [deps=SMS-010] [scope=docs/workstreams/shadcn-menu-select-policy-followon-v1]
  Goal: Close this follow-on or split a smaller lane if the remaining work becomes a visual/parity campaign.
  Validation: `WORKSTREAM.json`, `TODO.md`, and `EVIDENCE_AND_GATES.md` agree.
  Evidence: Closeout note or updated status.
