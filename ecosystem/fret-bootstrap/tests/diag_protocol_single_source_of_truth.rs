use std::fs;
use std::path::{Path, PathBuf};

fn visit_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn runner_does_not_fork_diag_script_protocol_types() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");

    let mut files = Vec::new();
    visit_rs_files(&src_dir, &mut files);
    files.sort();

    let disallowed_definition_markers: &[&str] = &[
        "struct UiActionScriptV1",
        "struct UiActionScriptV2",
        "enum UiActionStepV1",
        "enum UiActionStepV2",
        "struct UiSelectorV1",
        "enum UiSelectorV1",
        "struct UiPredicateV1",
        "enum UiPredicateV1",
        "struct UiScriptResultV1",
        "enum UiScriptStageV1",
    ];

    let mut offenders = Vec::new();
    for path in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };

        let mut in_legacy_module = false;
        let mut legacy_brace_balance: i32 = 0;
        let mut previous_line: Option<&str> = None;

        for (line_index, line) in text.lines().enumerate() {
            if !in_legacy_module
                && line.contains("mod legacy_forked_script_protocol")
                && previous_line.map(str::trim) != Some("#[cfg(any())]")
            {
                offenders.push(format!(
                    "{}:{}: legacy fork module must remain `#[cfg(any())]`",
                    path.display(),
                    line_index + 1
                ));
            }

            if !in_legacy_module
                && previous_line.map(str::trim) == Some("#[cfg(any())]")
                && line.contains("mod legacy_forked_script_protocol")
                && line.contains('{')
            {
                in_legacy_module = true;
                legacy_brace_balance =
                    line.matches('{').count() as i32 - line.matches('}').count() as i32;
                previous_line = Some(line);
                continue;
            }

            if in_legacy_module {
                legacy_brace_balance += line.matches('{').count() as i32;
                legacy_brace_balance -= line.matches('}').count() as i32;
                if legacy_brace_balance <= 0 {
                    in_legacy_module = false;
                }
                previous_line = Some(line);
                continue;
            }

            for marker in disallowed_definition_markers {
                if line.contains(marker) {
                    offenders.push(format!(
                        "{}:{}: forbidden local protocol definition marker `{}`",
                        path.display(),
                        line_index + 1,
                        marker
                    ));
                }
            }

            previous_line = Some(line);
        }
    }

    assert!(
        offenders.is_empty(),
        "Runner crate must not fork diagnostics script protocol types; use `crates/fret-diag-protocol`.\n\n{}",
        offenders.join("\n")
    );
}
