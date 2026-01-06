# Verification Example

This directory contains a working example demonstrating how to:
1. Use `[patch.crates-io]` to substitute the case-preserving http crate
2. Verify that custom headers preserve their casing end-to-end

## Files

- `Cargo.toml` - Shows the `[patch]` configuration
- `src/main.rs` - Rust client that sends custom headers
- `server.py` - Python server that echoes received headers

## Running

### 1. Start the Python server

```bash
python3 server.py
```

The server listens on `http://localhost:8000` and prints received headers.

### 2. Run the Rust client

```bash
cargo run
```

### Expected Output

**Server output:**
```
--- Headers received ---
X-Custom-Header: CustomValue
X-Another-Custom: AnotherValue
...
------------------------
VERIFICATION SUCCESS: X-Custom-Header found
```

**Client output:**
```
Custom header as_str(): X-Custom-Header
Standard header as_str(): content-type

Sending POST request to http://localhost:8000 with headers:
  X-Custom-Header: "CustomValue"
  X-Another-Custom: "AnotherValue"

Response status: 200 OK
✓ Request succeeded! Check server output for header verification.
```

## Key Points

1. **Custom headers preserve case**: `X-Custom-Header` stays as `X-Custom-Header`
2. **Standard headers are lowercase**: `Content-Type` becomes `content-type` (for h2 compatibility)
3. **The `[patch]` directive** makes reqwest (and all its dependencies) use our patched http crate
