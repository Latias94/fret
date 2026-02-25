use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PerfHintSeverity {
    Info,
    Warn,
    Error,
}

impl PerfHintSeverity {
    fn parse(s: &str) -> Result<Self, String> {
        match s.trim() {
            "info" => Ok(Self::Info),
            "warn" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            other => Err(format!(
                "invalid perf hint severity: {other} (expected: info|warn|error)"
            )),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PerfHintGateOptions {
    pub(crate) enabled: bool,
    pub(crate) min_severity: PerfHintSeverity,
    pub(crate) deny_codes: BTreeSet<String>,
}

pub(crate) fn parse_perf_hint_gate_options(
    enabled: bool,
    deny_specs: &[String],
    min_severity_spec: Option<&str>,
) -> Result<PerfHintGateOptions, String> {
    let min_severity = match min_severity_spec {
        Some(v) => PerfHintSeverity::parse(v)?,
        None => PerfHintSeverity::Warn,
    };
    let mut deny_codes: BTreeSet<String> = BTreeSet::new();
    for spec in deny_specs {
        for raw in spec.split(',') {
            let code = raw.trim();
            if code.is_empty() {
                continue;
            }
            deny_codes.insert(code.to_string());
        }
    }
    Ok(PerfHintGateOptions {
        enabled,
        min_severity,
        deny_codes,
    })
}

pub(crate) fn perf_hint_gate_failures_for_triage_json(
    script_key: &str,
    bundle_path: &Path,
    run_index: Option<u64>,
    triage: &serde_json::Value,
    opts: &PerfHintGateOptions,
) -> Vec<serde_json::Value> {
    if !opts.enabled {
        return Vec::new();
    }

    let hints = triage
        .get("hints")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if hints.is_empty() {
        return Vec::new();
    }

    let mut failures: Vec<serde_json::Value> = Vec::new();
    for hint in hints {
        let code = hint.get("code").and_then(|v| v.as_str()).unwrap_or("");
        if code.is_empty() {
            continue;
        }
        let sev_str = hint
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("info");
        let sev = PerfHintSeverity::parse(sev_str).unwrap_or(PerfHintSeverity::Info);

        let is_denied = if opts.deny_codes.is_empty() {
            true
        } else {
            opts.deny_codes.contains(code)
        };
        if !is_denied {
            continue;
        }
        if sev < opts.min_severity {
            continue;
        }

        failures.push(serde_json::json!({
            "script": script_key,
            "bundle": bundle_path.display().to_string(),
            "run_index": run_index,
            "code": code,
            "severity": sev.as_str(),
            "hint": hint,
        }));
    }
    failures
}
