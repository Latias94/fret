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
