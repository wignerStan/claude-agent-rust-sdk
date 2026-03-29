use claude_agent_types::security::*;

#[test]
fn api_key_new_and_expose() {
    let key = ApiKey::new("sk-test-12345");
    assert_eq!(key.expose(), "sk-test-12345");
}

#[test]
fn api_key_default_is_empty() {
    let key = ApiKey::default();
    assert!(key.is_empty());
    assert_eq!(key.expose(), "");
}

#[test]
fn api_key_is_empty_true() {
    assert!(ApiKey::new("").is_empty());
}

#[test]
fn api_key_is_empty_false() {
    assert!(!ApiKey::new("not-empty").is_empty());
}

#[test]
fn api_key_display_redacted() {
    assert_eq!(format!("{}", ApiKey::new("sk-secret")), "[REDACTED]");
}

#[test]
fn api_key_debug_redacted() {
    assert_eq!(format!("{:?}", ApiKey::new("sk-secret")), "ApiKey([REDACTED])");
}

#[test]
fn api_key_serialize_redacted() {
    let json = serde_json::to_string(&ApiKey::new("sk-secret")).unwrap();
    assert_eq!(json, r#""[REDACTED]""#);
}

#[test]
fn api_key_deserialize_captures() {
    let key: ApiKey = serde_json::from_str(r#""sk-incoming""#).unwrap();
    assert_eq!(key.expose(), "sk-incoming");
}

#[test]
fn api_key_deserialize_empty() {
    let key: ApiKey = serde_json::from_str(r#""""#).unwrap();
    assert!(key.is_empty());
}

#[test]
fn api_key_roundtrip_redacts() {
    let key = ApiKey::new("original");
    let json = serde_json::to_string(&key).unwrap();
    assert_eq!(json, r#""[REDACTED]""#);
    let back: ApiKey = serde_json::from_str(&json).unwrap();
    assert_eq!(back.expose(), "[REDACTED]");
}

#[test]
fn api_key_clone() {
    let key = ApiKey::new("sk-clone");
    let cloned = key.clone();
    assert_eq!(key.expose(), cloned.expose());
    assert_eq!(format!("{:?}", cloned), "ApiKey([REDACTED])");
}

#[test]
fn constant_time_eq_equal() {
    assert!(constant_time_eq(b"same", b"same"));
}

#[test]
fn constant_time_eq_different() {
    assert!(!constant_time_eq(b"abc", b"abd"));
}

#[test]
fn constant_time_eq_different_lengths() {
    assert!(!constant_time_eq(b"short", b"longer"));
    assert!(!constant_time_eq(b"", b"a"));
    assert!(!constant_time_eq(b"a", b""));
}

#[test]
fn constant_time_eq_empty() {
    assert!(constant_time_eq(b"", b""));
}

#[test]
fn constant_time_eq_single_byte() {
    assert!(constant_time_eq(b"x", b"x"));
    assert!(!constant_time_eq(b"x", b"y"));
}

#[test]
fn constant_time_str_eq_equal() {
    assert!(constant_time_str_eq("pw", "pw"));
}

#[test]
fn constant_time_str_eq_different() {
    assert!(!constant_time_str_eq("pw", "pw!"));
}

#[test]
fn constant_time_str_eq_empty() {
    assert!(constant_time_str_eq("", ""));
}

#[test]
fn validation_error_display() {
    let err = ValidationError { field: "email".to_string(), message: "invalid".to_string() };
    assert_eq!(format!("{}", err), "email: invalid");
}

#[test]
fn validation_error_debug() {
    let err = ValidationError { field: "name".to_string(), message: "blank".to_string() };
    let dbg = format!("{:?}", err);
    assert!(dbg.contains("name"));
    assert!(dbg.contains("blank"));
}

#[test]
fn validation_error_is_std_error() {
    let err = ValidationError { field: "f".to_string(), message: "m".to_string() };
    let _: &dyn std::error::Error = &err;
}

#[test]
fn validate_not_empty_valid() {
    assert!(validate_not_empty("name", "Alice").is_ok());
    assert!(validate_not_empty("name", "  x  ").is_ok());
}

#[test]
fn validate_not_empty_blank() {
    let r = validate_not_empty("name", "");
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().field, "name");
}

#[test]
fn validate_not_empty_whitespace() {
    assert!(validate_not_empty("f", "   ").is_err());
}

#[test]
fn validate_max_length_within() {
    assert!(validate_max_length("n", "short", 100).is_ok());
}

#[test]
fn validate_max_length_exact() {
    assert!(validate_max_length("n", "exact", 5).is_ok());
}

#[test]
fn validate_max_length_exceeded() {
    let r = validate_max_length("n", "too long", 5);
    assert!(r.is_err());
    assert!(r.unwrap_err().message.contains("5"));
}

#[test]
fn validate_max_length_empty() {
    assert!(validate_max_length("f", "", 100).is_ok());
}

#[test]
fn validate_json_size_within() {
    assert!(validate_json_size("p", &serde_json::json!({"k": "v"}), 100).is_ok());
}

#[test]
fn validate_json_size_exceeded() {
    let r = validate_json_size("p", &serde_json::json!({"k": "x".repeat(100)}), 50);
    assert!(r.is_err());
    assert!(r.unwrap_err().message.contains("50"));
}

#[test]
fn validate_json_size_nested() {
    assert!(validate_json_size("d", &serde_json::json!({"a": {"b": "c"}}), 100).is_ok());
}

#[test]
fn proptest_constant_time_eq_reflexive() {
    proptest::proptest!(|(s: String)| {
        let bytes = s.as_bytes();
        assert!(constant_time_eq(bytes, bytes));
    });
}

#[test]
fn proptest_constant_time_str_eq_reflexive() {
    proptest::proptest!(|(s: String)| {
        assert!(constant_time_str_eq(&s, &s));
    });
}

#[test]
fn proptest_api_key_roundtrip() {
    proptest::proptest!(|(s: String)| {
        let key = ApiKey::new(s.clone());
        assert_eq!(key.expose(), s);
    });
}

#[test]
fn proptest_validate_not_empty_reflexive() {
    proptest::proptest!(|(s: String)| {
        if !s.trim().is_empty() {
            assert!(validate_not_empty("f", &s).is_ok());
        }
    });
}
