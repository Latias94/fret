# Diag simplification v1 — M0 baseline (capability matrix + naming policy)

Status: Active (workstream note; not a contract)

This note closes the M0 exit criteria for `docs/workstreams/diag-simplification-v1.md` by documenting:

- a filesystem vs DevTools WS **behavior/capability matrix** (what to expect, where artifacts live),
- a stable **naming + backward-compat** policy for `reason_code` and `capabilities`.

For deeper background, see:

- `docs/workstreams/diag-extensibility-and-capabilities-v1/capabilities.md`
- `docs/workstreams/diag-extensibility-and-capabilities-v1/evidence-and-trace.md`

---

## Transport behavior matrix (filesystem vs DevTools WS)

Important: capabilities are **runner-owned** (native vs web, feature flags, etc). The matrix below documents how
tooling discovers and uses them per transport.

| Topic | Filesystem transport | DevTools WS transport | Notes |
| --- | --- | --- | --- |
| Session discovery | Implicit single session via filesystem shim (`session.list`) | Explicit `session.list` from DevTools hub | Tooling should still select a session deterministically. |
| Capability discovery | Read `capabilities.json` under `FRET_DIAG_DIR` | Read from `session.list.sessions[*].capabilities` | Strings are normalized (legacy → namespaced). |
| Required capabilities | `meta.required_capabilities` + inferred requirements from script steps | Same | Inference examples: `capture_screenshot` ⇒ `diag.screenshot_png`, `schema_version>=2` ⇒ `diag.script_v2`. |
| Missing capabilities | **Fail fast**: write `check.capabilities.json` + write `script.result.json` (`reason_code=capability.missing`) | Same | Avoids "timeouts by accident". |
| Script push/run | Write `script.json` then touch `script.touch` | Send `script.run` message payload | Tooling should treat these as equivalent logical operations. |
| Script result | Runner writes `script.result.json` + touches `script.result.touch` | Tooling waits for `script.result` message and writes `script.result.json` locally | Both should converge on the same local artifact layout under `--dir`. |
| Bundle dump request | Touch `trigger.touch` (labels are not represented on the filesystem control plane today) | Send `bundle.dump` (supports label on WS control plane) | In both modes, tooling observes `bundle.dumped`. |
| Bundle dumped signal | `latest.txt` update is surfaced as a synthetic `bundle.dumped` message | `bundle.dumped` message from runtime | Tooling materializes WS bundles locally. |
| Artifact materialization | Runtime writes `bundle.json` under `FRET_DIAG_DIR/<export>/bundle.json` | Tooling writes `<out_dir>/<export>/bundle.json` from embedded payload (or falls back to reading runtime path when accessible) | Tooling updates `latest.txt` under `--dir` in both modes. |
| Exit request | Touch `exit.touch` (tooling) | Send `app.exit.request` (tooling) | Transport-neutral semantics: "exit after run" should not depend on transport. |
| On-demand screenshots (`capture_screenshot`) | Requires runner support (`diag.screenshot_png`) and filesystem request/result files | Requires runner support (`diag.screenshot_png`) and WS message flow | Always gate on `diag.screenshot_png` before running. |

---

## Capability naming policy (stable + extensible)

Capabilities are **opaque strings** exchanged between tooling and the runner. Tooling must only use them for:

- deciding whether a script can run deterministically,
- failing fast with structured evidence when support is missing.

### Namespaces

Recommended namespaces:

- `devtools.*`: control plane API surface (sessions/inspect/pick/scripts/bundles).
- `diag.*`: runner/script execution surface (script schema, screenshot, window targeting, input injection, etc).

Rule: tooling MUST ignore unknown capability strings (forward compatible).

### Legacy strings (compat)

Legacy un-namespaced strings may still appear. Tooling normalizes a small set for compatibility:

- `script_v2` → `diag.script_v2`
- `screenshot_png` → `diag.screenshot_png`
- `multi_window` → `diag.multi_window`
- `pointer_kind_touch` → `diag.pointer_kind_touch`
- `gesture_pinch` → `diag.gesture_pinch`

New capabilities should be introduced namespaced-only (do not add new legacy aliases).

### Ownership

- `devtools.*`: owned by the DevTools hub / control-plane implementation.
- `diag.*`: owned by the in-app diagnostics runtime + runner integration.
- Tooling owns only the normalization + gating policy, not the capability vocabulary itself.

---

## `reason_code` naming policy (stable taxonomy)

`UiScriptResultV1.reason_code` is a stable string that makes failures machine-triageable.

Rules:

- codes MUST be stable across refactors (do not change meanings),
- prefer dotted namespaces (`capability.*`, `selector.*`, `timeout`, `assert.*`, ...),
- avoid over-specifying unless it materially improves triage beyond structured evidence.

Current examples:

- `capability.missing` (tooling-side gating failure; see `check.capabilities.json` + `script.result.json` evidence)
- `selector.not_found`, `semantics.missing`, `timeout`, `assert.failed` (runner-side script failures)
- `tooling.connect.failed`, `tooling.script.read_failed`, `tooling.script.parse_failed` (tooling-side setup failures)
- `tooling.repeat.failed` (tooling-side repeat harness failure; should still write a local `script.result.json`)
- `tooling.artifact.integrity.failed` (tooling-side artifact corruption / hash mismatch; should still write/mark a local `script.result.json`)
- `timeout.tooling.script_result`, `timeout.tooling.bundle_dump` (tooling-side timeouts; should still write a local `script.result.json`)

When adding a new `reason_code`, also add/extend bounded structured evidence so the reason is explainable without logs.
