from __future__ import annotations

from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "fret examples imui split source"

MOVED_DEMOS = [
    "imui_floating_windows_demo",
    "imui_hello_demo",
    "imui_shadcn_adapter_demo",
]


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

    for demo in MOVED_DEMOS:
        new_source = Path(f"apps/fret-examples-imui/src/{demo}.rs")
        old_source = Path(f"apps/fret-examples/src/{demo}.rs")
        if not (WORKSPACE_ROOT / new_source).is_file():
            failures.append(f"{new_source.as_posix()}: missing moved IMUI demo source")
        if (WORKSPACE_ROOT / old_source).exists():
            failures.append(f"{old_source.as_posix()}: monolithic source copy should not exist")

    imui_cargo = read(Path("apps/fret-examples-imui/Cargo.toml"))
    require(imui_cargo, 'name = "fret-examples-imui"', Path("apps/fret-examples-imui/Cargo.toml"), failures)
    forbid(imui_cargo, "fret-examples =", Path("apps/fret-examples-imui/Cargo.toml"), failures)

    imui_lib = read(Path("apps/fret-examples-imui/src/lib.rs"))
    for demo in MOVED_DEMOS:
        require(imui_lib, f"pub mod {demo};", Path("apps/fret-examples-imui/src/lib.rs"), failures)

        bin_path = Path(f"apps/fret-examples-imui/src/bin/{demo}.rs")
        imui_bin = read(bin_path)
        require(imui_bin, f"fret_examples_imui::{demo}::run()", bin_path, failures)

    examples_cargo = read(Path("apps/fret-examples/Cargo.toml"))
    require(
        examples_cargo,
        'fret-examples-imui = { path = "../fret-examples-imui" }',
        Path("apps/fret-examples/Cargo.toml"),
        failures,
    )

    examples_lib = read(Path("apps/fret-examples/src/lib.rs"))
    for demo in MOVED_DEMOS:
        require(
            examples_lib,
            f"pub use fret_examples_imui::{demo};",
            Path("apps/fret-examples/src/lib.rs"),
            failures,
        )
        require(
            examples_lib,
            f'include_str!("../../fret-examples-imui/src/{demo}.rs")',
            Path("apps/fret-examples/src/lib.rs"),
            failures,
        )

    if failures:
        fail(GATE_NAME, f"{len(failures)} split marker problem(s):\n  - " + "\n  - ".join(failures))

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
