#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import sys
import textwrap
import unittest
from pathlib import Path
from types import ModuleType


def load_consumption_profiles_module() -> ModuleType:
    path = Path(__file__).with_name("check_consumption_profiles.py")
    spec = importlib.util.spec_from_file_location("check_consumption_profiles", path)
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


PROFILES = load_consumption_profiles_module()


class ConsumptionProfileTests(unittest.TestCase):
    def test_tree_without_banned_backend_packages_passes(self) -> None:
        tree = textwrap.dedent(
            """
            fret v0.1.0 (/repo/ecosystem/fret)
            fret-core v0.1.0 (/repo/crates/fret-core)
            fret-runtime v0.1.0 (/repo/crates/fret-runtime)
            """
        ).strip()

        PROFILES._assert_tree_excludes("portable", tree, {"wgpu", "fret-runner-web"})

    def test_tree_with_native_renderer_backend_is_rejected(self) -> None:
        tree = textwrap.dedent(
            """
            fret v0.1.0 (/repo/ecosystem/fret)
            fret-render-wgpu v0.1.0 (/repo/crates/fret-render-wgpu)
            wgpu v26.0.1
            """
        ).strip()

        with self.assertRaisesRegex(SystemExit, "fret-render-wgpu, wgpu"):
            PROFILES._assert_tree_excludes("backend-free", tree, {"fret-render-wgpu", "wgpu"})

    def test_tree_with_web_backend_is_rejected(self) -> None:
        tree = textwrap.dedent(
            """
            fret v0.1.0 (/repo/ecosystem/fret)
            fret-platform-web v0.1.0 (/repo/crates/fret-platform-web)
            fret-runner-web v0.1.0 (/repo/crates/fret-runner-web)
            """
        ).strip()

        with self.assertRaisesRegex(SystemExit, "fret-platform-web, fret-runner-web"):
            PROFILES._assert_tree_excludes(
                "backend-free",
                tree,
                {"fret-platform-web", "fret-runner-web"},
            )


if __name__ == "__main__":
    raise SystemExit(unittest.main())
