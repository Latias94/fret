use std::path::Path;
use std::process::Command;
use std::process::Stdio;

use crate::util::now_unix_ms;

fn parse_vmmap_size_token_to_bytes(token: &str) -> Option<u64> {
    let t = token.trim();
    if t.is_empty() {
        return None;
    }
    if t.eq_ignore_ascii_case("N/A") {
        return None;
    }

    let (num, suffix) = t
        .chars()
        .position(|c| c.is_ascii_alphabetic())
        .map(|idx| (&t[..idx], &t[idx..]))
        .unwrap_or((t, ""));

    let n = num.trim().parse::<f64>().ok()?;
    let mult = match suffix.trim().to_ascii_uppercase().as_str() {
        "" | "B" => 1.0,
        "K" | "KB" => 1024.0,
        "M" | "MB" => 1024.0 * 1024.0,
        "G" | "GB" => 1024.0 * 1024.0 * 1024.0,
        "T" | "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };

    Some(((n * mult).round() as u128).min(u64::MAX as u128) as u64)
}

fn parse_vmmap_u64_token(token: &str) -> Option<u64> {
    let t = token.trim().trim_end_matches(',');
    if t.is_empty() {
        return None;
    }
    t.replace(',', "").parse::<u64>().ok()
}

fn parse_vmmap_percent_token(token: &str) -> Option<f64> {
    let t = token.trim().trim_end_matches('%');
    if t.is_empty() {
        return None;
    }
    t.parse::<f64>().ok()
}

#[derive(Debug, Clone, serde::Serialize)]
struct VmmapRegionRow {
    region_type: String,
    virtual_bytes: u64,
    resident_bytes: u64,
    dirty_bytes: u64,
    swapped_bytes: u64,
    volatile_bytes: u64,
    nonvol_bytes: u64,
    empty_bytes: u64,
    region_count: u64,
}

fn parse_vmmap_regions_table(stdout: &str) -> Vec<VmmapRegionRow> {
    let mut rows = Vec::new();
    let mut in_table = false;

    for line in stdout.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }

        if l.starts_with("REGION TYPE") {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }

        if l.starts_with("TOTAL") || l.starts_with("TOTAL,") {
            break;
        }
        if l.starts_with("==========") || l.starts_with("===========") {
            continue;
        }

        let tokens: Vec<&str> = l.split_whitespace().collect();
        let first_numeric = tokens
            .iter()
            .position(|t| t.chars().next().is_some_and(|c| c.is_ascii_digit()));
        let Some(first_numeric) = first_numeric else {
            continue;
        };

        let region_type = tokens[..first_numeric].join(" ");
        if tokens.len() < first_numeric.saturating_add(8) {
            continue;
        }

        let parse_size = |idx: usize| parse_vmmap_size_token_to_bytes(tokens[idx]).unwrap_or(0);

        let virtual_bytes = parse_size(first_numeric);
        let resident_bytes = parse_size(first_numeric + 1);
        let dirty_bytes = parse_size(first_numeric + 2);
        let swapped_bytes = parse_size(first_numeric + 3);
        let volatile_bytes = parse_size(first_numeric + 4);
        let nonvol_bytes = parse_size(first_numeric + 5);
        let empty_bytes = parse_size(first_numeric + 6);
        let region_count = parse_vmmap_u64_token(tokens[first_numeric + 7]).unwrap_or(0);

        rows.push(VmmapRegionRow {
            region_type,
            virtual_bytes,
            resident_bytes,
            dirty_bytes,
            swapped_bytes,
            volatile_bytes,
            nonvol_bytes,
            empty_bytes,
            region_count,
        });
    }

    rows
}

#[derive(Debug, Clone, serde::Serialize)]
struct VmmapMallocZoneRow {
    zone: String,
    virtual_bytes: u64,
    resident_bytes: u64,
    dirty_bytes: u64,
    swapped_bytes: u64,
    allocation_count: u64,
    allocated_bytes: u64,
    frag_bytes: u64,
    frag_percent: Option<f64>,
    region_count: u64,
}

