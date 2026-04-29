from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "examples source tree policy"

EXAMPLES_SRC = Path("apps/fret-examples/src")
IMUI_EXAMPLES_SRC = Path("apps/fret-examples-imui/src")
EXAMPLES_SOURCE_ROOTS = [EXAMPLES_SRC, IMUI_EXAMPLES_SRC]
EXCLUDED_SOURCES = {
    "apps/fret-examples/src/lib.rs",
    "apps/fret-examples-imui/src/lib.rs",
}

ALLOWED_FRET_UI_SHADCN_IMPORTS = {
    "use fret_ui_shadcn::facade as shadcn;",
    "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
}

ALLOWED_RAW_SHADCN_ESCAPES = [
    "shadcn::raw::typography::",
    "shadcn::raw::extras::",
    "fret::shadcn::raw::prelude::",
    "shadcn::raw::advanced::sync_theme_from_environment(",
    "fret::shadcn::raw::advanced::sync_theme_from_environment(",
    "shadcn::raw::advanced::install_with_ui_services(",
    "fret::shadcn::raw::advanced::install_with_ui_services(",
]

RAW_ACTION_NOTIFY_MARKERS = [
    "use fret::advanced::AppUiRawActionNotifyExt as _;",
    "cx.on_action_notify::<",
    "cx.on_payload_action_notify::<",
]

FIRST_PARTY_CURATED_SHADCN_SURFACES = [
    EXAMPLES_SRC / "assets_demo.rs",
    EXAMPLES_SRC / "async_playground_demo.rs",
    EXAMPLES_SRC / "cjk_conformance_demo.rs",
    EXAMPLES_SRC / "components_gallery.rs",
    EXAMPLES_SRC / "custom_effect_v1_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_glass_chrome_web_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_identity_web_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_lut_web_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_web_demo.rs",
    EXAMPLES_SRC / "custom_effect_v3_demo.rs",
    EXAMPLES_SRC / "docking_arbitration_demo.rs",
    EXAMPLES_SRC / "docking_demo.rs",
    EXAMPLES_SRC / "drop_shadow_demo.rs",
    EXAMPLES_SRC / "embedded_viewport_demo.rs",
    EXAMPLES_SRC / "emoji_conformance_demo.rs",
    EXAMPLES_SRC / "genui_demo.rs",
    EXAMPLES_SRC / "hello_counter_demo.rs",
    EXAMPLES_SRC / "ime_smoke_demo.rs",
    EXAMPLES_SRC / "imui_editor_proof_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_interaction_showcase_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_shadcn_adapter_demo.rs",
    EXAMPLES_SRC / "liquid_glass_demo.rs",
    EXAMPLES_SRC / "markdown_demo.rs",
    EXAMPLES_SRC / "postprocess_theme_demo.rs",
    EXAMPLES_SRC / "query_async_tokio_demo.rs",
    EXAMPLES_SRC / "simple_todo_demo.rs",
    EXAMPLES_SRC / "sonner_demo.rs",
]

CURATED_SHADCN_FORBIDDEN_MARKERS = [
    "use fret_ui_shadcn as shadcn;",
    "use fret_ui_shadcn::{self as shadcn",
    "shadcn::shadcn_themes::",
    "shadcn::typography::",
]


@dataclass(frozen=True)
class Failure:
    path: Path
    line_no: int | None
    message: str
    line: str | None = None


def normalize(text: str) -> str:
    return "".join(text.split())


def rel_path(path: Path) -> Path:
    return path.resolve().relative_to(WORKSPACE_ROOT)


def read_source(path: Path) -> str:
    full_path = path if path.is_absolute() else WORKSPACE_ROOT / path
    try:
        return full_path.read_text(encoding="utf-8")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {rel_path(full_path).as_posix()}: {exc}")


