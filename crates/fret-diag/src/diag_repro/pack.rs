use super::super::*;

pub(super) struct PackOutcome {
    pub(super) packed_zip: Option<PathBuf>,
    pub(super) overall_error: Option<String>,
    pub(super) overall_reason_code: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn pack_repro_zip(
    multi_pack: bool,
    pack_items: &[ReproPackItem],
    selected_bundle_path: Option<&PathBuf>,
    resolved_out_dir: &Path,
    summary_path: &Path,
    zip_out: &Path,
    pack_defaults: (bool, bool, bool),
    pack_schema2_only: bool,
    ensure_ai_packet: bool,
    pack_ai_only: bool,
    with_renderdoc: bool,
    with_tracy: bool,
    stats_top: usize,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> Result<PackOutcome, String> {
    if multi_pack {
        let bundles: Vec<ReproZipBundle> = pack_items
            .iter()
            .enumerate()
            .map(|(idx, item)| ReproZipBundle {
                prefix: repro_zip_prefix_for_script(item, idx),
                bundle_artifact: item.bundle_artifact.clone(),
                source_script: item.script_path.clone(),
            })
            .collect();

        if pack_ai_only {
            if ensure_ai_packet {
                for item in &bundles {
                    let bundle_dir = resolve_bundle_root_dir(&item.bundle_artifact)?;
                    let packet_dir = bundle_dir.join("ai.packet");
                    if packet_dir.is_dir() {
                        continue;
                    }
                    if let Err(err) = crate::commands::ai_packet::generate_ai_packet_dir(
                        &item.bundle_artifact,
                        &bundle_dir,
                        &packet_dir,
                        pack_defaults.1,
                        stats_top,
                        Some(sort),
                        warmup_frames,
                        None,
                    ) {
                        return Ok(PackOutcome {
                            packed_zip: None,
                            overall_error: Some(format!("failed to generate ai.packet: {err}")),
                            overall_reason_code: Some("tooling.ai_packet.failed".to_string()),
                        });
                    }
                }
            }

            if let Err(err) =
                pack_repro_ai_zip_multi(zip_out, resolved_out_dir, summary_path, &bundles)
            {
                return Ok(PackOutcome {
                    packed_zip: None,
                    overall_error: Some(format!("failed to pack repro ai-only zip: {err}")),
                    overall_reason_code: Some("tooling.pack.failed".to_string()),
                });
            }

            return Ok(PackOutcome {
                packed_zip: Some(zip_out.to_path_buf()),
                overall_error: None,
                overall_reason_code: None,
            });
        }

        if let Err(err) = pack_repro_zip_multi(
            zip_out,
            pack_defaults.0,
            pack_defaults.1,
            pack_defaults.2,
            pack_schema2_only,
            with_renderdoc,
            with_tracy,
            resolved_out_dir,
            summary_path,
            &bundles,
            stats_top,
            sort,
            warmup_frames,
        ) {
            return Ok(PackOutcome {
                packed_zip: None,
                overall_error: Some(format!("failed to pack repro zip: {err}")),
                overall_reason_code: None,
            });
        }

        return Ok(PackOutcome {
            packed_zip: Some(zip_out.to_path_buf()),
            overall_error: None,
            overall_reason_code: None,
        });
    }

    let Some(bundle_path) = selected_bundle_path else {
        return Ok(PackOutcome {
            packed_zip: None,
            overall_error: Some(
                "no bundle artifact found (add `capture_bundle` or enable script auto-dumps)"
                    .to_string(),
            ),
            overall_reason_code: Some("tooling.bundle_missing".to_string()),
        });
    };

    let bundle_dir = resolve_bundle_root_dir(bundle_path)?;
    let artifacts_root = if bundle_dir.starts_with(resolved_out_dir) {
        resolved_out_dir.to_path_buf()
    } else {
        bundle_dir
            .parent()
            .unwrap_or(resolved_out_dir)
            .to_path_buf()
    };

    if pack_ai_only {
        if ensure_ai_packet {
            let packet_dir = bundle_dir.join("ai.packet");
            if !packet_dir.is_dir() {
                if let Err(err) = crate::commands::ai_packet::generate_ai_packet_dir(
                    bundle_path,
                    &bundle_dir,
                    &packet_dir,
                    pack_defaults.1,
                    stats_top,
                    Some(sort),
                    warmup_frames,
                    None,
                ) {
                    return Ok(PackOutcome {
                        packed_zip: None,
                        overall_error: Some(format!("failed to generate ai.packet: {err}")),
                        overall_reason_code: Some("tooling.ai_packet.failed".to_string()),
                    });
                }
            }
        }

        if let Err(err) = pack_ai_packet_dir_to_zip(&bundle_dir, zip_out, &artifacts_root) {
            return Ok(PackOutcome {
                packed_zip: None,
                overall_error: Some(format!("failed to pack repro ai-only zip: {err}")),
                overall_reason_code: Some("tooling.pack.failed".to_string()),
            });
        }

        return Ok(PackOutcome {
            packed_zip: Some(zip_out.to_path_buf()),
            overall_error: None,
            overall_reason_code: None,
        });
    }

    if let Err(err) = pack_bundle_dir_to_zip(
        &bundle_dir,
        zip_out,
        pack_defaults.0,
        pack_defaults.1,
        pack_defaults.2,
        pack_schema2_only,
        with_renderdoc,
        with_tracy,
        &artifacts_root,
        stats_top,
        sort,
        warmup_frames,
    ) {
        return Ok(PackOutcome {
            packed_zip: None,
            overall_error: Some(format!("failed to pack repro zip: {err}")),
            overall_reason_code: Some("tooling.pack.failed".to_string()),
        });
    }

    Ok(PackOutcome {
        packed_zip: Some(zip_out.to_path_buf()),
        overall_error: None,
        overall_reason_code: None,
    })
}