fn parse_vmmap_malloc_zone_table(stdout: &str) -> Vec<VmmapMallocZoneRow> {
    let mut rows = Vec::new();
    let mut in_table = false;

    for line in stdout.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }

        if l.starts_with("MALLOC ZONE") {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }

        if l.starts_with("TOTAL") || l.starts_with("TOTAL,") {
            break;
        }
        if l.starts_with("==========") || l.starts_with("===========") {
            continue;
        }

        let tokens: Vec<&str> = l.split_whitespace().collect();
        let first_numeric = tokens
            .iter()
            .position(|t| t.chars().next().is_some_and(|c| c.is_ascii_digit()));
        let Some(first_numeric) = first_numeric else {
            continue;
        };

        let zone = tokens[..first_numeric].join(" ");
        if tokens.len() < first_numeric.saturating_add(9) {
            continue;
        }

        let parse_size = |idx: usize| parse_vmmap_size_token_to_bytes(tokens[idx]).unwrap_or(0);

        let virtual_bytes = parse_size(first_numeric);
        let resident_bytes = parse_size(first_numeric + 1);
        let dirty_bytes = parse_size(first_numeric + 2);
        let swapped_bytes = parse_size(first_numeric + 3);
        let allocation_count = parse_vmmap_u64_token(tokens[first_numeric + 4]).unwrap_or(0);
        let allocated_bytes = parse_size(first_numeric + 5);
        let frag_bytes = parse_size(first_numeric + 6);
        let frag_percent = parse_vmmap_percent_token(tokens[first_numeric + 7]);
        let region_count = parse_vmmap_u64_token(tokens[first_numeric + 8]).unwrap_or(0);

        rows.push(VmmapMallocZoneRow {
            zone,
            virtual_bytes,
            resident_bytes,
            dirty_bytes,
            swapped_bytes,
            allocation_count,
            allocated_bytes,
            frag_bytes,
            frag_percent,
            region_count,
        });
    }

    rows
}

