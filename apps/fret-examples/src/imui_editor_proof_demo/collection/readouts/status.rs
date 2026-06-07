use super::super::ProofCollectionAsset;

pub(in super::super) fn proof_collection_command_status_line(status: &str) -> String {
    format!("Command status: {status}")
}

pub(in super::super) fn proof_collection_select_all_status(selected_count: usize) -> String {
    format!("Selected all {selected_count} visible asset(s).")
}

pub(in super::super) fn proof_collection_rename_ready_status(label: &str) -> String {
    format!(
        "Rename ready: {label}. The inline editor will focus, Enter commits, and Escape or blur cancels."
    )
}

pub(in super::super) fn proof_collection_rename_commit_status(
    previous: &str,
    next: &str,
) -> String {
    format!("Renamed {previous} -> {next}.")
}

pub(in super::super) fn proof_collection_rename_invalid_status(label: &str) -> String {
    format!("Rename for {label} still needs a non-empty label.")
}

pub(in super::super) fn proof_collection_rename_cancel_status(label: &str) -> String {
    format!("Rename canceled for {label}.")
}

pub(in super::super) fn proof_collection_rename_status_line(status: &str) -> String {
    format!("Rename status: {status}")
}

pub(in super::super) fn proof_collection_duplicate_status(
    duplicated_assets: &[ProofCollectionAsset],
) -> String {
    let labels = duplicated_assets
        .iter()
        .map(|asset| asset.label.as_ref())
        .collect::<Vec<_>>()
        .join(", ");
    format!("Duplicated {} asset(s): {labels}", duplicated_assets.len())
}

pub(in super::super) fn proof_collection_delete_status(
    deleted_assets: &[ProofCollectionAsset],
) -> String {
    let labels = deleted_assets
        .iter()
        .map(|asset| asset.label.as_ref())
        .collect::<Vec<_>>()
        .join(", ");
    format!("Deleted {} asset(s): {labels}", deleted_assets.len())
}
