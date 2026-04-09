# Diagnostics Architecture (Fearless Refactor v1) — Risk Matrix

Last updated: 2026-03-02

This matrix enumerates the highest-risk failure modes and the gates that keep the refactor safe.

This is not an ADR. If a mitigation is a hard contract, promote it to an ADR.

---

## R1 — Artifact regressions (missing outputs, “just timeout” failures)

Symptoms:

- no `script.result.json` on failure,
- bundle dumps “succeed” but no local artifact exists,
- long timeouts hide real errors.

Mitigation / gates:

- enforce artifact invariants (see `EVIDENCE_AND_GATES.md`),
- add focused unit tests around artifact materialization and path resolution,
- require stable `reason_code` on every failure path.

---

## R2 — Protocol drift (breaking changes, incompatible bundles/scripts)

Symptoms:

- old bundles can’t be read by new tooling,
- old scripts stop running without clear error.

Mitigation / gates:

- additive-only changes without ADR + versioning,
- `diag doctor` style validation stays strict and must remain compatible,
- keep schema v1/v2 shims and test them.

---

## R3 — Runtime overhead (per-frame perf cliffs)

Symptoms:

- diagnostics enabled causes jank even without dumping bundles,
- memory growth or allocations explode on large trees (50k+ semantics nodes).

Mitigation / gates:

- keep snapshot capture bounded and incremental,
- clip large payloads and report clipping,
- add perf gates for diagnostics-enabled scenarios (`fretboard-dev diag perf ...`).

---

## R4 — Extension abuse (ecosystem “debug extensions” become unbounded dump pipes)

Symptoms:

- extensions push huge JSON blobs every frame,
- tooling becomes slow or artifacts become unshareable.

Mitigation / gates:

- hard byte caps per extension key + clip reports,
- capability gating: extensions required by scripts must be declared explicitly,
- add lint that flags large extension payloads in bundles.

---

## R5 — Concurrency hazards (`FRET_DIAG_DIR` collisions)

Symptoms:

- two runs stomp each other’s `latest.txt` / triggers,
- scripts intermittently time out.

Mitigation / gates:

- default to session-isolated out dirs in tooling (`--session-auto`),
- make collisions detectable and fail fast with a stable `reason_code`,
- document “one out dir per agent/task” in quick starts.

---

## R6 — WS transport security footguns (accidental exposure)

Symptoms:

- server binds to non-loopback,
- token is missing/weak,
- browser origin spoof risks.

Mitigation / gates:

- loopback-only default,
- token required by default,
- optionally enforce an origin allowlist in web runner mode.

---

## R7 — Layout sidecars cause stutter (Taffy dumps are expensive)

Symptoms:

- enabling layout dumps causes large hitches,
- developers leave dump flags on by mistake.

Mitigation / gates:

- sidecars must be opt-in per repro (not always-on),
- keep `dump once` defaults in docs,
- tooling should surface a warning when dump flags are enabled.