def examples_rust_sources() -> list[Path]:
    paths: list[Path] = []
    for root in EXAMPLES_SOURCE_ROOTS:
        paths.extend((WORKSPACE_ROOT / root).rglob("*.rs"))
    return sorted(path for path in paths if rel_path(path).as_posix() not in EXCLUDED_SOURCES)


def push_line_failure(
    failures: list[Failure],
    path: Path,
    line_no: int,
    message: str,
    line: str,
) -> None:
    failures.append(Failure(rel_path(path), line_no, message, line.strip()))


def check_source_tree_policies(path: Path, source: str, failures: list[Failure]) -> None:
    if "use fret_ui_shadcn as shadcn;" in source:
        failures.append(
            Failure(rel_path(path), None, "reintroduced `use fret_ui_shadcn as shadcn;`")
        )
    if "use fret_ui_shadcn::{self as shadcn" in source:
        failures.append(
            Failure(rel_path(path), None, "reintroduced `use fret_ui_shadcn::{self as shadcn`")
        )

    for line_no, line in enumerate(source.splitlines(), start=1):
        trimmed = line.strip()

        if "fret_ui_shadcn::" in line and trimmed not in ALLOWED_FRET_UI_SHADCN_IMPORTS:
            push_line_failure(
                failures,
                path,
                line_no,
                "non-curated fret_ui_shadcn import",
                line,
            )

        if "shadcn::raw::" in trimmed or "fret::shadcn::raw::" in trimmed:
            if not any(marker in trimmed for marker in ALLOWED_RAW_SHADCN_ESCAPES):
                push_line_failure(
                    failures,
                    path,
                    line_no,
                    "undocumented shadcn raw escape hatch",
                    line,
                )

        for marker in RAW_ACTION_NOTIFY_MARKERS:
            if marker in line:
                push_line_failure(
                    failures,
                    path,
                    line_no,
                    f"raw action notify helper: {marker}",
                    line,
                )

    normalized = normalize(source)
    if ".setup(|" in normalized:
        failures.append(Failure(rel_path(path), None, "inline `.setup(|...)` closure"))
    if ".setup(move|" in normalized:
        failures.append(Failure(rel_path(path), None, "inline `.setup(move |...)` closure"))

    if ".setup_with(" in normalized:
        if path.name != "imui_editor_proof_demo.rs":
            failures.append(Failure(rel_path(path), None, "unexpected `.setup_with(...)` usage"))
        elif ".setup_with(move|" not in normalized:
            failures.append(
                Failure(
                    rel_path(path),
                    None,
                    "`imui_editor_proof_demo.rs` should keep `.setup_with(move |...)`",
                )
            )


def check_first_party_curated_shadcn_surfaces(failures: list[Failure]) -> None:
    for relative_path in FIRST_PARTY_CURATED_SHADCN_SURFACES:
        path = WORKSPACE_ROOT / relative_path
        source = read_source(path)
        for marker in CURATED_SHADCN_FORBIDDEN_MARKERS:
            if marker in source:
                failures.append(
                    Failure(
                        rel_path(path),
                        None,
                        f"first-party shadcn surface used forbidden marker: {marker}",
                    )
                )


def print_failures(failures: list[Failure]) -> None:
    if not failures:
        return

    print(f"[gate] {GATE_NAME}")
    print(f"[gate] FAIL: {len(failures)} source policy problem(s)")
    for failure in failures[:60]:
        location = failure.path.as_posix()
        if failure.line_no is not None:
            location = f"{location}:{failure.line_no}"
        print(f"  - {location}: {failure.message}")
        if failure.line is not None:
            print(f"      {failure.line}")
    if len(failures) > 60:
        print(f"  ... and {len(failures) - 60} more")


def main() -> None:
    failures: list[Failure] = []
    for path in examples_rust_sources():
        check_source_tree_policies(path, read_source(path), failures)
    check_first_party_curated_shadcn_surfaces(failures)

    print_failures(failures)
    if failures:
        raise SystemExit(1)

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
