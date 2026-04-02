#!/usr/bin/env python3
"""
Validate optional workstream state files under docs/workstreams/*/WORKSTREAM.json.
"""

from __future__ import annotations

import json
import sys
from datetime import date
from pathlib import Path


VALID_STATUSES = {"active", "maintenance", "closed", "historical"}
VALID_SCOPE_KINDS = {"execution", "closeout", "audit", "reference"}
VALID_DOC_ROLES = {
    "positioning",
    "design",
    "target",
    "status",
    "next",
    "execution",
    "closeout",
    "reference",
}
VALID_CONTINUE_ACTIONS = {"continue", "start_follow_on", "stay_closed"}


def repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def ensure_repo_relative_path(
    repo: Path, source_file: Path, value: object, field_name: str, errors: list[str]
) -> str | None:
    if not isinstance(value, str) or not value.strip():
        errors.append(f"{source_file}: {field_name} must be a non-empty repo-relative path")
        return None

    raw = value.strip()
    candidate = (repo / raw).resolve()
    try:
        candidate.relative_to(repo)
    except ValueError:
        errors.append(f"{source_file}: {field_name} escapes the repository root: {raw}")
        return None

    if not candidate.exists():
        errors.append(f"{source_file}: {field_name} target does not exist: {raw}")
        return None

    return raw


def ensure_iso_date(value: object, source_file: Path, field_name: str, errors: list[str]) -> None:
    if not isinstance(value, str):
        errors.append(f"{source_file}: {field_name} must be a YYYY-MM-DD string")
        return
    try:
        date.fromisoformat(value)
    except ValueError:
        errors.append(f"{source_file}: {field_name} must be a valid YYYY-MM-DD date")


def ensure_string(value: object, source_file: Path, field_name: str, errors: list[str]) -> str | None:
    if not isinstance(value, str) or not value.strip():
        errors.append(f"{source_file}: {field_name} must be a non-empty string")
        return None
    return value.strip()


def validate_authoritative_docs(
    repo: Path, source_file: Path, value: object, errors: list[str]
) -> set[str]:
    roles: set[str] = set()
    if not isinstance(value, list) or not value:
        errors.append(f"{source_file}: authoritative_docs must be a non-empty list")
        return roles

    for index, item in enumerate(value):
        item_field = f"authoritative_docs[{index}]"
        if not isinstance(item, dict):
            errors.append(f"{source_file}: {item_field} must be an object")
            continue
        role = ensure_string(item.get("role"), source_file, f"{item_field}.role", errors)
        path = ensure_repo_relative_path(repo, source_file, item.get("path"), f"{item_field}.path", errors)
        if role is not None and role not in VALID_DOC_ROLES:
            errors.append(
                f"{source_file}: {item_field}.role must be one of {sorted(VALID_DOC_ROLES)}"
            )
        if role is not None and path is not None:
            roles.add(role)
    return roles


def validate_repro(source_file: Path, value: object, errors: list[str]) -> None:
    if not isinstance(value, dict):
        errors.append(f"{source_file}: repro must be an object")
        return
    ensure_string(value.get("summary"), source_file, "repro.summary", errors)
    commands = value.get("commands")
    if not isinstance(commands, list) or not commands:
        errors.append(f"{source_file}: repro.commands must be a non-empty list")
        return
    for index, command in enumerate(commands):
        ensure_string(command, source_file, f"repro.commands[{index}]", errors)


def validate_gates(source_file: Path, value: object, errors: list[str]) -> None:
    if not isinstance(value, list) or not value:
        errors.append(f"{source_file}: gates must be a non-empty list")
        return
    for index, item in enumerate(value):
        item_field = f"gates[{index}]"
        if not isinstance(item, dict):
            errors.append(f"{source_file}: {item_field} must be an object")
            continue
        ensure_string(item.get("name"), source_file, f"{item_field}.name", errors)
        ensure_string(item.get("command"), source_file, f"{item_field}.command", errors)


def validate_path_list(
    repo: Path, source_file: Path, value: object, field_name: str, errors: list[str], *, allow_empty: bool
) -> None:
    if not isinstance(value, list):
        errors.append(f"{source_file}: {field_name} must be a list")
        return
    if not value and not allow_empty:
        errors.append(f"{source_file}: {field_name} must not be empty")
        return
    for index, item in enumerate(value):
        ensure_repo_relative_path(repo, source_file, item, f"{field_name}[{index}]", errors)


