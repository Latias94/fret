import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

import diag_editor_paint_contract_validate as validate
import diag_editor_paint_contract_verify_artifacts as verify


def _write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value), encoding="utf-8")


def _step(
    root: Path,
    name: str,
    *,
    with_paint_perf: bool,
    repeat: int,
    torture_overlay_us: int = 0,
    stale_summary_paths: bool = False,
) -> dict[str, object]:
    step_dir = root / name
    check_path = step_dir / "check.perf_thresholds.json"
    stats_path = root / "runner-logs" / name / "stats.stdout.json"
    _write_json(check_path, {"failures": []})
    stats_doc: dict[str, object] = {
        "p95": {
            "paint_widget_time_us": 10,
            "renderer_prepare_text_us": 11,
            "renderer_encode_scene_us": 12,
            "renderer_upload_us": 13,
        }
    }
    if with_paint_perf:
        stats_doc["code_editor_paint_perf"] = {
            "max": {"us_torture_overlay": torture_overlay_us},
            "p95": {"us_total": 14},
        }
    _write_json(stats_path, stats_doc)

    plan = validate.build_plan(
        python_bin="python",
        fretboard_bin=validate._default_fretboard_bin(),
        launch_cmd=validate._default_launch_cmd(),
        out_dir=str(root),
        resize_attempts=3,
        resize_repeat=repeat if name == "resize-jitter" else 7,
        typical_repeat=repeat if name == "typical-autoscroll" else 15,
        complex_repeat=repeat if name == "complex-wheel" else 7,
        warmup_frames=5,
        skip_preflight=True,
        with_paint_perf=with_paint_perf,
    )
    cmd = {str(step["name"]): list(step["cmd"]) for step in plan}[name]

    summary_check_path = Path("Z:/target-machine/stale/check.perf_thresholds.json") if stale_summary_paths else check_path
    summary_stats_path = Path("Z:/target-machine/stale/stats.stdout.json") if stale_summary_paths else stats_path

    return {
        "name": name,
        "ok": True,
        "cmd": cmd,
        "artifacts": {
            "check_perf_thresholds": str(summary_check_path),
            "check_perf_thresholds_failures": 0,
            "worst_bundle": str(step_dir / "bundle.schema2.json"),
        },
        "thresholds_ok": True,
        "stats": {
            "ok": True,
            "stdout": str(summary_stats_path),
            "missing_coverage": [],
            "coverage": validate.stats_coverage_for_doc(stats_doc),
        },
    }


def _write_summary(
    root: Path,
    *,
    with_paint_perf: bool,
    resize_repeat: int = 7,
    torture_overlay_us: int = 0,
    stale_summary_paths: bool = False,
) -> None:
    steps = [
        _step(
            root,
            "resize-jitter",
            with_paint_perf=with_paint_perf,
            repeat=resize_repeat,
            torture_overlay_us=torture_overlay_us,
            stale_summary_paths=stale_summary_paths,
        ),
        _step(
            root,
            "typical-autoscroll",
            with_paint_perf=with_paint_perf,
            repeat=15,
            torture_overlay_us=torture_overlay_us,
            stale_summary_paths=stale_summary_paths,
        ),
        _step(
            root,
            "complex-wheel",
            with_paint_perf=with_paint_perf,
            repeat=7,
            torture_overlay_us=torture_overlay_us,
            stale_summary_paths=stale_summary_paths,
        ),
    ]
    _write_json(
        root / "summary.json",
        {
            "kind": "editor_paint_contract_validate_summary",
            "ok": True,
            "target_profile": validate.TARGET_PROFILE,
            "date_tag": "test-date",
            "with_paint_perf": with_paint_perf,
            "launch_bin": None,
            "launch_cmd": validate._default_launch_cmd(),
            "stats_enabled": True,
            "steps": steps,
        },
    )


