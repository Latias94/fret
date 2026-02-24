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
                    if let Err(err) = crate::commands::ai_packet::ensure_ai_packet_dir_best_effort(
                        Some(&item.bundle_artifact),
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
            let res = crate::commands::ai_packet::ensure_ai_packet_dir_best_effort(
                Some(bundle_path),
                &bundle_dir,
                &packet_dir,
                pack_defaults.1,
                stats_top,
                Some(sort),
                warmup_frames,
                None,
            );
            if let Err(err) = res {
                return Ok(PackOutcome {
                    packed_zip: None,
                    overall_error: Some(format!("failed to generate ai.packet: {err}")),
                    overall_reason_code: Some("tooling.ai_packet.failed".to_string()),
                });
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

#[cfg(test)]
mod tests {
    use super::*;

    fn write_sidecars_only_bundle_dir(bundle_dir: &Path) {
        std::fs::create_dir_all(bundle_dir).expect("create bundle dir");
        std::fs::write(
            bundle_dir.join("bundle.meta.json"),
            serde_json::to_vec(&serde_json::json!({
                "kind": "bundle_meta",
                "schema_version": 1,
                "warmup_frames": 0,
                "bundle": "bundle.json",
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            bundle_dir.join("test_ids.index.json"),
            b"{\"kind\":\"test_ids_index\",\"schema_version\":1}",
        )
        .unwrap();
        std::fs::write(
            bundle_dir.join("bundle.index.json"),
            serde_json::to_vec(&serde_json::json!({
                "kind": "bundle_index",
                "schema_version": 1,
                "warmup_frames": 0,
                "bundle": "bundle.json",
                "windows": [],
                "script": { "steps": [] },
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            bundle_dir.join("frames.index.json"),
            serde_json::to_vec(&serde_json::json!({
                "kind": "frames_index",
                "schema_version": 1,
                "bundle": "bundle.json",
                "generated_unix_ms": 0,
                "warmup_frames": 0,
                "has_semantics_table": true,
                "columns": ["frame_id", "window_snapshot_seq", "timestamp_unix_ms", "total_time_us", "layout_time_us", "paint_time_us", "semantics_fingerprint", "semantics_source_tag"],
                "windows_total": 0,
                "snapshots_total": 0,
                "frames_total": 0,
                "windows": []
            }))
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn repro_ai_only_can_generate_ai_packet_from_sidecars_only() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-repro-pack-ai-only-sidecars-{}-{}",
            crate::util::now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let resolved_out_dir = root.join("out");
        std::fs::create_dir_all(&resolved_out_dir).expect("create out dir");

        let bundle_a_dir = resolved_out_dir.join("bundle-a");
        let bundle_b_dir = resolved_out_dir.join("bundle-b");
        write_sidecars_only_bundle_dir(&bundle_a_dir);
        write_sidecars_only_bundle_dir(&bundle_b_dir);

        let script_a = root.join("a.script.json");
        let script_b = root.join("b.script.json");
        std::fs::write(&script_a, "{\"schema_version\":1,\"steps\":[]}").expect("write script a");
        std::fs::write(&script_b, "{\"schema_version\":1,\"steps\":[]}").expect("write script b");

        let pack_items = vec![
            crate::ReproPackItem {
                script_path: script_a.clone(),
                bundle_artifact: bundle_a_dir.join("bundle.schema2.json"),
            },
            crate::ReproPackItem {
                script_path: script_b.clone(),
                bundle_artifact: bundle_b_dir.join("bundle.schema2.json"),
            },
        ];

        let summary_path = resolved_out_dir.join("repro.summary.json");
        std::fs::write(&summary_path, "{\"schema_version\":1}").expect("write summary");

        let zip_out = root.join("repro.ai.zip");
        let out = pack_repro_zip(
            true,
            &pack_items,
            None,
            &resolved_out_dir,
            &summary_path,
            &zip_out,
            (false, false, false),
            false,
            true,
            true,
            false,
            false,
            5,
            BundleStatsSort::Invalidation,
            0,
        )
        .expect("pack repro ai-only");

        assert!(out.packed_zip.is_some());
        assert!(zip_out.is_file());

        let f = std::fs::File::open(&zip_out).expect("open out zip");
        let mut zip = zip::ZipArchive::new(f).expect("open zip archive");
        let names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).expect("zip entry").name().to_string())
            .collect();

        let prefixes = [
            crate::pack_zip::repro_zip_prefix_for_script(&pack_items[0], 0),
            crate::pack_zip::repro_zip_prefix_for_script(&pack_items[1], 1),
        ];
        for prefix in prefixes {
            assert!(
                names
                    .iter()
                    .any(|n| n == &format!("{prefix}/_root/ai.packet/bundle.meta.json")),
                "{prefix} expected ai.packet/bundle.meta.json under _root"
            );
        }

        let _ = std::fs::remove_dir_all(&root);
    }
}
