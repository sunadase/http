# HTTP (Case-Preserving Fork)

A fork of the [hyperium/http](https://github.com/hyperium/http) crate that **preserves the original casing of CUSTOM HTTP header names**.

## Why This Fork?

Some legacy servers require HTTP headers with specific casing (e.g., `X-Custom-Header` instead of `x-custom-header`). The standard `http` crate normalizes all header names to lowercase, which breaks compatibility with such servers.

This fork modifies the `http` crate to:
- **Preserve original casing** for custom headers (e.g., `X-Custom-Header` stays as-is)
- **Maintain case-insensitive comparison** for Hash/Eq (required for correct `HeaderMap` behavior)
- **Stay compatible** with `reqwest`, `hyper`, `h2`, and the broader Rust HTTP ecosystem

## Behavior

| Header Type | as_str() Output | Notes |
|-------------|-----------------|-------|
| Custom headers | Original casing preserved | `X-Custom-Header` → `"X-Custom-Header"` |
| Standard headers | Lowercase | `Content-Type` → `"content-type"` |

### Why Standard Headers Are Lowercase

Standard headers (Content-Type, Accept, etc.) are normalized to lowercase to maintain compatibility with the `h2` crate, which uses `HeaderName` constants in `match` patterns. This requires `StructuralPartialEq`, which is only possible with derived (not manual) `PartialEq`.

For most use cases, this is acceptable because:
1. RFC 7230 requires HTTP headers to be case-insensitive
2. Custom headers (`X-*` or non-standard names) are typically where case sensitivity matters for legacy systems

## Usage

### Using with Cargo Patch

To use this fork with crates like `reqwest` that depend on `http`:

```toml
[dependencies]
reqwest = "0.12"

[patch.crates-io]
http = { git = "https://github.com/sunadase/http.git" }
# Or use a local path:
# http = { path = "/path/to/this/http" }
```

### Example

```rust
use http::header::{HeaderMap, HeaderName, HeaderValue};
use std::str::FromStr;

fn main() {
    // Custom headers preserve case
    let custom = HeaderName::from_str("X-Custom-Header").unwrap();
    assert_eq!(custom.as_str(), "X-Custom-Header"); // Case preserved!

    // Standard headers are normalized to lowercase
    let standard = HeaderName::from_str("Content-Type").unwrap();
    assert_eq!(standard.as_str(), "content-type");

    // Case-insensitive equality still works
    let h1 = HeaderName::from_str("X-Custom-Header").unwrap();
    let h2 = HeaderName::from_str("x-custom-header").unwrap();
    assert_eq!(h1, h2); // Equal despite different casing

    // HeaderMap lookups are case-insensitive
    let mut map = HeaderMap::new();
    map.insert(h1, HeaderValue::from_static("value"));
    assert!(map.get("x-custom-header").is_some()); // Found!
}
```

## Verification

The `verification/` directory contains a working example project that demonstrates:
1. How to use `[patch.crates-io]` to substitute this fork
2. A Python server + Rust client test confirming case preservation works end-to-end

To run the verification:

```bash
cd verification
python3 server.py &
cargo run
# Should output: VERIFICATION SUCCESS: X-Custom-Header found
```

## Testing

```bash
# Run all tests
cargo test

# Run case preservation specific tests
cargo test --test case_preservation
cargo test --test e2e_verification
```

## Technical Details

### Changes from Upstream

1. **`Repr` enum simplified** - Removed `StandardPreserved` variant, using only `Standard` and `Custom`

2. **`Custom` wrapper uses `ByteStr`** - Stores original bytes, implements case-insensitive `Hash` and `PartialEq`

3. **`from_static` updated** - Case-insensitive matching for standard headers in const context

4. **`parse_hdr` updated** - Creates lowercase buffer for standard header matching while preserving original input for custom headers

5. **Internal Renaming** - Renamed `MaybeLower` to `MaybeValidated` to accurately reflect that custom headers are validated for safety but not necessarily lowercased

6. **Validation Tables** - Introduced separate `HEADER_CHARS` (case-preserving) and `HEADER_CHARS_LOWER` (normalizing) tables for flexible parsing

### Key Files Modified

- `src/header/name.rs` - Header name parsing and storage
- `src/byte_str.rs` - Case-insensitive hashing for ByteStr

## Limitations

1. **Standard headers don't preserve case** - `Content-Type` always becomes `content-type`
2. **HTTP/2 and HTTP/3 compliance** - These protocols require lowercase headers anyway, so case preservation only matters for HTTP/1.1
3. **This is a fork** - You'll need to maintain updates from upstream

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

## Credits

Based on [hyperium/http](https://github.com/hyperium/http) by:
- Alex Crichton
- Carl Lerche  
- Sean McArthur
