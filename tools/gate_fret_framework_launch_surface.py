from __future__ import annotations

import re
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "fret-framework curated launch facade"
LIB_RS = WORKSPACE_ROOT / "crates/fret-framework/src/lib.rs"


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.relative_to(WORKSPACE_ROOT)}: {exc}")


def main() -> None:
    text = read_text(LIB_RS)
    problems: list[str] = []

    launch_block = re.search(
        r"#\[cfg\(feature = \"launch\"\)\]\s*pub\s+mod\s+launch\s*\{(?P<body>.*?)\n\}",
        text,
        flags=re.S,
    )
    if launch_block is None:
        fail(GATE_NAME, "missing `#[cfg(feature = \"launch\")] pub mod launch { ... }` block")
    body = re.sub(r"(?m)^\s*//!.*$", "", launch_block.group("body"))
    body = re.sub(r"(?m)^\s*//(?!\!).*$", "", body)

    required_exports = [
        "FnDriver",
        "FnDriverHooks",
        "RunnerError",
        "WgpuInit",
        "WindowCreateSpec",
        "WinitCommandContext",
        "WinitEventContext",
        "WinitGlobalContext",
        "WinitHotReloadContext",
        "WinitRenderContext",
        "WinitRunnerConfig",
        "WinitWindowContext",
    ]
    for export_name in required_exports:
        if re.search(rf"\b{re.escape(export_name)}\b", body) is None:
            problems.append(f"missing curated launch export `{export_name}`")

    forbidden_exports = [
        "WinitAppDriver",
        "windows_mf_video",
        "apple_avfoundation_video",
        "android_mediacodec_video",
        "dx12",
        "ImportedViewportRenderTarget",
        "NativeExternalTextureFrame",
    ]
    for export_name in forbidden_exports:
        if re.search(rf"\b{re.escape(export_name)}\b", body) is not None:
            problems.append(
                f"forbidden launch-facade export `{export_name}` leaked into `fret-framework::launch`"
            )

    if problems:
        print(f"[gate] {GATE_NAME}")
        print(f"[gate] FAIL: {len(problems)} issue(s)")
        for problem in problems:
            print(f"  - {problem}")
        raise SystemExit(1)

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
