from __future__ import annotations

import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import diag_u8_text_budget_gate as gate


def _args(**overrides: object):
    class Args:
        out_dir = ""
        dry_run = True
        skip_native = False
        skip_text_heavy = False
        skip_code_editor = False
        repeat = 3
        timeout_ms = 900_000
        fretboard_cmd = "fretboard-dev"
        text_heavy_script = gate.TEXT_HEAVY_SCRIPT
        code_editor_script = gate.CODE_EDITOR_SCRIPT
        text_heavy_launch_cmd = gate.TEXT_HEAVY_LAUNCH_CMD
        code_editor_launch_cmd = gate.CODE_EDITOR_LAUNCH_CMD
        native_atlas_max_pages = 2
        text_heavy_max_atlas_bytes = 48 * gate.MIB
        text_heavy_max_shape_cache_entries = 4096
        text_heavy_max_shape_cache_bytes = 32 * gate.MIB
        code_editor_max_atlas_bytes = 16 * gate.MIB
        code_editor_max_shape_cache_entries = 4096
        code_editor_max_shape_cache_bytes = 16 * gate.MIB
        web_export_bundle: list[str] = []
        web_max_shape_cache_entry_limit = 1024
        web_max_atlas_pages = 1
        web_max_text_atlas_evicted_pages = 0
        out_report = ""

    args = Args()
    for key, value in overrides.items():
        setattr(args, key, value)
    return args


def _bundle(
    *,
    shape_limit: int = 1024,
    max_pages: int = 1,
    evicted_pages: int = 0,
    include_upload_metrics: bool = True,
) -> dict[str, object]:
    stats: dict[str, object] = {}
    if include_upload_metrics:
        stats.update(
            {
                "renderer_prepare_text_us": 10,
                "renderer_text_atlas_upload_bytes": 2048,
                "renderer_text_atlas_evicted_pages": evicted_pages,
                "renderer_geometry_upload_text_glyph_instance_bytes": 512,
                "renderer_geometry_upload_text_glyph_instance_write_count": 1,
                "renderer_geometry_upload_text_vertex_bytes": 1024,
                "renderer_geometry_upload_text_vertex_write_count": 1,
                "renderer_encode_scene_text_ops": 12,
            }
        )
    return {
        "schema_version": 2,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "resource_caches": {
                            "render_text": {
                                "shape_cache_entries": 64,
                                "shape_cache_entry_limit": shape_limit,
                                "shape_cache_bytes_estimate_total": 4096,
                                "frame_shape_cache_evictions": 0,
                                "mask_atlas": {
                                    "width": 16,
                                    "height": 16,
                                    "pages": 1,
                                    "max_pages": max_pages,
                                },
                                "color_atlas": {
                                    "width": 16,
                                    "height": 16,
                                    "pages": 1,
                                    "max_pages": max_pages,
                                },
                                "subpixel_atlas": {
                                    "width": 16,
                                    "height": 16,
                                    "pages": 1,
                                    "max_pages": max_pages,
                                },
                            }
                        },
                        "debug": {"stats": stats},
                    }
                ],
            }
        ],
    }


class U8TextBudgetGateTests(unittest.TestCase):
    def test_native_probe_defaults_cover_text_and_code_editor_budgets(self) -> None:
        probes = gate.native_probes(_args())

        self.assertEqual(["text-heavy", "code-editor"], [probe.name for probe in probes])
        text_thresholds = dict(probes[0].thresholds)
        editor_thresholds = dict(probes[1].thresholds)
        self.assertEqual(48 * gate.MIB, text_thresholds["render_text_atlas_bytes_live_estimate_total"])
        self.assertEqual(16 * gate.MIB, editor_thresholds["render_text_atlas_bytes_live_estimate_total"])
        self.assertEqual(4096, editor_thresholds["render_text_shape_cache_entries"])
        self.assertEqual(2, editor_thresholds["render_text_mask_atlas_max_pages"])

    def test_repeat_command_uses_existing_memory_p90_gate(self) -> None:
        probe = gate.native_probes(_args(skip_code_editor=True))[0]
        cmd = gate.build_repeat_cmd(
            fretboard_cmd="fretboard-dev",
            probe=probe,
            out_dir=Path("target/u8-text-budget/text-heavy"),
            repeat=5,
            timeout_ms=1234,
        )

        self.assertIn("repeat", cmd)
        self.assertIn("--no-compare", cmd)
        self.assertIn("--check-memory-p90-max", cmd)
        self.assertIn("render_text_shape_cache_entries:4096", cmd)
        self.assertIn("--launch", cmd)
        self.assertEqual("5", cmd[cmd.index("--repeat") + 1])
        self.assertEqual("1234", cmd[cmd.index("--timeout-ms") + 1])

    def test_web_bundle_accepts_wasm_text_budget_metrics(self) -> None:
        with TemporaryDirectory() as td:
            path = Path(td) / "bundle.json"
            path.write_text(json.dumps(_bundle()), encoding="utf-8")

            report = gate.validate_web_bundle(path, _args())

        self.assertTrue(report["ok"], report)
        observed = report["observed"]
        self.assertEqual(1024, observed["render_text_shape_cache_entry_limit"])
        self.assertEqual(1, observed["render_text_mask_atlas_max_pages"])
        self.assertEqual(0, observed["renderer_text_atlas_evicted_pages"])

    def test_web_bundle_rejects_native_sized_wasm_budgets(self) -> None:
        with TemporaryDirectory() as td:
            path = Path(td) / "bundle.json"
            path.write_text(json.dumps(_bundle(shape_limit=4096, max_pages=2)), encoding="utf-8")

            report = gate.validate_web_bundle(path, _args())

        self.assertFalse(report["ok"])
        failure_metrics = {failure.get("metric") for failure in report["failures"]}
        self.assertIn("render_text_shape_cache_entry_limit", failure_metrics)
        self.assertIn("render_text_mask_atlas_max_pages", failure_metrics)

    def test_web_bundle_requires_upload_metrics(self) -> None:
        with TemporaryDirectory() as td:
            path = Path(td) / "bundle.json"
            path.write_text(json.dumps(_bundle(include_upload_metrics=False)), encoding="utf-8")

            report = gate.validate_web_bundle(path, _args())

        self.assertFalse(report["ok"])
        failure_metrics = {failure.get("metric") for failure in report["failures"]}
        self.assertIn("renderer_text_atlas_upload_bytes", failure_metrics)
        self.assertIn("renderer_geometry_upload_text_vertex_write_count", failure_metrics)

    def test_dry_run_summary_does_not_execute_native_commands(self) -> None:
        args = _args(
            skip_code_editor=True,
            out_dir="target/u8-text-budget-dry-run",
            text_heavy_script="tools/diag-scripts/tooling/text/text-heavy-memory-steady.json",
        )

        summary = gate.build_summary(Path.cwd(), args)

        self.assertTrue(summary["ok"], summary)
        self.assertTrue(summary["dry_run"])
        self.assertEqual(1, len(summary["native"]["probes"]))
        self.assertIsNone(summary["native"]["probes"][0]["rc"])


if __name__ == "__main__":
    unittest.main()
