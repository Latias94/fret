# Device-Shell Strategy Surface v1

Status: Active
Last updated: 2026-04-10

Related:

- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- `docs/audits/shadcn-sidebar.md`

This narrow follow-on exists because the broad adaptive-closure lane is now closed, but fresh
evidence still shows one specific higher-level gap:

> Fret has the low-level adaptive taxonomy and explicit `fret::env` / `fret::adaptive` lanes, yet
> app-facing desktop/mobile shell branching is still mostly authored as repeated
> `viewport_width_at_least(...)` + manual component swaps.

Current evidence already shows three distinct truths:

- `Sidebar` is intentionally an app-shell / device-shell surface and should not become the generic
  editor-panel answer.
- `Drawer` docs currently teach desktop/mobile pairing as explicit `Dialog` vs `Drawer` branches.
- other app-level recipes such as date-picker dropdowns still hand-roll `Popover` vs `Drawer`
  switching directly at the snippet layer.

That is no longer a broad adaptive-taxonomy problem.
It is a narrow strategy-surface problem above raw query reads.

## Assumptions-first baseline

### 1) The missing layer is strategy/policy, not runtime mechanism

- Area: owning layer
- Assumption: the current gap belongs above `fret::env` and above `crates/fret-ui`.
- Evidence:
  - `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
  - `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
  - `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
- Confidence: Confident
- Consequence if wrong: this lane would drift back into a broad adaptive/runtime reopening.

### 2) Device-shell branching should stay explicit

- Area: public authoring surface
- Assumption: the target surface should say "device shell", "mobile", "desktop", or equivalent;
  it should not collapse back into generic `responsive(...)`.
- Evidence:
  - `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
  - `docs/audits/shadcn-sidebar.md`
  - `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
- Confidence: Confident
- Consequence if wrong: panel/container and device-shell stories would blur again.

### 3) We already have proof surfaces for the baseline drift

- Area: proof surface
- Assumption: existing gallery source surfaces are enough to open this lane without another broad
  visual sweep first.
- Evidence:
  - `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
  - `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs`
  - `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
- Confidence: Likely
- Consequence if wrong: the lane will need one dedicated diag/demo before any extraction.

### 4) Editor rails stay out of scope

- Area: boundary protection
- Assumption: this lane should only own desktop/mobile shell branching, not panel/container
  adaptation for editor rails.
- Evidence:
  - `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
  - `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
- Confidence: Confident
- Consequence if wrong: this lane would become another broad adaptive bucket.

## In scope

- Decide what the shared higher-level device-shell strategy surface should look like above raw
  viewport queries.
- Classify which families should stay explicit app-local branches vs which deserve a reusable
  strategy helper or wrapper.
- Keep naming explicit about device-shell ownership.
- Leave one focused source/gate set for:
  - `Dialog` vs `Drawer`,
  - `Popover` vs `Drawer`,
  - `Sidebar` app-shell device inference.

## Out of scope

- Reopening container-query vs environment-query taxonomy work.
- Generic panel/container adaptive helpers.
- Moving policy into `crates/fret-ui`.
- Extracting editor rail primitives.
- Converting every gallery snippet to a new helper before the contract is frozen.

## Owning layers

- `ecosystem/fret-ui-kit`
  - shared strategy types/helpers above raw query reads
- `ecosystem/fret-ui-shadcn`
  - recipe-owned desktop/mobile wrappers where source-aligned evidence justifies them
- `ecosystem/fret`
  - explicit app-facing facade re-export only if the policy lane proves durable
- `apps/fret-ui-gallery`
  - teaching surface and focused source tests

## Target shipped state

When this lane is done, the following must be true:

1. Fret has an explicit answer for higher-level desktop/mobile shell branching above raw
   `viewport_width_at_least(...)` reads.
2. The answer does not blur app-shell/device-shell logic with panel/container adaptation.
3. Shared helpers, if any, remain explicit about device-shell ownership in both naming and docs.
4. Gallery examples no longer need to rediscover the same `Popover` vs `Drawer` /
   `Dialog` vs `Drawer` branch shape ad hoc.
5. `Sidebar` remains an app-shell/device-shell surface rather than silently widening into a
   generic adaptive rail abstraction.
