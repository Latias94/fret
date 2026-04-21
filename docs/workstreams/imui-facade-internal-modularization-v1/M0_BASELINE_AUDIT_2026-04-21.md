# ImUi Facade Internal Modularization v1 - M0 Baseline Audit (2026-04-21)

## Purpose

Freeze why immediate-mode internal modularization is now its own lane and identify the lowest-risk
first slice.

## Findings

### 1) The main risk is concentration, not owner ambiguity.

Current hotspot sizes:

- `ecosystem/fret-ui-kit/src/imui.rs`: 2209 lines
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`: 1027 lines
- `ecosystem/fret-ui-kit/src/imui/options.rs`: 907 lines
- `ecosystem/fret-ui-kit/src/imui/response.rs`: 644 lines

These files are already above the point where unrelated follow-ons start colliding inside the same
module even when the outward API is correct.

### 2) `imui.rs` still mixes too many roles.

`ecosystem/fret-ui-kit/src/imui.rs` currently acts as:

- the module hub,
- the public re-export surface,
- the facade/extension-trait home,
- and a large accumulation point for helper-local utilities and wrapper glue.

This is workable, but it raises the review cost of every additional narrow follow-on.

### 3) `interaction_runtime.rs` is the largest remaining structural hazard.

`ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs` currently mixes:

- window/global stores,
- hover-delay bookkeeping,
- lifecycle activation/deactivation bookkeeping,
- disabled-scope helpers,
- and drag threshold / drag-finish transitions.

That file still needs a later owner split, but it is more coupled than the option/response
vocabularies.

### 4) `options.rs` and `response.rs` are the safest first slice.

`ecosystem/fret-ui-kit/src/imui/options.rs` and
`ecosystem/fret-ui-kit/src/imui/response.rs` already expose stable outward vocabularies.

The first structural gain is therefore:

- keep the root files as re-export hubs,
- move the internal definitions into private concern-specific modules,
- and prove that first-party build/test surfaces stay unchanged.

### 5) Public surface freeze remains the central constraint.

The parity audit and ADR posture still say the same thing:

- do not widen `crates/fret-ui`,
- do not use this lane to reopen key-owner or menu/tab policy questions,
- and do not introduce new immediate helpers under the cover of module motion.

## First-slice decision

M0 chooses this first implementation slice:

1. modularize `options.rs`,
2. modularize `response.rs`,
3. leave `interaction_runtime.rs` and `imui.rs` for later milestones,
4. wire the new lane into roadmap/workstream/todo indexes and a minimal source-policy gate.

## Evidence anchors

- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `python3 tools/audit_crate.py --crate fret-ui-kit`
