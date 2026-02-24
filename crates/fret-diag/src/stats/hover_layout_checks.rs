use super::BundleStatsReport;

pub(crate) fn check_report_for_hover_layout_invalidations(
    report: &BundleStatsReport,
    max_allowed: u32,
) -> Result<(), String> {
    if report.max_hover_layout_invalidations <= max_allowed {
        return Ok(());
    }

    let mut extra = String::new();
    if let Some(worst) = report.worst_hover_layout.as_ref() {
        extra.push_str(&format!(
            " worst(window={} tick={} frame={} hover_layout={})",
            worst.window,
            worst.tick_id,
            worst.frame_id,
            worst.hover_declarative_layout_invalidations
        ));
        if !worst.hotspots.is_empty() {
            let items: Vec<String> = worst
                .hotspots
                .iter()
                .take(3)
                .map(|h| {
                    let mut s = format!(
                        "layout={} hit={} paint={} node={}",
                        h.layout, h.hit_test, h.paint, h.node
                    );
                    if let Some(test_id) = h.test_id.as_deref()
                        && !test_id.is_empty()
                    {
                        s.push_str(&format!(" test_id={test_id}"));
                    }
                    if let Some(role) = h.role.as_deref()
                        && !role.is_empty()
                    {
                        s.push_str(&format!(" role={role}"));
                    }
                    s
                })
                .collect();
            extra.push_str(&format!(" hotspots=[{}]", items.join(" | ")));
        }
    }

    Err(format!(
        "hover-attributed declarative layout invalidations detected (max_per_frame={} allowed={max_allowed}).{}",
        report.max_hover_layout_invalidations, extra
    ))
}
