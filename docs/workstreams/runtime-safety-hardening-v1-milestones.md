# Runtime Safety Hardening v1 — Milestones

## M0 — Plan + link (same day)

Deliverables:

- Workstream docs exist and are linked:
  - `docs/workstreams/runtime-safety-hardening-v1.md`
  - `docs/workstreams/runtime-safety-hardening-v1-todo.md`
  - `docs/workstreams/runtime-safety-hardening-v1-milestones.md`
- Minimal gate set is documented and agreed:
  - `cargo nextest run -p fret-runtime`
  - `cargo nextest run -p fret-ui`
  - `cargo nextest run -p fret-app`
  - `python3 tools/check_layering.py`

Exit criteria:

- A branch/worktree plan exists for landing breaking API changes safely.

## M1 — ModelStore v2 (highest risk first)

Deliverables:

- ADR for `ModelStore v2` is written and accepted (or explicitly gated as proposed).
  - Current (this branch): `docs/adr/0269-modelstore-v2-lease-and-unwind-safety-v1.md` (Proposed)
- Public leasing is removed/privatized; closure-based APIs are the only supported access path.
- `get_copied/get_cloned` are non-panicking by default; `try_get_*` returns explicit errors for `AlreadyLeased/TypeMismatch`.
- Regression tests exist for:
  - non-panicking lease violations,
  - unwind does not poison store state (when `panic=unwind`).

Exit criteria:

- `cargo nextest run -p fret-runtime` is green.
- First-party call sites compile without using the legacy leasing surface.

## M2 — Theme v2 (diagnostics + normalization)

Deliverables:

- Theme token contract ADR is written.
  - Current (this branch): `docs/adr/0270-theme-token-contract-tiers-and-missing-token-policy-v1.md` (Proposed)
- Mechanism layer uses typed keys for core tokens.
- Missing tokens never panic by default; missing extension tokens generate diagnostics.
  - Missing-token diagnostics are warn-once (stable key), and strict mode can opt back into panics.

Exit criteria:

- `cargo nextest run -p fret-ui` is green.
- At least one targeted test covers missing-token behavior.

## M3 — Remove avoidable unsafe + globals hardening

Deliverables:

- Menu patch `unsafe` is removed from `crates/fret-runtime`.
- Global lease violations no longer panic by default (return `Result` errors; strict mode optional).
- Nested `with_global_mut` no longer relies on `unsafe` (nested leases are treated as an error; non-strict mode runs the closure against a temporary value).

Exit criteria:

- `cargo nextest run -p fret-app` is green.

## M4 — Env flags caching (hot-path hygiene)

Deliverables:

- `FRET_*` debug flags are parsed once into a cached config struct.
- Hot-path code in `fret-ui` no longer reads environment variables directly.

Exit criteria:

- Perf-neutrality validated by existing perf baselines (where available).

## M5 — Clippy hygiene (warnings-as-errors, local gates)

Deliverables:

- `cargo clippy -p fret-ui --all-targets -- -D warnings` is green.
- `cargo clippy -p fret-app --all-targets -- -D warnings` is green.

Exit criteria:

- Clippy can be used as a regression gate for the workstream crates without surfacing new warnings.

## M6 — Local `unsafe` tightening (fret-ui follow-ups)

Deliverables:

- `fret-ui` local helpers avoid unnecessary raw pointer casts in hot data structures.
- `TestHost` no longer relies on avoidable `unsafe` for globals leasing.
- Small inline list invariants are covered by targeted tests.

Exit criteria:

- `cargo clippy -p fret-ui --all-targets -- -D warnings` is green.
- `cargo nextest run -p fret-ui` is green.

## M7 — Defensive panic hardening (fret-app follow-ups)

Deliverables:

- `fret-app` globals leasing restores invariants even under unexpected internal corruption (non-strict mode recovers with diagnostics; strict mode panics).
- Targeted regression tests cover the recovery behavior.

Exit criteria:

- `cargo clippy -p fret-app --all-targets -- -D warnings` is green.
- `cargo nextest run -p fret-app` is green.
- `python3 tools/check_layering.py` is green.

## M8 — Defensive panic hardening (fret-ui follow-ups)

Deliverables:

- `fret-ui` element state access is resilient to corrupted state storage (type mismatches) by default.
- Element state storage invariants are restored on unwind (no state poisoning).
- Declarative host widgets avoid `expect(...)` for text input/area caches (defensive fallbacks; strict mode remains available via `FRET_STRICT_RUNTIME`).

Exit criteria:

- `cargo clippy -p fret-ui --all-targets -- -D warnings` is green.
- `cargo nextest run -p fret-ui` is green.
- `python3 tools/check_layering.py` is green.

## M9 — Panic surface audit (fret-ui follow-ups)

Deliverables:

- Remove "checked above" `expect(...)` and redundant `Option` unwrapping in input/dispatch hot paths.
- Avoid `expect(...)` on `taffy` layout engine operations; strict mode may panic, default mode warns once and enables widget fallback.
- Remove `.unwrap()` from default theme color parsing; strict mode may panic, default mode warns and uses fallback colors.

Exit criteria:

- `cargo clippy -p fret-ui --all-targets -- -D warnings` is green.
- `cargo nextest run -p fret-ui` is green.
- `python3 tools/check_layering.py` is green.
