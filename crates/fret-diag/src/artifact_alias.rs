use std::path::Path;

pub(crate) fn bundle_artifact_alias_pair(bundle_path: &Path) -> (String, String) {
    let bundle_artifact = bundle_path.display().to_string();
    let bundle_json = bundle_artifact.clone();
    (bundle_artifact, bundle_json)
}

#[cfg(test)]
mod tests {
    use super::bundle_artifact_alias_pair;
    use std::path::Path;

    #[test]
    fn bundle_artifact_alias_pair_dual_writes_legacy_bundle_json_alias() {
        let (bundle_artifact, bundle_json) =
            bundle_artifact_alias_pair(Path::new("target/fret-diag/bundle.schema2.json"));

        assert_eq!(bundle_artifact, "target/fret-diag/bundle.schema2.json");
        assert_eq!(bundle_json, bundle_artifact);
    }
}
