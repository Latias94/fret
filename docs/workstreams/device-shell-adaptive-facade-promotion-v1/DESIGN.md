# Device-Shell Adaptive Facade Promotion v1

Status: Closed closeout reference
Last updated: 2026-04-11

Status note (2026-04-11): this document remains the lane-opening rationale. The current shipped
guidance lives in `CLOSEOUT_AUDIT_2026-04-11.md`.

Related:

- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

This narrow follow-on exists because `device-shell-strategy-surface-v1` is now closed with two real
gallery consumers of the shared `device_shell_*` helper.

That closed lane intentionally deferred one separate question:

> now that the helper is proven, should app-facing code keep importing it from
> `fret_ui_kit::adaptive`, or should the explicit app-facing lane promote it to
> `fret::adaptive::{...}` while still keeping it out of the default prelude?

## Assumptions-first baseline

### 1) This is a facade question, not a helper-shape question

- Area: scope ownership
- Assumption: the helper surface itself is already frozen enough; this follow-on only decides the
  explicit app-facing import lane.
- Evidence:
  - `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
  - `ecosystem/fret-ui-kit/src/adaptive.rs`
- Confidence: Confident
- Consequence if wrong: this lane would reopen helper naming/policy and blur into the closed owner
  split lane.

### 2) Promotion must stay off the default preludes

- Area: public surface hygiene
- Assumption: even if the helper is promoted to `fret::adaptive`, it should remain on an explicit
  import lane and stay out of `fret::app::prelude::*` and `fret::component::prelude::*`.
- Evidence:
  - `ecosystem/fret/src/lib.rs`
  - `docs/crate-usage-guide.md`
  - `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- Confidence: Confident
- Consequence if wrong: the default first-contact surface would widen without a clear need.

### 3) The promotion rule is already written down

- Area: decision trigger
- Assumption: the previous lane's target interface already defines the threshold for promotion.
- Evidence:
  - `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`
  - `docs/workstreams/device-shell-strategy-surface-v1/M3_SECOND_CONSUMER_PROOF_2026-04-11.md`
- Confidence: Confident
- Consequence if wrong: this follow-on would be making up a new rule after the fact.

## In scope

- Decide whether `DeviceShellMode`, `DeviceShellSwitchPolicy`, `device_shell_mode(...)`, and
  `device_shell_switch(...)` should be re-exported from `fret::adaptive`.
- Keep default app/component preludes unchanged.
- Update at least one app-facing proof surface to use the promoted explicit lane.
- Leave one focused gate set for root export and prelude-boundary coverage.

## Out of scope

- Renaming the helper surface.
- Reopening helper owner split (`fret-ui-kit` remains the owner).
- Recipe-owned wrapper growth.
- Panel/container adaptive expansion.
- Default prelude promotion.
