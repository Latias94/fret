# Local-State Facade Boundary Hardening v1 — Design

Status: closed / maintenance lane (initial O1 hardening batch landed)
Last updated: 2026-03-16

Related:

- `docs/workstreams/local-state-architecture-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/OPTION_MATRIX_2026-03-16.md`
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`
- `docs/workstreams/local-state-facade-boundary-hardening-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/local-state-facade-boundary-hardening-v1/SURFACE_INVENTORY_2026-03-16.md`
- `docs/workstreams/local-state-facade-boundary-hardening-v1/MILESTONES.md`
- `docs/workstreams/local-state-facade-boundary-hardening-v1/TODO.md`
- `docs/adr/0223-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`
- `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret/src/lib.rs`

---

## 0) Why this lane existed

`local-state-architecture-fearless-refactor-v1` is now closed on **O1**:

- keep `LocalState<T>` model-backed,
- keep `use_local*` / `LocalState<T>` as the only default local-state story,
- keep `use_state` as the explicit raw-model seam,
- do not open a self-owned/plain-Rust storage prototype.

That architectural decision is settled.

What remains is implementation hardening:

> make the public facade, docs, rustdoc, and source-policy gates say the same thing about the
> default local-state lane versus the explicit raw-model and bridge lanes.

This is intentionally narrower than the closed architecture lane.
It is not allowed to reopen the storage-model question by accident.

Closeout update on 2026-03-16:

- the initial O1 hardening batch is now landed,
- `CLOSEOUT_AUDIT_2026-03-16.md` closes this lane as closed / maintenance,
- and no narrower follow-on is required under current evidence.

---

## 1) Current surface facts

Today the repo already has the right high-level direction, but the boundary is still spread across
multiple surfaces:

- `use_local*` is the default authoring path in docs/templates/examples.
- `tools/gate_no_use_state_in_default_teaching_surfaces.py` already protects first-contact teaching
  surfaces from drifting back to `use_state`.
- `fret::app::prelude::*` intentionally omits `AppUiRawStateExt`.
- `fret::advanced::prelude::*` intentionally reexports `AppUiRawStateExt`.
- `LocalState<T>` still exposes explicit raw/bridge surfaces such as:
  - `model()` / `clone_model()`,
  - `read_in(...)` / `value_in*` / `update_in*` / `set_in(...)`,
  - `watch_in(...)` / `layout_in(...)` / `paint_in(...)` / `hit_test_in(...)`.
- `use_local_with(...)` still layers on `use_state_with(...)` inside the runtime implementation.

This means the repo is not blocked on another state architecture choice.
It is blocked on clarifying which of the surviving seams are:

- default app-authoring surface,
- explicit raw-model seam,
- explicit bridge for helper-heavy or hybrid surfaces,
- or advanced runtime/component ownership boundary.

---

## 2) Goals

### G1 — Freeze one explicit surface taxonomy

This lane should leave the repo with one stable reading:

- **default app lane**:
  - `use_local*`,
  - `LocalState<T>`,
  - tracked reads through `TrackedStateExt`,
  - `cx.actions().locals::<A>(...)`.
- **explicit raw-model lane**:
  - `AppUiRawStateExt::use_state*`,
  - returned `Model<T>` handles,
  - advanced-only, non-default, intentional.
- **explicit bridge lane**:
  - `LocalState::model()`,
  - `LocalState::clone_model()`,
  - `LocalState::*_in(...)`,
  - helper-heavy `ElementContext` accessors such as `watch_in(...)`.

### G2 — Harden wording and discoverability before deleting APIs

The first job is not “remove things because they look low-level”.
The first job is to make sure:

- docs,
- rustdoc,
- prelude/export policy,
- and source-policy tests

all describe the same contract.

### G3 — Keep advanced and hybrid ownership honest

The repo still needs explicit bridge surfaces for:

- component APIs that intentionally speak `Model<T>`,
- helper-heavy `ElementContext` call sites,
- hybrid/default-host + runtime-owned integrations,
- and advanced manual assembly.

This lane must keep those seams explicit rather than pretending every caller should look like a
starter todo app.

### G4 — Make future reduction optional and reviewable

If the repo later wants to shrink the explicit seam further, this lane should leave behind:

- a classified inventory,
- wording that already distinguishes default from explicit,
- and source-policy gates that keep future reductions narrow.

---

## 3) Non-goals

This lane is not for:

- changing the storage model behind `LocalState<T>`,
- opening a self-owned/plain-Rust prototype,
- reintroducing `use_state` into default teaching surfaces,
- deleting explicit `Model<T>` bridges that still reflect real ownership,
- widening `fret::app::prelude::*`,
- or inventing new local-state helper sugar to make the boundary look cleaner.

---

## 4) Hard constraints

### C1 — `use_local*` remains the only default local-state story

No patch on this lane may make `use_state` or raw `Model<T>` handles look co-equal with
`LocalState<T>`.

### C2 — Explicit seams stay off the default prelude

`AppUiRawStateExt` must remain on the advanced lane, not the app prelude.

### C3 — Bridge APIs must be labeled by ownership, not hidden

If a surface exists because a caller intentionally needs:

- `ModelStore`,
- `ElementContext`,
- or a raw `Model<T>` handle,

then the docs should say so directly.

### C4 — No layering inversion

Portable crates still must not learn about app-facing `LocalState<T>`.
This lane hardens the app-facing facade only.

---

## 5) Execution batches

### B1 — Inventory and classification

Produce one concrete inventory of:

- raw-model seams,
- bridge seams,
- default local-state surfaces,
- prelude/export placement,
- and current gates.

### B2 — Target boundary wording

Tighten:

- `ecosystem/fret/src/view.rs` rustdoc,
- `ecosystem/fret/src/lib.rs` public-surface wording,
- and the main docs indices / usage notes that still talk about local-state surfaces.

The output should be a stable contract sentence, not an aspirational note.

### B3 — Narrow source-policy and export hardening

If the wording audit shows drift, patch the narrowest possible set of:

- source-policy tests in `ecosystem/fret/src/lib.rs`,
- rustdoc assertions in `ecosystem/fret/src/view.rs`,
- or export placement / advanced-lane wording.

Avoid larger code motion unless a smaller wording/gate patch cannot express the boundary cleanly.

### B4 — Close or spin out

Close this lane once the boundary is stable.
Only spin out another lane if review finds a narrower code-level change that is real, bounded, and
not just deferred architecture anxiety.

Current result:

- the initial wording/gate batch is landed,
- the closeout audit now records that wording + source-policy hardening was sufficient,
- and the lane closes without opening another immediate follow-on.

---

## 6) Definition of done

This lane is done when:

- the repo consistently describes `use_local*` / `LocalState<T>` as the only default local-state
  story,
- `use_state` is consistently documented as explicit raw-model API on the advanced lane,
- `LocalState::{model, clone_model, *_in, *watch_in}` are classified as intentional bridge APIs
  rather than ambiguous “also normal” helpers,
- source-policy gates protect the default teaching lane,
- and the lane closes without reopening storage-model design.
