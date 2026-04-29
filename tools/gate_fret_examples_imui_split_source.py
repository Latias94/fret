from __future__ import annotations

from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "fret examples imui split source"

NEW_DEMO_SOURCE = Path("apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs")
OLD_DEMO_SOURCE = Path("apps/fret-examples/src/imui_shadcn_adapter_demo.rs")


def read(path: Path) -> str:
    full_path = WORKSPACE_ROOT / path
    try:
        return full_path.read_text(encoding="utf-8")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.as_posix()}: {exc}")


def require(text: str, marker: str, path: Path, failures: list[str]) -> None:
    if marker not in text:
        failures.append(f"{path.as_posix()}: missing {marker}")


def forbid(text: str, marker: str, path: Path, failures: list[str]) -> None:
    if marker in text:
        failures.append(f"{path.as_posix()}: forbidden {marker}")


def main() -> None:
    failures: list[str] = []

    if not (WORKSPACE_ROOT / NEW_DEMO_SOURCE).is_file():
        failures.append(f"{NEW_DEMO_SOURCE.as_posix()}: missing moved IMUI demo source")
    if (WORKSPACE_ROOT / OLD_DEMO_SOURCE).exists():
        failures.append(f"{OLD_DEMO_SOURCE.as_posix()}: monolithic source copy should not exist")

    imui_cargo = read(Path("apps/fret-examples-imui/Cargo.toml"))
    require(imui_cargo, 'name = "fret-examples-imui"', Path("apps/fret-examples-imui/Cargo.toml"), failures)
    forbid(imui_cargo, "fret-examples =", Path("apps/fret-examples-imui/Cargo.toml"), failures)

    imui_lib = read(Path("apps/fret-examples-imui/src/lib.rs"))
    require(imui_lib, "pub mod imui_shadcn_adapter_demo;", Path("apps/fret-examples-imui/src/lib.rs"), failures)

    imui_bin = read(Path("apps/fret-examples-imui/src/bin/imui_shadcn_adapter_demo.rs"))
    require(
        imui_bin,
        "fret_examples_imui::imui_shadcn_adapter_demo::run()",
        Path("apps/fret-examples-imui/src/bin/imui_shadcn_adapter_demo.rs"),
        failures,
    )

    examples_cargo = read(Path("apps/fret-examples/Cargo.toml"))
    require(
        examples_cargo,
        'fret-examples-imui = { path = "../fret-examples-imui" }',
        Path("apps/fret-examples/Cargo.toml"),
        failures,
    )

    examples_lib = read(Path("apps/fret-examples/src/lib.rs"))
    require(
        examples_lib,
        "pub use fret_examples_imui::imui_shadcn_adapter_demo;",
        Path("apps/fret-examples/src/lib.rs"),
        failures,
    )
    require(
        examples_lib,
        'include_str!("../../fret-examples-imui/src/imui_shadcn_adapter_demo.rs")',
        Path("apps/fret-examples/src/lib.rs"),
        failures,
    )

    if failures:
        fail(GATE_NAME, f"{len(failures)} split marker problem(s):\n  - " + "\n  - ".join(failures))

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