pub(crate) fn collect_macos_vmmap_summary_best_effort(
    pid: u32,
    out_dir: &Path,
    vmmap_summary_file: &str,
) -> Option<serde_json::Value> {
    if vmmap_summary_file.trim().is_empty() {
        return None;
    }

    let out = Command::new("/usr/bin/vmmap")
        .args(["-summary", &pid.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let vmmap_file = out_dir.join(vmmap_summary_file);
    let _ = std::fs::write(&vmmap_file, &stdout);

    let mut physical_footprint_bytes: Option<u64> = None;
    let mut physical_footprint_peak_bytes: Option<u64> = None;

    let regions_table = parse_vmmap_regions_table(&stdout);
    let malloc_zone_table = parse_vmmap_malloc_zone_table(&stdout);

    for line in stdout.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix("Physical footprint:") {
            let token = rest.trim().split_whitespace().next().unwrap_or("");
            physical_footprint_bytes = parse_vmmap_size_token_to_bytes(token);
            continue;
        }
        if let Some(rest) = l.strip_prefix("Physical footprint (peak):") {
            let token = rest.trim().split_whitespace().next().unwrap_or("");
            physical_footprint_peak_bytes = parse_vmmap_size_token_to_bytes(token);
            continue;
        }
    }

    let region_dirty_bytes = |name: &str| -> Option<u64> {
        regions_table
            .iter()
            .find(|r| r.region_type.eq_ignore_ascii_case(name))
            .map(|r| r.dirty_bytes)
    };

    let region_dirty_bytes_sum_prefix = |prefix: &str| -> Option<u64> {
        let mut sum: u64 = 0;
        let mut any = false;
        for row in &regions_table {
            if row.region_type.starts_with(prefix) {
                any = true;
                sum = sum.saturating_add(row.dirty_bytes);
            }
        }
        any.then_some(sum)
    };

    let owned_unmapped_memory_dirty_bytes = region_dirty_bytes("owned unmapped memory");
    let io_surface_dirty_bytes =
        region_dirty_bytes_sum_prefix("IOSurface").or_else(|| region_dirty_bytes("IOSurface"));
    let io_accelerator_dirty_bytes = region_dirty_bytes_sum_prefix("IOAccelerator")
        .or_else(|| region_dirty_bytes("IOAccelerator"));
    let malloc_small_dirty_bytes = region_dirty_bytes_sum_prefix("MALLOC_SMALL")
        .or_else(|| region_dirty_bytes("MALLOC_SMALL"));
    let malloc_dirty_bytes_total =
        region_dirty_bytes_sum_prefix("MALLOC_").or_else(|| region_dirty_bytes("MALLOC"));

    let mut regions_top_dirty = regions_table.clone();
    regions_top_dirty.sort_by_key(|r| std::cmp::Reverse(r.dirty_bytes));
    regions_top_dirty.truncate(12);

    let mut regions_top_resident = regions_table.clone();
    regions_top_resident.sort_by_key(|r| std::cmp::Reverse(r.resident_bytes));
    regions_top_resident.truncate(12);

    let mut malloc_top_allocated = malloc_zone_table.clone();
    malloc_top_allocated.sort_by_key(|r| std::cmp::Reverse(r.allocated_bytes));
    malloc_top_allocated.truncate(12);

    let mut malloc_top_frag = malloc_zone_table.clone();
    malloc_top_frag.sort_by_key(|r| std::cmp::Reverse(r.frag_bytes));
    malloc_top_frag.truncate(12);

    let default_malloc_zone = malloc_zone_table
        .iter()
        .find(|r| r.zone.to_ascii_lowercase().contains("defaultmalloczone"))
        .cloned();

    let mut malloc_total_allocated_bytes: u64 = 0;
    let mut malloc_total_frag_bytes: u64 = 0;
    let mut malloc_total_dirty_bytes: u64 = 0;
    let mut malloc_total_allocation_count: u64 = 0;
    let mut malloc_total_region_count: u64 = 0;
    for row in &malloc_zone_table {
        malloc_total_allocated_bytes =
            malloc_total_allocated_bytes.saturating_add(row.allocated_bytes);
        malloc_total_frag_bytes = malloc_total_frag_bytes.saturating_add(row.frag_bytes);
        malloc_total_dirty_bytes = malloc_total_dirty_bytes.saturating_add(row.dirty_bytes);
        malloc_total_allocation_count =
            malloc_total_allocation_count.saturating_add(row.allocation_count);
        malloc_total_region_count = malloc_total_region_count.saturating_add(row.region_count);
    }

    Some(serde_json::json!({
        "collector": "vmmap -summary",
        "captured_unix_ms": now_unix_ms(),
        "vmmap_summary_file": vmmap_summary_file,
        "physical_footprint_bytes": physical_footprint_bytes,
        "physical_footprint_peak_bytes": physical_footprint_peak_bytes,
        "regions": {
            "owned_unmapped_memory_dirty_bytes": owned_unmapped_memory_dirty_bytes,
            "io_surface_dirty_bytes": io_surface_dirty_bytes,
            "io_accelerator_dirty_bytes": io_accelerator_dirty_bytes,
            "malloc_small_dirty_bytes": malloc_small_dirty_bytes,
            "malloc_dirty_bytes_total": malloc_dirty_bytes_total,
        },
        "tables": {
            "regions": {
                "rows_total": regions_table.len(),
                "top_dirty": regions_top_dirty,
                "top_resident": regions_top_resident,
            },
            "malloc_zones": {
                "rows_total": malloc_zone_table.len(),
                "default_zone": default_malloc_zone,
                "total": {
                    "allocated_bytes": malloc_total_allocated_bytes,
                    "frag_bytes": malloc_total_frag_bytes,
                    "dirty_bytes": malloc_total_dirty_bytes,
                    "allocation_count": malloc_total_allocation_count,
                    "region_count": malloc_total_region_count,
                },
                "top_allocated": malloc_top_allocated,
                "top_frag": malloc_top_frag,
            },
        },
        "note": "best-effort; vmmap output is captured at a tool-selected point in time.",
    }))
}