def validate_continue_policy(source_file: Path, value: object, errors: list[str]) -> str | None:
    if not isinstance(value, dict):
        errors.append(f"{source_file}: continue_policy must be an object")
        return None

    action = ensure_string(
        value.get("default_action"), source_file, "continue_policy.default_action", errors
    )
    ensure_string(value.get("notes"), source_file, "continue_policy.notes", errors)
    if action is not None and action not in VALID_CONTINUE_ACTIONS:
        errors.append(
            f"{source_file}: continue_policy.default_action must be one of "
            f"{sorted(VALID_CONTINUE_ACTIONS)}"
        )
        return None
    return action


def validate_workstream_file(repo: Path, path: Path) -> list[str]:
    errors: list[str] = []

    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return [f"{path}: invalid JSON: {exc}"]
    except OSError as exc:
        return [f"{path}: failed to read file: {exc}"]

    if not isinstance(data, dict):
        return [f"{path}: top-level JSON value must be an object"]

    if data.get("schema_version") != 1:
        errors.append(f"{path}: schema_version must be 1")

    slug = ensure_string(data.get("slug"), path, "slug", errors)
    if slug is not None and slug != path.parent.name:
        errors.append(f"{path}: slug must match containing directory '{path.parent.name}'")

    ensure_string(data.get("title"), path, "title", errors)

    status = ensure_string(data.get("status"), path, "status", errors)
    if status is not None and status not in VALID_STATUSES:
        errors.append(f"{path}: status must be one of {sorted(VALID_STATUSES)}")
        status = None

    ensure_iso_date(data.get("updated"), path, "updated", errors)

    scope_kind = ensure_string(data.get("scope_kind"), path, "scope_kind", errors)
    if scope_kind is not None and scope_kind not in VALID_SCOPE_KINDS:
        errors.append(f"{path}: scope_kind must be one of {sorted(VALID_SCOPE_KINDS)}")

    ensure_string(data.get("problem"), path, "problem", errors)
    roles = validate_authoritative_docs(repo, path, data.get("authoritative_docs"), errors)
    validate_repro(path, data.get("repro"), errors)
    validate_gates(path, data.get("gates"), errors)
    validate_path_list(repo, path, data.get("evidence"), "evidence", errors, allow_empty=False)
    validate_path_list(repo, path, data.get("adr_refs"), "adr_refs", errors, allow_empty=True)
    continue_action = validate_continue_policy(path, data.get("continue_policy"), errors)

    follow_on_of = data.get("follow_on_of")
    if follow_on_of is not None and (not isinstance(follow_on_of, str) or not follow_on_of.strip()):
        errors.append(f"{path}: follow_on_of must be a non-empty string when present")

    if not ({"positioning", "design"} & roles):
        errors.append(
            f"{path}: authoritative_docs must include at least one 'positioning' or 'design' role"
        )
    if "execution" not in roles:
        errors.append(f"{path}: authoritative_docs must include at least one 'execution' role")
    if status in {"closed", "historical"} and "closeout" not in roles:
        errors.append(
            f"{path}: closed or historical lanes must include a 'closeout' authoritative doc"
        )

    if status in {"active", "maintenance"} and continue_action not in {"continue"}:
        errors.append(
            f"{path}: active or maintenance lanes must use continue_policy.default_action='continue'"
        )
    if status in {"closed", "historical"} and continue_action not in {"start_follow_on", "stay_closed"}:
        errors.append(
            f"{path}: closed or historical lanes must use continue_policy.default_action "
            f"'start_follow_on' or 'stay_closed'"
        )

    return errors


def main() -> int:
    repo = repo_root()
    root = repo / "docs" / "workstreams"
    files = sorted(root.rglob("WORKSTREAM.json"))

    if not files:
        print("No WORKSTREAM.json files found. Skipping.")
        return 0

    errors: list[str] = []
    for path in files:
        errors.extend(validate_workstream_file(repo, path))

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1

    print(f"Validated {len(files)} WORKSTREAM.json file(s).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
