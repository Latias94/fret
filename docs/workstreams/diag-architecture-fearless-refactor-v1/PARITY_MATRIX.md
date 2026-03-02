# Diagnostics Architecture (Fearless Refactor v1) — Parity Matrix

Last updated: 2026-03-02

This matrix is a checklist of outcomes, not 1:1 API parity.

Legend:

- `✅` supported / landed
- `⚠️` partial
- `❌` not yet
- `🧭` intentionally different (document rationale)

---

## 1) Transport parity (filesystem vs WS)

| Outcome | Filesystem transport | DevTools WS transport | Notes |
| --- | --- | --- | --- |
| Push script | ✅ | ✅ | Both should accept the same script schema v1/v2 |
| Run script and get `script.result.json` locally | ✅ | ⚠️ | WS must materialize locally even when app can’t write |
| Bundle dump produces `bundle.schema2.json` locally | ✅ | ⚠️ | Requires chunking + host-side materialization |
| Pick (arm + result) | ✅ | ✅ | Must preserve stable selector JSON |
| Continuous inspect | ✅ | ✅ | UX differs; artifact should be consistent |
| Screenshot capture via protocol | ✅ | ✅ | Must be bounded and optional |
| Stable reason codes on failure | ✅ | ✅ | No “just timeout” failures |
| Deterministic exit in `--launch` mode | ✅ | ✅ | Requires `app.exit.request` support and discipline |

---

## 2) Frontend parity (CLI vs DevTools GUI)

| Outcome | `fretboard` CLI | DevTools GUI | Notes |
| --- | --- | --- | --- |
| Browse scripts library | ✅ | ✅ | GUI should be faster; CLI must remain complete |
| Run script/suite with gates | ✅ | ⚠️ | GUI should call the same tooling engine APIs |
| Pack bounded repro zip / AI packet | ✅ | ⚠️ | GUI can be a thin wrapper |
| Inspect/pick selector UX | ⚠️ | ⚠️ | Both exist; GUI should make it “everyday” |
| View bundle meta/index quickly | ✅ | ⚠️ | GUI should reuse `diag meta/index/slice` outputs |

---

## 3) Platform parity (native vs wasm)

| Outcome | Native | wasm/web runner | Notes |
| --- | --- | --- | --- |
| Run scripts deterministically | ✅ | ⚠️ | Requires WS transport; filesystem triggers not available |
| Local artifact materialization | ✅ | ✅ | For web runner, materialization is host-side |
| Screenshots | ✅ | ⚠️ | GPU readback may differ; keep optional |
| Layout sidecars (Taffy dump) | ✅ | ❌ | Likely native-only at first |

---

## 4) Layout debugging parity (correctness vs performance)

| Outcome | Status | Notes |
| --- | --- | --- |
| Layout correctness gates via semantics bounds predicates | ⚠️ | Must become the primary “layout regression” gate style |
| Layout performance gates (solve/measure hot paths) | ✅ | Already supported via perf tooling; keep readable evidence |
| Bundle-scoped layout sidecars (explainability) | ❌ | Planned: tie Taffy dump (or summary) to repro artifacts |

