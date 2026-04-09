# Icon System Extension v1 — Milestones

Status: Closed closeout lane (contract shipped; follow-on only)
Last updated: 2026-04-09

Closeout note on 2026-04-09:

- `CLOSEOUT_AUDIT_2026-04-09.md` closes this lane on the shipped v1 icon contract.
- Future work should start as a narrower follow-on rather than reopening this broad lane.

## M0 — Baseline decision

Exit criteria:

- The lane records the current implementation/ADR drift clearly.
- The scope is frozen to:
  - icon contract,
  - declarative multicolor authoring surface,
  - third-party pack protocol.
- The owning layers are explicit and reviewable.

Artifacts:

- `BASELINE_AUDIT_2026-04-09.md`
- `BASELINE_DECISION_2026-04-09.md`
- `TARGET_INTERFACE_STATE.md`
- `DESIGN.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`

Current status:

- `BASELINE_AUDIT_2026-04-09.md` records the starting ADR 0065 / implementation drift.
- `BASELINE_DECISION_2026-04-09.md` freezes the correctness-first boundary:
  first-class icon-definition contract, distinct multicolor declarative surface, and explicit
  third-party pack protocol.
- `TARGET_INTERFACE_STATE.md` records the durable target surface instead of a smallest-diff plan.
- M0 exit criteria are satisfied.
- The lane has since fully closed through M3.

## M1 — Contract slice

Exit criteria:

- `fret-icons` exposes a first-class icon-definition / resolved-icon contract.
- Alias resolution preserves source + fallback + presentation semantics together.
- The change does not leak vendor policy or asset-locator policy into reusable component surfaces.
- `fret-icons` remains renderer-agnostic.

Likely touch points:

- `ecosystem/fret-icons`
- ADR/alignment docs if the accepted contract itself changes

Current status:

- Landed in `ecosystem/fret-icons/src/lib.rs`.
- ADR/alignment docs updated to reflect the new icon-definition contract.
- M1 exit criteria are satisfied.

## M2 — Declarative runtime slice

Exit criteria:

- Fret exposes a first-class declarative `SvgImage` path for multicolor SVG rendering.
- Existing monochrome `currentColor` icon helpers remain intact and continue to teach the default
  UI-icon path.
- The renderer boundary still owns rasterization/caching.

Likely touch points:

- `crates/fret-ui`
- `ecosystem/fret-ui-kit`

Current status:

- Landed in `crates/fret-ui/src/{element.rs,elements/cx.rs,declarative/frame.rs,declarative/mount.rs,declarative/host_widget/{measure.rs,layout.rs,paint.rs},declarative/tests/svg_image.rs}`.
- `ecosystem/fret-ui-kit/src/declarative/icon.rs` now keeps themed `icon(...)` separate from
  explicit registry-authored `icon_authored(...)`.
- Regression coverage exists in both owning crates:
  - `crates/fret-ui/src/declarative/tests/svg_image.rs`
  - `ecosystem/fret-ui-kit/src/declarative/icon.rs`
- M2 exit criteria are satisfied.
- The lane has since fully closed through M3.

## M3 — Pack protocol and teaching-surface closure

Exit criteria:

- The third-party pack protocol is written down and reflected in docs/examples.
- Pack metadata/provenance has an explicit contract home.
- `fret`/`fret-bootstrap` golden-path guidance remains explicit.
- The lane leaves one smallest repro, one gate set, and one evidence set for future follow-ons.

Likely touch points:

- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`
- one smallest proof surface

Current status:

- Landed in `ecosystem/fret-icons/src/lib.rs`,
  `ecosystem/fret-icons-lucide/src/{lib.rs,app.rs}`,
  `ecosystem/fret-icons-radix/src/{lib.rs,app.rs}`,
  `ecosystem/fret-bootstrap/src/lib.rs`,
  `docs/crate-usage-guide.md`, and `docs/examples/todo-app-golden-path.md`.
- Pack metadata/provenance is now explicit in code through `IconPackRegistration`,
  `InstalledIconPacks`, and first-party `PACK_METADATA` exports.
- App/bootstrap guidance now teaches explicit install seams instead of asking users to infer the
  pack contract from scattered examples.
- M3 exit criteria are satisfied.

## Closeout condition

This lane can close when:

- the contract delta is explicit,
- the runtime/declarative multicolor story is real and gated,
- and third-party pack guidance no longer depends on inference from scattered docs.

Status on 2026-04-09:

- satisfied.
- this lane is now closed and should reopen only through a narrower follow-on if fresh evidence
  exceeds the closeout audit.
