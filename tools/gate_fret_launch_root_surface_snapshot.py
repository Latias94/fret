from __future__ import annotations

import re
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "fret-launch root surface snapshot"
LIB_RS = WORKSPACE_ROOT / "crates/fret-launch/src/lib.rs"
EXPECTED_PUBLIC_MODULES = {
    "assets",
    "dev_state",
    "imported_viewport_target",
    "native_external_import",
    "shared_allocation",
    "media",
}
EXPECTED_CORE_RUNNER_EXPORTS = {
    "EngineFrameKeepalive",
    "EngineFrameUpdate",
    "FnDriver",
    "FnDriverHooks",
    "RenderTargetUpdate",
    "ViewportOverlay3dHooks",
    "ViewportOverlay3dHooksService",
    "ViewportOverlay3dImmediateService",
    "ViewportRenderTarget",
    "ViewportRenderTargetWithDepth",
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
    "install_viewport_overlay_3d_immediate",
    "record_viewport_overlay_3d",
    "run_app",
    "run_app_with_event_loop",
    "upload_viewport_overlay_3d_immediate",
}
EXPECTED_DEV_STATE_EXPORTS = {
    "DevStateExport",
    "DevStateHook",
    "DevStateHooks",
    "DevStateService",
    "DevStateWindowKeyRegistry",
}
EXPECTED_NATIVE_RUNNER_EXPORTS = {"WinitAppBuilder"}
EXPECTED_WASM_RUNNER_EXPORTS = {"WebRunnerHandle", "run_app_with_handle"}
PUBLIC_MODULE_RE = re.compile(
    r'(?ms)^(?:#\[[^\n]+\]\s*\n)*pub\s+mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:;|\{)'
)
CORE_RUNNER_BLOCK_RE = re.compile(r'pub\s+use\s+runner::\{(?P<exports>.*?)\};', flags=re.S)
DEV_STATE_EXPORT_RE = re.compile(r'(?m)^pub\s+use\s+dev_state::(?P<exports>[^;]+);')
NATIVE_RUNNER_EXPORT_RE = re.compile(
    r'#\[cfg\(not\(target_arch = "wasm32"\)\)\]\s*pub\s+use\s+runner::(?P<exports>[^;]+);',
    flags=re.M,
)
WASM_RUNNER_EXPORT_RE = re.compile(
    r'#\[cfg\(target_arch = "wasm32"\)\]\s*pub\s+use\s+runner::\{(?P<exports>.*?)\};',
    flags=re.S,
)
ROOT_USE_RE = re.compile(r'(?m)^pub\s+use\s+([A-Za-z_][A-Za-z0-9_]*)::')
IDENT_RE = re.compile(r'\b[A-Za-z_][A-Za-z0-9_]*\b')


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.relative_to(WORKSPACE_ROOT)}: {exc}")


def parse_ident_list(block: str) -> set[str]:
    block = re.sub(r'(?m)^\s*//.*$', '', block)
    return set(IDENT_RE.findall(block))


def diff_message(label: str, actual: set[str], expected: set[str]) -> list[str]:
    problems: list[str] = []
    missing = sorted(expected - actual)
    extra = sorted(actual - expected)
    if missing:
        problems.append(f"missing {label}: {', '.join(missing)}")
    if extra:
        problems.append(f"unexpected {label}: {', '.join(extra)}")
    return problems


def main() -> None:
    text = read_text(LIB_RS)
    problems: list[str] = []

    modules = {match.group(1) for match in PUBLIC_MODULE_RE.finditer(text)}
    problems.extend(diff_message("top-level public modules", modules, EXPECTED_PUBLIC_MODULES))

    core_match = CORE_RUNNER_BLOCK_RE.search(text)
    if core_match is None:
        fail(GATE_NAME, "missing `pub use runner::{...};` root export block")
    core_exports = parse_ident_list(core_match.group("exports"))
    problems.extend(diff_message("core runner exports", core_exports, EXPECTED_CORE_RUNNER_EXPORTS))

    dev_state_exports: set[str] = set()
    for match in DEV_STATE_EXPORT_RE.finditer(text):
        dev_state_exports.update(parse_ident_list(match.group("exports")))
    problems.extend(diff_message("dev-state root exports", dev_state_exports, EXPECTED_DEV_STATE_EXPORTS))

    native_match = NATIVE_RUNNER_EXPORT_RE.search(text)
    native_exports = set() if native_match is None else parse_ident_list(native_match.group("exports"))
    problems.extend(diff_message("native-only root runner exports", native_exports, EXPECTED_NATIVE_RUNNER_EXPORTS))

    wasm_match = WASM_RUNNER_EXPORT_RE.search(text)
    wasm_exports = set() if wasm_match is None else parse_ident_list(wasm_match.group("exports"))
    problems.extend(diff_message("wasm-only root runner exports", wasm_exports, EXPECTED_WASM_RUNNER_EXPORTS))

    leaked_roots = sorted({root for root in ROOT_USE_RE.findall(text) if root not in {"error", "dev_state", "runner"}})
    if leaked_roots:
        problems.append(f"unexpected top-level `pub use` roots: {', '.join(leaked_roots)}")

    if problems:
        print(f"[gate] {GATE_NAME}")
        print(f"[gate] FAIL: {len(problems)} issue(s)")
        for problem in problems:
            print(f"  - {problem}")
        raise SystemExit(1)

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
