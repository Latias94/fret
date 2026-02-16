use super::*;

#[test]
fn capability_key_kind_matches_platform_capabilities_accessors() {
    let caps = PlatformCapabilities::default();

    for &key in KNOWN_BOOL_CAPABILITY_KEYS {
        assert!(caps.bool_key(key).is_some(), "bool_key must accept {key}");
        assert_eq!(capability_key_kind(key), Some(CapabilityValueKind::Bool));
    }

    for &key in KNOWN_STR_CAPABILITY_KEYS {
        assert!(caps.str_key(key).is_some(), "str_key must accept {key}");
        assert_eq!(capability_key_kind(key), Some(CapabilityValueKind::Str));
    }

    assert_eq!(capability_key_kind("does.not.exist"), None);
    assert_eq!(caps.bool_key("does.not.exist"), None);
    assert_eq!(caps.str_key("does.not.exist"), None);
}

#[test]
fn clipboard_text_capabilities_deserialize_legacy_and_struct_shapes() {
    let legacy: PlatformCapabilities =
        serde_json::from_str(r#"{ "clipboard": { "text": true } }"#).expect("deserialize legacy");
    assert!(legacy.clipboard.text.read);
    assert!(legacy.clipboard.text.write);

    let split: PlatformCapabilities =
        serde_json::from_str(r#"{ "clipboard": { "text": { "read": true, "write": false } } }"#)
            .expect("deserialize split");
    assert!(split.clipboard.text.read);
    assert!(!split.clipboard.text.write);
}
