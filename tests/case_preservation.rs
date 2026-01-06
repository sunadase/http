/// Tests for case-preserving header names.
///
/// These tests verify that the http crate preserves the original casing
/// of header names while maintaining case-insensitive comparison semantics.

#[test]
fn test_case_preserved_custom_header() {
    use http::header::HeaderName;
    use std::str::FromStr;

    // Create a custom header with mixed case
    let name = HeaderName::from_str("X-Custom-Header").unwrap();

    // The original casing should be preserved in as_str()
    assert_eq!(name.as_str(), "X-Custom-Header");

    // Case-insensitive equality should still work
    assert_eq!(name, HeaderName::from_str("x-custom-header").unwrap());
    assert_eq!(name, HeaderName::from_str("X-CUSTOM-HEADER").unwrap());
}

#[test]
fn test_standard_header_always_lowercase() {
    use http::header::{HeaderName, CONTENT_LENGTH};
    use std::str::FromStr;

    // Standard headers always serialize as lowercase (no case preservation)
    // This is a trade-off for compatibility with h2/reqwest
    let name = HeaderName::from_str("Content-Length").unwrap();

    // Standard headers are normalized to lowercase
    assert_eq!(name.as_str(), "content-length");

    // Should still equal the standard constant
    assert_eq!(name, CONTENT_LENGTH);

    // Lowercase version should also equal
    let name_lower = HeaderName::from_str("content-length").unwrap();
    assert_eq!(name_lower.as_str(), "content-length");
    assert_eq!(name, name_lower);
}

#[test]
fn test_headermap_lookup_case_insensitive() {
    use http::header::{HeaderMap, HeaderName, HeaderValue};
    use std::str::FromStr;

    let mut map = HeaderMap::new();

    // Insert with mixed case
    let name = HeaderName::from_str("X-Custom-Header").unwrap();
    map.insert(name.clone(), HeaderValue::from_static("value1"));

    // Lookup should work case-insensitively
    assert!(map.get("x-custom-header").is_some());
    assert!(map.get("X-Custom-Header").is_some());
    assert!(map.get("X-CUSTOM-HEADER").is_some());

    // All lookups should return the same value
    assert_eq!(map.get("x-custom-header").unwrap(), "value1");
}

#[test]
fn test_headermap_case_insensitive_merge() {
    use http::header::{HeaderMap, HeaderName, HeaderValue};
    use std::str::FromStr;

    let mut map = HeaderMap::new();

    // Insert with different casings - should merge into one entry
    let name1 = HeaderName::from_str("content-length").unwrap();
    let name2 = HeaderName::from_str("Content-Length").unwrap();

    map.insert(name1, HeaderValue::from_static("100"));
    map.insert(name2, HeaderValue::from_static("200"));

    // Should only have one entry
    assert_eq!(map.keys_len(), 1);
    // The value should be the second one (overwritten)
    assert_eq!(map.get("content-length").unwrap(), "200");
}

#[test]
fn test_hash_consistency() {
    use http::header::HeaderName;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::str::FromStr;

    fn compute_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    // Same header with different casing should hash the same
    let h1 = HeaderName::from_str("X-Custom-Header").unwrap();
    let h2 = HeaderName::from_str("x-custom-header").unwrap();
    let h3 = HeaderName::from_str("X-CUSTOM-HEADER").unwrap();

    assert_eq!(compute_hash(&h1), compute_hash(&h2));
    assert_eq!(compute_hash(&h2), compute_hash(&h3));

    // Standard headers with different casing should hash the same
    let s1 = HeaderName::from_str("Content-Type").unwrap();
    let s2 = HeaderName::from_str("content-type").unwrap();

    assert_eq!(compute_hash(&s1), compute_hash(&s2));
}

#[test]
fn test_from_bytes_preserves_case() {
    use http::header::HeaderName;

    // from_bytes should preserve case for custom headers
    let h = HeaderName::from_bytes(b"X-Custom-Header").unwrap();
    assert_eq!(h.as_str(), "X-Custom-Header");

    let h_lower = HeaderName::from_bytes(b"x-custom-header").unwrap();
    assert_eq!(h_lower.as_str(), "x-custom-header");

    // But they should still be equal
    assert_eq!(h, h_lower);
}

#[test]
fn test_different_headers_not_equal() {
    use http::header::HeaderName;
    use std::str::FromStr;

    let h1 = HeaderName::from_str("X-Foo").unwrap();
    let h2 = HeaderName::from_str("X-Bar").unwrap();
    assert_ne!(h1, h2);

    let h3 = HeaderName::from_str("x-foo").unwrap();
    assert_eq!(h1, h3); // Same header, different case
    assert_ne!(h2, h3); // Different headers
}

#[test]
fn test_from_static_preserves_case_for_custom() {
    use http::header::HeaderName;

    let h = HeaderName::from_static("X-My-Custom-Header");
    assert_eq!(h.as_str(), "X-My-Custom-Header");
}

#[test]
fn test_boundary_63_byte_header() {
    use http::header::HeaderName;
    use std::str::FromStr;

    // Test header at exactly 63 bytes (boundary between short/long paths)
    let name_63 = format!("X-{}", "a".repeat(61)); // X- + 61 = 63
    assert_eq!(name_63.len(), 63);

    let h = HeaderName::from_str(&name_63).unwrap();
    assert_eq!(h.as_str(), name_63);
}

#[test]
fn test_boundary_64_byte_header() {
    use http::header::HeaderName;
    use std::str::FromStr;

    // Test header at 64 bytes (first long header)
    let name_64 = format!("X-{}", "a".repeat(62)); // X- + 62 = 64
    assert_eq!(name_64.len(), 64);

    let h = HeaderName::from_str(&name_64).unwrap();
    assert_eq!(h.as_str(), name_64);
}

#[test]
fn test_from_lowercase_rejects_uppercase() {
    use http::header::HeaderName;

    // from_lowercase should reject uppercase (HTTP/2 requirement)
    assert!(HeaderName::from_lowercase(b"Content-Type").is_err());
    assert!(HeaderName::from_lowercase(b"X-CUSTOM").is_err());

    // But lowercase should work
    assert!(HeaderName::from_lowercase(b"content-type").is_ok());
    assert!(HeaderName::from_lowercase(b"x-custom").is_ok());
}
