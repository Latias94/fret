# Evidence-first triage checklist

Start here (portable artifacts):

- `script.result.json`: `reason_code`, failing `step_index`, bounded evidence ring-buffers.
- `bundle.json`: frame snapshots (semantics/layout/stats/debug surfaces).
- layout sidecar artifacts from `capture_layout_sidecar`: width/height/flex/min-size ownership when visual diffs are ambiguous.
- `check.lint.json`: bundle lint findings (often a fast “why is the tree weird?” answer).

## Common evidence fields (script.result.json)

- `evidence.selector_resolution_trace`: why a selector matched (or didn’t), with top-N candidates.
- `evidence.hit_test_trace`: injected pointer position vs hit chain, including:
  - attribution (`blocking_reason` / `blocking_root` / `blocking_layer_id`),
  - `routing_explain` hint string,
  - capture/occlusion owners (best-effort; in-run references only),
  - capture owner element path (`pointer_capture_element_path`, best-effort).
- `evidence.click_stable_trace`: stable-click decision trace (including hit-test mismatch probes).
- `evidence.bounds_stable_trace`: bounds-stability trace for `wait_bounds_stable` steps.
- `evidence.focus_trace`: focused identity + `text_input_snapshot` + barrier/capture hints.
- `evidence.shortcut_routing_trace`: why a keydown went to IME/widget path vs command dispatch.
- `evidence.overlay_placement_trace`: overlay placement decisions (geometry-first).
- `evidence.ime_event_trace` / `evidence.web_ime_trace`: redaction-friendly IME summaries.

## Reason-code first debugging

- `selector.not_found`
  - Inspect `selector_resolution_trace` and then run `diag lint` for duplicate/missing `test_id`.
- layout / clipping drift
  - Inspect the layout sidecar first to see which node owns width, height, flex, min-size, and overflow before changing tokens or screenshot baselines.
- `timeout`
  - Prefer adding an intermediate `capture_bundle` near the “interesting point”.
  - Replace sleeps with `wait_until`, `click_stable`, `wait_bounds_stable`.
- “click didn’t land” / routing issues
  - Inspect `hit_test_trace` → start with `routing_explain`, then check capture/occlusion owners.
- Focus / typing stalls
  - Inspect `focus_trace` + `text_input_snapshot` + `shortcut_routing_trace`.

## Fallback when you only have bundle.json (avoid rg/grep explosions)

Prefer bounded tooling (`diag meta/query/slice/ai-packet`) when available. If you cannot run `fretboard` but you do
have a local `bundle.json`, this is a bounded “find a needle” fallback (prints up to 50 matches):

```powershell
@'
import json,sys
from collections import deque
p, needle = sys.argv[1], sys.argv[2]
data = json.load(open(p, encoding="utf-8"))
out = []
st = deque([("", data)])
while st and len(out) < 50:
    path, v = st.pop()
    if isinstance(v, dict):
        for k, vv in v.items():
            st.append(((path + "." + k) if path else k, vv))
    elif isinstance(v, list):
        for i, vv in enumerate(v):
            st.append((f"{path}[{i}]", vv))
    else:
        s = str(v)
        if needle in s:
            out.append((path, s[:160]))
for a, b in out:
    print(f"{a} => {b}")
print("matches", len(out))
'@ | python - <bundle.json> <needle>
```
