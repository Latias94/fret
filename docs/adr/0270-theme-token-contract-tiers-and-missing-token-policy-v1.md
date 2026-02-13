# ADR 0270: Theme Token Contract Tiers and Missing-Token Policy (v1)

Status: Proposed

## Context

Fret’s styling system is token-based (ADR 0032) with a baseline token set and alias bridge for
ecosystem semantics (ADR 0050). In practice, a theme config or component ecosystem can reference
tokens that are:

- absent because of a theme upgrade/mismatch,
- misspelled,
- intentionally optional (ecosystem-only),
- or introduced by new components while the theme file lags behind.

Historically, several “required token” accessors could panic when a token was missing. That is
acceptable in early prototypes, but it is not acceptable as a long-lived runtime contract for
core/mechanism crates (ADR 0066).

This ADR defines the v1 missing-token policy and the tiering between mechanism-owned typed keys and
ecosystem-owned string extension tokens.

## Goals

1. Make theme token reads **non-panicking by default**.
2. Define which tokens are “core/mechanism-owned” vs “ecosystem extension”.
3. Preserve diagnosability: missing tokens should be discoverable via stable diagnostics.
4. Provide an opt-in strict mode that re-enables panics for development.
5. Keep `crates/fret-ui` mechanism-only (ADR 0066): policy decisions remain in ecosystem crates.

## Non-goals (v1)

- Designing a full schema/versioning system for theme configs.
- Introducing a mandatory logging/telemetry framework for diagnostics.
- Enforcing a single “component token vocabulary” across all ecosystems.

## Decision

### D1 — Two-tier token contract

**Tier 1: Typed core keys (mechanism-owned)**

Mechanism/runtime code should prefer typed keys for the “hard-to-change” baseline surfaces:

- `ThemeColorKey` and `ThemeMetricKey` (and future typed groups as needed).

Policy:

- Typed core keys must always resolve to a value (either explicitly configured or filled from a
  stable baseline), and must not panic in the default runtime mode.

**Tier 2: String tokens (ecosystem extension)**

Ecosystem layers may use string keys (dotted token ids; ADR 0050 alias bridge included) for tokens
that are not part of the mechanism-owned baseline.

Policy:

- Missing string tokens are treated as a configuration error that must be diagnosable, but must not
  crash the process by default.

### D2 — Missing token behavior: warn-once + stable fallback

For all string token categories (colors, metrics, corners, numbers, durations, easings, text
styles), the default behavior is:

1. Attempt to resolve the token.
2. If missing:
   - emit a warn-once diagnostic keyed by `(kind, token_id)`,
   - return a stable fallback value.

Fallback strategy (v1):

- Prefer mapping to the baseline theme (`default_theme()`) when a meaningful baseline exists (ADR 0050).
- Otherwise return a deterministic “safe” value (e.g. zero duration, identity easing, empty style)
  chosen to avoid panics and keep rendering stable.

### D3 — `*_required` accessors are legacy and must be non-panicking by default

`*_required` accessors exist for historical API compatibility, but in v1 they follow the same
non-panicking behavior as `*_token` in the default runtime mode.

They may be kept as a “compat shim” for a limited time, but call sites should migrate to `*_token`
to make the behavior explicit.

### D4 — Strict runtime mode (opt-in)

When `FRET_STRICT_RUNTIME=1` is set, missing tokens in the “string token” layer may panic to
surface configuration problems early during development.

Strict mode is intended for local development and CI debugging, not as a production default.

### D5 — Key canonicalization and aliasing remain compatible with ADR 0050

String token reads canonicalize/alias legacy shadcn/gpui-component vocabulary into the baseline
token set described by ADR 0050 where appropriate.

This keeps ecosystem components functional while migrating toward more typed, mechanism-owned keys.

## Consequences

- A missing token no longer terminates the process in production-default behavior.
- Visual output may degrade to a baseline fallback, but the runtime remains usable and provides
  diagnostics to fix the theme/component mismatch.
- Ecosystem crates can adopt additional optional tokens without forcing an immediate theme update.

## Implementation (evidence)

- `crates/fret-ui/src/theme/mod.rs` (`*_token` accessors; warn-once + fallback; strict mode gate)
- Tests:
  - `crates/fret-ui/src/theme/mod.rs`
    - `required_accessors_do_not_panic_when_tokens_are_missing_by_default`
    - `required_accessors_panic_in_strict_runtime_mode`
- Ecosystem compatibility:
  - `ecosystem/fret-ui-kit/src/style/theme_read.rs` (`ThemeTokenRead::{color_token,metric_token}`)

