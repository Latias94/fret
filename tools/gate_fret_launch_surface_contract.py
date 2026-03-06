from __future__ import annotations

import re
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "fret-launch curated root surface"
LIB_RS = WORKSPACE_ROOT / "crates/fret-launch/src/lib.rs"


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.relative_to(WORKSPACE_ROOT)}: {exc}")


def require_pattern(text: str, pattern: str, message: str, *, flags: int = 0) -> list[str]:
    if re.search(pattern, text, flags=flags) is None:
        return [message]
    return []


def forbid_pattern(text: str, pattern: str, message: str, *, flags: int = 0) -> list[str]:
    if re.search(pattern, text, flags=flags) is not None:
        return [message]
    return []


def main() -> None:
    text = read_text(LIB_RS)
    problems: list[str] = []

    problems.extend(
        forbid_pattern(
            text,
            r"\bpub\s+mod\s+runner\b",
            "`runner` must remain internal plumbing, not a public root module",
        )
    )
    problems.extend(
        forbid_pattern(
            text,
            r"\bpub\s+use\s+runner::\*",
            "crate root must not wildcard re-export `runner::*`",
        )
    )

    for module_name in [
        "imported_viewport_target",
        "native_external_import",
        "media",
    ]:
        problems.extend(
            require_pattern(
                text,
                rf"\bpub\s+mod\s+{module_name}\s*\{{",
                f"missing specialized root module `{module_name}`",
            )
        )

    problems.extend(
        require_pattern(
            text,
            r"#\[cfg\(not\(target_arch = \"wasm32\"\)\)\]\s*pub\s+mod\s+shared_allocation\s*\{",
            "missing cfg-gated specialized root module `shared_allocation`",
            flags=re.MULTILINE,
        )
    )

    core_block_match = re.search(r"pub\s+use\s+runner::\{(?P<block>.*?)\};", text, flags=re.S)
    if core_block_match is None:
        problems.append("missing root `pub use runner::{...};` core export block")
        core_block = ""
    else:
        core_block = core_block_match.group("block")

    required_core_exports = [
        "FnDriver",
        "FnDriverHooks",
        "WgpuInit",
        "WindowCreateSpec",
        "WindowLogicalSize",
        "WindowPhysicalPosition",
        "WindowPosition",
        "WinitAppDriver",
        "WinitCommandContext",
        "WinitEventContext",
        "WinitGlobalContext",
        "WinitHotReloadContext",
        "WinitRenderContext",
        "WinitRunnerConfig",
        "WinitWindowContext",
        "run_app",
        "run_app_with_event_loop",
    ]
    for export_name in required_core_exports:
        if re.search(rf"\b{re.escape(export_name)}\b", core_block) is None:
            problems.append(f"missing core root export `{export_name}` in `pub use runner::{{...}}` block")

    problems.extend(
        require_pattern(
            text,
            r"\bpub\s+use\s+error::RunnerError;",
            "missing root export `RunnerError` from `error` module",
        )
    )

    forbidden_core_exports = [
        "WinitRunner",
        "RunnerUserEvent",
        "run_app_with_event_loop_and_handle",
    ]
    for export_name in forbidden_core_exports:
        if re.search(rf"\b{re.escape(export_name)}\b", core_block) is not None:
            problems.append(f"forbidden root export `{export_name}` leaked into core `pub use runner::{{...}}` block")

    problems.extend(
        require_pattern(
            text,
            r"#\[cfg\(not\(target_arch = \"wasm32\"\)\)\]\s*pub\s+use\s+runner::WinitAppBuilder;",
            "missing native root re-export `WinitAppBuilder`",
            flags=re.MULTILINE,
        )
    )
    problems.extend(
        require_pattern(
            text,
            r"#\[cfg\(target_arch = \"wasm32\"\)\]\s*pub\s+use\s+runner::\{WebRunnerHandle,\s*run_app_with_handle\};",
            "missing wasm root exports `WebRunnerHandle` / `run_app_with_handle`",
            flags=re.MULTILINE,
        )
    )

    forbidden_root_reexports = [
        r"\bpub\s+use\s+runner::WinitRunner\b",
        r"\bpub\s+use\s+runner::RunnerUserEvent\b",
        r"\bpub\s+use\s+runner::run_app_with_event_loop_and_handle\b",
        r"\bpub\s+use\s+runner::windows_mf_video\b",
        r"\bpub\s+use\s+runner::apple_avfoundation_video\b",
        r"\bpub\s+use\s+runner::android_mediacodec_video\b",
        r"\bpub\s+use\s+runner::dx12\b",
    ]
    for pattern in forbidden_root_reexports:
        problems.extend(
            forbid_pattern(
                text,
                pattern,
                f"forbidden direct root re-export matched `{pattern}`",
            )
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
