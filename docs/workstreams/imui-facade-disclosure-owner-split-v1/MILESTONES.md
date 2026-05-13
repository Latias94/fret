# ImUi Facade Disclosure Owner Split v1 - Milestones

Status: closed
Last updated: 2026-05-13

## M0 - Baseline

Status: complete

Result (2026-05-13): `facade_writer.rs` was 1506 lines before this follow-on and still owned the
disclosure wrapper cluster.

## M1 - Disclosure Facade Owner Split

Status: complete

Result (2026-05-13): `M1_DISCLOSURE_FACADE_OWNER_SPLIT_2026-05-13.md` moved disclosure wrappers
into private `facade_writer/disclosure.rs`. `facade_writer.rs` dropped from 1506 to 1464 lines,
and `disclosure.rs` is 46 lines.

## M2 - Closeout

Status: complete

Result (2026-05-13): `CLOSEOUT_AUDIT_2026-05-13.md` closes the lane and keeps future text,
boolean/model, table, docking, multi-window, and additive widget work in separate follow-ons.
