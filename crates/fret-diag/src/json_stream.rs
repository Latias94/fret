use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub(crate) type BundleJsonDeserializer =
    serde_json::Deserializer<serde_json::de::IoRead<BufReader<File>>>;

pub(crate) fn with_bundle_json_deserializer(
    bundle_path: &Path,
    f: impl FnOnce(&mut BundleJsonDeserializer) -> Result<(), serde_json::Error>,
) -> Result<(), String> {
    let file = File::open(bundle_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);

    f(&mut de).map_err(|e| e.to_string())
}

pub(crate) fn with_bundle_json_deserializer_allow_stop(
    bundle_path: &Path,
    stop_marker: &'static str,
    f: impl FnOnce(&mut BundleJsonDeserializer) -> Result<(), serde_json::Error>,
) -> Result<(), String> {
    let file = File::open(bundle_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);

    match f(&mut de) {
        Ok(()) => Ok(()),
        Err(err) => {
            let msg = err.to_string();
            if msg.starts_with(stop_marker) {
                Ok(())
            } else {
                Err(msg)
            }
        }
    }
}
