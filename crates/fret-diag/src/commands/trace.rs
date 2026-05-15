use std::path::{Path, PathBuf};

use serde_json::{Value, json};

pub(crate) fn cmd_trace(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    trace_out: Option<PathBuf>,
    trace_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }

    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle artifact path (try: fretboard-dev diag trace <base_or_session_out_dir|bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let resolved = crate::commands::resolve::resolve_bundle_ref(&src)?;
    let bundle_path = resolved.bundle_artifact;
    let bundle_dir = resolved.bundle_dir;
    let out = trace_out
        .map(|path| crate::resolve_path(workspace_root, path))
        .unwrap_or_else(|| bundle_dir.join("trace.chrome.json"));
    let trace = crate::trace::chrome_trace_json_from_bundle_path(&bundle_path)?;
    crate::trace::write_chrome_trace_value(&out, &trace)?;
    if trace_json {
        let report = trace_command_report_json(&bundle_path, &bundle_dir, &out, &trace);
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
        );
    } else {
        println!("{}", out.display());
    }
    Ok(())
}

fn trace_command_report_json(
    bundle_path: &Path,
    bundle_dir: &Path,
    trace_path: &Path,
    trace: &Value,
) -> Value {
    json!({
        "kind": crate::perf_schema::PERF_TRACE_REPORT_KIND,
        "schema_version": crate::perf_schema::PERF_TRACE_REPORT_SCHEMA_VERSION,
        "schema_policy": crate::perf_schema::schema_policy_json(),
        "bundle_artifact": bundle_path.display().to_string(),
        "bundle_dir": bundle_dir.display().to_string(),
        "trace_chrome_json_path": trace_path.display().to_string(),
        "trace_kind": trace.get("kind").cloned().unwrap_or(Value::Null),
        "trace_schema_version": trace.get("schema_version").cloned().unwrap_or(Value::Null),
        "trace_source": trace.get("trace_source").cloned().unwrap_or(Value::Null),
        "real_spans_included": trace.get("real_spans_included").cloned().unwrap_or(Value::Null),
        "real_span_event_count": trace.get("real_span_event_count").cloned().unwrap_or(Value::Null),
        "real_span_extension_keys": trace.get("real_span_extension_keys").cloned().unwrap_or(Value::Null),
        "trace_event_count": trace
            .get("traceEvents")
            .and_then(|events| events.as_array())
            .map(|events| events.len())
            .unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_command_report_json_projects_real_span_metadata() {
        let trace = json!({
            "kind": crate::perf_schema::PERF_TRACE_CHROME_KIND,
            "schema_version": crate::perf_schema::PERF_TRACE_SCHEMA_VERSION,
            "trace_source": crate::perf_schema::PERF_TRACE_SOURCE_BUNDLE_SYNTHETIC_PHASES_WITH_EXTENSION_SPANS,
            "real_spans_included": true,
            "real_span_event_count": 2,
            "real_span_extension_keys": ["fret.perf.spans.v1"],
            "traceEvents": [{ "name": "fret.ui.view" }, { "name": "fret.ui.paint" }]
        });

        let report = trace_command_report_json(
            Path::new("target/fret-diag/demo/bundle.json"),
            Path::new("target/fret-diag/demo"),
            Path::new("target/fret-diag/demo/trace.chrome.json"),
            &trace,
        );

        assert_eq!(
            report.get("kind").and_then(|v| v.as_str()),
            Some(crate::perf_schema::PERF_TRACE_REPORT_KIND)
        );
        assert_eq!(
            report.get("schema_version").and_then(|v| v.as_u64()),
            Some(crate::perf_schema::PERF_TRACE_REPORT_SCHEMA_VERSION as u64)
        );
        assert_eq!(
            report
                .get("schema_policy")
                .and_then(|v| v.get("compatibility"))
                .and_then(|v| v.as_str()),
            Some("additive_only")
        );
        assert_eq!(
            report.get("trace_source").and_then(|v| v.as_str()),
            Some(
                crate::perf_schema::PERF_TRACE_SOURCE_BUNDLE_SYNTHETIC_PHASES_WITH_EXTENSION_SPANS
            )
        );
        assert_eq!(
            report.get("real_spans_included").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            report.get("real_span_event_count").and_then(|v| v.as_u64()),
            Some(2)
        );
        assert_eq!(
            report.get("trace_event_count").and_then(|v| v.as_u64()),
            Some(2)
        );
        assert_eq!(
            report
                .get("real_span_extension_keys")
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
                .and_then(|v| v.as_str()),
            Some("fret.perf.spans.v1")
        );
    }
}