def _remove_cmd_item(summary_path: Path, step_name: str, value: str) -> None:
    doc = json.loads(summary_path.read_text(encoding="utf-8"))
    for step in doc["steps"]:
        if step["name"] == step_name:
            step["cmd"] = [item for item in step["cmd"] if item != value]
    summary_path.write_text(json.dumps(doc), encoding="utf-8")


def _remove_summary_field(summary_path: Path, field: str) -> None:
    doc = json.loads(summary_path.read_text(encoding="utf-8"))
    doc.pop(field, None)
    summary_path.write_text(json.dumps(doc), encoding="utf-8")


class EditorPaintContractVerifyArtifactsTests(unittest.TestCase):
    def test_validation_summary_passes_without_code_editor_paint_perf(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            _write_summary(root, with_paint_perf=False)

            report = verify.verify_summary_dir(root, expect_with_paint_perf=False)

        self.assertTrue(report["ok"], report["errors"])
        self.assertEqual("test-date", report["date_tag"])

    def test_attribution_summary_requires_code_editor_paint_perf(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            _write_summary(root, with_paint_perf=False)

            report = verify.verify_summary_dir(root, expect_with_paint_perf=True)

        self.assertFalse(report["ok"])
        self.assertTrue(any("with_paint_perf" in error for error in report["errors"]))
        self.assertTrue(any("code_editor_paint_perf" in error for error in report["errors"]))

    def test_attribution_summary_requires_torture_overlay_zero(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            _write_summary(root, with_paint_perf=True, torture_overlay_us=1)

            report = verify.verify_summary_dir(root, expect_with_paint_perf=True)

        self.assertFalse(report["ok"])
        self.assertTrue(any("code_editor_torture_overlay_zero" in error for error in report["errors"]))

    def test_pair_verification_accepts_validation_and_attribution_dirs(self) -> None:
        with TemporaryDirectory() as td:
            validation = Path(td) / "validation"
            attribution = Path(td) / "attribution"
            _write_summary(validation, with_paint_perf=False)
            _write_summary(attribution, with_paint_perf=True)

            report = verify.verify_artifact_dirs(validation, attribution)

        self.assertTrue(report["ok"], report)

    def test_synced_artifacts_can_remap_stale_target_machine_paths(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            _write_summary(root, with_paint_perf=True, stale_summary_paths=True)

            report = verify.verify_summary_dir(root, expect_with_paint_perf=True)

        self.assertTrue(report["ok"], report["errors"])

    def test_repeat_one_smoke_is_rejected(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            _write_summary(root, with_paint_perf=False, resize_repeat=1)

            report = verify.verify_summary_dir(root, expect_with_paint_perf=False)

        self.assertFalse(report["ok"])
        self.assertTrue(any("--repeat must be 7" in error for error in report["errors"]))

    def test_direct_diag_perf_steps_require_overlay_disabled_env(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            _write_summary(root, with_paint_perf=False)
            _remove_cmd_item(
                root / "summary.json",
                "typical-autoscroll",
                "FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY=0",
            )

            report = verify.verify_summary_dir(root, expect_with_paint_perf=False)

        self.assertFalse(report["ok"])
        self.assertTrue(any("missing required envs" in error for error in report["errors"]))

    def test_validation_pass_rejects_paint_perf_env_on_direct_diag_perf_steps(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            _write_summary(root, with_paint_perf=True)

            report = verify.verify_summary_dir(root, expect_with_paint_perf=False)

        self.assertFalse(report["ok"])
        self.assertTrue(any("must not set FRET_CODE_EDITOR_DIAG_PAINT_PERF=1" in error for error in report["errors"]))

    def test_summary_requires_date_tag_for_traceability(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            _write_summary(root, with_paint_perf=False)
            _remove_summary_field(root / "summary.json", "date_tag")

            report = verify.verify_summary_dir(root, expect_with_paint_perf=False)

        self.assertFalse(report["ok"])
        self.assertTrue(any("date_tag" in error for error in report["errors"]))


if __name__ == "__main__":
    unittest.main()
