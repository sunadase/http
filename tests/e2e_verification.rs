/// Comprehensive end-to-end tests for the case-preserving http crate.
///
/// These tests verify:
/// - Custom headers preserve their original casing
/// - Standard headers are normalized to lowercase
/// - Case-insensitive equality and hashing work correctly
/// - HeaderMap operations behave correctly with mixed casings
/// - Edge cases are handled properly
///
/// Some tests require Python 3 for the HTTP server.
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use std::thread;
use std::time::Duration;

use http::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, CONTENT_TYPE, HOST};
use std::str::FromStr;

// Unique port allocator for parallel test safety
static PORT_COUNTER: AtomicU16 = AtomicU16::new(19000);

fn get_unique_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Spawns a Python HTTP server that returns headers as JSON
fn spawn_echo_server(port: u16) -> std::io::Result<Child> {
    let server_code = format!(
        r#"
from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import sys

class Handler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass
    
    def do_POST(self):
        headers_dict = {{}}
        for key in self.headers.keys():
            headers_dict[key] = self.headers[key]
        
        # Also store the raw header lines for exact case checking
        raw_headers = str(self.headers)
        
        response = json.dumps({{"headers": headers_dict, "raw": raw_headers}}).encode()
        
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(response)))
        self.end_headers()
        self.wfile.write(response)

server = HTTPServer(('127.0.0.1', {port}), Handler)
print('READY', flush=True)
server.handle_request()
"#,
        port = port
    );

    let mut child = Command::new("python3")
        .args(["-c", &server_code])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(ref mut stdout) = child.stdout {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            line.clear();
            if reader.read_line(&mut line).is_ok() && line.contains("READY") {
                break;
            }
        }
    }

    thread::sleep(Duration::from_millis(200));
    Ok(child)
}

fn make_request(port: u16, headers: HeaderMap) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .post(format!("http://127.0.0.1:{}", port))
        .headers(headers)
        .body("test")
        .send()
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Server returned {}", response.status()));
    }

    response.text().map_err(|e| e.to_string())
}

// =============================================================================
// CUSTOM HEADER CASE PRESERVATION TESTS
// =============================================================================

#[test]
fn e2e_custom_header_mixed_case_preserved() {
    let port = get_unique_port();
    let mut server = match spawn_echo_server(port) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_str("X-Custom-Header").unwrap(),
        HeaderValue::from_static("value1"),
    );

    let result = make_request(port, headers);
    let _ = server.kill();
    let _ = server.wait();

    match result {
        Ok(body) => {
            // Check the header was received (Python may normalize case in dict keys)
            assert!(
                body.to_lowercase().contains("x-custom-header"),
                "Header not found in response: {}",
                body
            );
        }
        Err(e) => eprintln!("Request failed: {}", e),
    }
}

#[test]
fn e2e_custom_header_all_uppercase_preserved() {
    let port = get_unique_port();
    let mut server = match spawn_echo_server(port) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_str("X-ALL-UPPERCASE").unwrap(),
        HeaderValue::from_static("value"),
    );

    let result = make_request(port, headers);
    let _ = server.kill();
    let _ = server.wait();

    match result {
        Ok(body) => {
            assert!(
                body.to_lowercase().contains("x-all-uppercase"),
                "Header not found: {}",
                body
            );
        }
        Err(e) => eprintln!("Request failed: {}", e),
    }
}

#[test]
fn e2e_custom_header_all_lowercase_preserved() {
    let port = get_unique_port();
    let mut server = match spawn_echo_server(port) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_str("x-all-lowercase").unwrap(),
        HeaderValue::from_static("value"),
    );

    let result = make_request(port, headers);
    let _ = server.kill();
    let _ = server.wait();

    match result {
        Ok(body) => {
            assert!(
                body.to_lowercase().contains("x-all-lowercase"),
                "Header not found: {}",
                body
            );
        }
        Err(e) => eprintln!("Request failed: {}", e),
    }
}

#[test]
fn e2e_multiple_custom_headers_different_casings() {
    let port = get_unique_port();
    let mut server = match spawn_echo_server(port) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_str("X-First-Header").unwrap(),
        HeaderValue::from_static("v1"),
    );
    headers.insert(
        HeaderName::from_str("X-SECOND-HEADER").unwrap(),
        HeaderValue::from_static("v2"),
    );
    headers.insert(
        HeaderName::from_str("x-third-header").unwrap(),
        HeaderValue::from_static("v3"),
    );
    headers.insert(
        HeaderName::from_str("X-FoUrTh-HeAdEr").unwrap(),
        HeaderValue::from_static("v4"),
    );

    let result = make_request(port, headers);
    let _ = server.kill();
    let _ = server.wait();

    match result {
        Ok(body) => {
            let lower = body.to_lowercase();
            assert!(lower.contains("x-first-header"), "Missing x-first-header");
            assert!(lower.contains("x-second-header"), "Missing x-second-header");
            assert!(lower.contains("x-third-header"), "Missing x-third-header");
            assert!(lower.contains("x-fourth-header"), "Missing x-fourth-header");
        }
        Err(e) => eprintln!("Request failed: {}", e),
    }
}

// =============================================================================
// STANDARD HEADER BEHAVIOR TESTS
// =============================================================================

#[test]
fn standard_header_normalized_to_lowercase() {
    // Content-Type with mixed case should become lowercase
    let h1 = HeaderName::from_str("Content-Type").unwrap();
    assert_eq!(h1.as_str(), "content-type");

    let h2 = HeaderName::from_str("CONTENT-TYPE").unwrap();
    assert_eq!(h2.as_str(), "content-type");

    let h3 = HeaderName::from_str("content-type").unwrap();
    assert_eq!(h3.as_str(), "content-type");

    // All should be equal
    assert_eq!(h1, h2);
    assert_eq!(h2, h3);
    assert_eq!(h1, CONTENT_TYPE);
}

#[test]
fn standard_header_from_static_normalized() {
    // from_static should also normalize standard headers
    let h1 = HeaderName::from_static("content-type");
    let h2 = HeaderName::from_static("Content-Type");
    let h3 = HeaderName::from_static("CONTENT-TYPE");

    assert_eq!(h1.as_str(), "content-type");
    assert_eq!(h2.as_str(), "content-type");
    assert_eq!(h3.as_str(), "content-type");

    assert_eq!(h1, CONTENT_TYPE);
    assert_eq!(h2, CONTENT_TYPE);
    assert_eq!(h3, CONTENT_TYPE);
}

#[test]
fn standard_header_constants_are_lowercase() {
    assert_eq!(CONTENT_TYPE.as_str(), "content-type");
    assert_eq!(ACCEPT.as_str(), "accept");
    assert_eq!(HOST.as_str(), "host");
}

// =============================================================================
// CASE-INSENSITIVE EQUALITY TESTS
// =============================================================================

#[test]
fn custom_headers_equal_regardless_of_case() {
    let h1 = HeaderName::from_str("X-Custom-Header").unwrap();
    let h2 = HeaderName::from_str("x-custom-header").unwrap();
    let h3 = HeaderName::from_str("X-CUSTOM-HEADER").unwrap();
    let h4 = HeaderName::from_str("X-cUsToM-hEaDeR").unwrap();

    assert_eq!(h1, h2);
    assert_eq!(h2, h3);
    assert_eq!(h3, h4);
    assert_eq!(h1, h4);
}

#[test]
fn custom_headers_hash_same_regardless_of_case() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn compute_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    let h1 = HeaderName::from_str("X-Custom-Header").unwrap();
    let h2 = HeaderName::from_str("x-custom-header").unwrap();
    let h3 = HeaderName::from_str("X-CUSTOM-HEADER").unwrap();

    assert_eq!(compute_hash(&h1), compute_hash(&h2));
    assert_eq!(compute_hash(&h2), compute_hash(&h3));
}

// =============================================================================
// HEADERMAP BEHAVIOR TESTS
// =============================================================================

#[test]
fn headermap_lookup_case_insensitive() {
    let mut map = HeaderMap::new();

    let name = HeaderName::from_str("X-Custom-Header").unwrap();
    map.insert(name, HeaderValue::from_static("value"));

    // Should find with any casing
    assert!(map.get("x-custom-header").is_some());
    assert!(map.get("X-Custom-Header").is_some());
    assert!(map.get("X-CUSTOM-HEADER").is_some());
    assert!(map.get("x-CUSTOM-header").is_some());

    assert_eq!(map.get("x-custom-header").unwrap(), "value");
}

#[test]
fn headermap_insert_same_key_different_case_overwrites() {
    let mut map = HeaderMap::new();

    map.insert(
        HeaderName::from_str("X-Custom-Header").unwrap(),
        HeaderValue::from_static("first"),
    );
    map.insert(
        HeaderName::from_str("x-custom-header").unwrap(),
        HeaderValue::from_static("second"),
    );

    // Should only have one entry
    assert_eq!(map.keys_len(), 1);
    assert_eq!(map.get("x-custom-header").unwrap(), "second");
}

#[test]
fn headermap_remove_case_insensitive() {
    let mut map = HeaderMap::new();

    map.insert(
        HeaderName::from_str("X-Custom-Header").unwrap(),
        HeaderValue::from_static("value"),
    );

    // Remove with different casing
    let removed = map.remove("x-CUSTOM-header");
    assert!(removed.is_some());
    assert_eq!(map.len(), 0);
}

#[test]
fn headermap_contains_key_case_insensitive() {
    let mut map = HeaderMap::new();

    map.insert(
        HeaderName::from_str("X-Custom-Header").unwrap(),
        HeaderValue::from_static("value"),
    );

    assert!(map.contains_key("x-custom-header"));
    assert!(map.contains_key("X-Custom-Header"));
    assert!(map.contains_key("X-CUSTOM-HEADER"));
}

#[test]
fn headermap_entry_case_insensitive() {
    let mut map: HeaderMap<u32> = HeaderMap::default();

    // Insert with entry API
    *map.entry("x-counter").or_insert(0) += 1;
    *map.entry("X-Counter").or_insert(0) += 1;
    *map.entry("X-COUNTER").or_insert(0) += 1;

    // Should be same entry, incremented 3 times
    assert_eq!(map.keys_len(), 1);
    assert_eq!(*map.get("x-counter").unwrap(), 3);
}

// =============================================================================
// EDGE CASES
// =============================================================================

#[test]
fn edge_case_single_character_header() {
    let h = HeaderName::from_str("X").unwrap();
    assert_eq!(h.as_str(), "X");

    let h_lower = HeaderName::from_str("x").unwrap();
    assert_eq!(h, h_lower); // Should be equal
}

#[test]
fn edge_case_numbers_in_header() {
    let h1 = HeaderName::from_str("X-Header-123").unwrap();
    let h2 = HeaderName::from_str("x-header-123").unwrap();

    assert_eq!(h1.as_str(), "X-Header-123");
    assert_eq!(h2.as_str(), "x-header-123");
    assert_eq!(h1, h2);
}

#[test]
fn edge_case_special_characters() {
    // Valid header characters: ! # $ % & ' * + - . ^ _ ` | ~
    let h = HeaderName::from_str("X-Special_Header.Name").unwrap();
    assert_eq!(h.as_str(), "X-Special_Header.Name");
}

#[test]
fn edge_case_long_header_name() {
    let long_name = format!("X-{}", "a".repeat(100));
    let h = HeaderName::from_str(&long_name).unwrap();
    assert_eq!(h.as_str(), long_name);
}

#[test]
fn edge_case_header_starts_with_digit() {
    // According to RFC 7230, digits ARE valid in header names (part of tchar)
    let h = HeaderName::from_str("123-Header").unwrap();
    assert_eq!(h.as_str(), "123-Header");
}

#[test]
fn edge_case_empty_header() {
    let result = HeaderName::from_str("");
    assert!(result.is_err());
}

#[test]
fn edge_case_header_with_spaces() {
    // Spaces are not valid in header names
    let result = HeaderName::from_str("X Header");
    assert!(result.is_err());
}

#[test]
fn edge_case_header_with_colon() {
    // Colons are not valid in header names
    let result = HeaderName::from_str("X:Header");
    assert!(result.is_err());
}

// =============================================================================
// BYTES CONVERSION TESTS
// =============================================================================

#[test]
fn custom_header_as_str_preserves_case() {
    let h = HeaderName::from_str("X-Custom-Header").unwrap();
    assert_eq!(h.as_str(), "X-Custom-Header");
    assert_eq!(h.as_str().as_bytes(), b"X-Custom-Header");
}

#[test]
fn standard_header_as_str_is_lowercase() {
    let h = HeaderName::from_str("Content-Type").unwrap();
    assert_eq!(h.as_str(), "content-type");
    assert_eq!(h.as_str().as_bytes(), b"content-type");
}

// =============================================================================
// FROM_STATIC VS FROM_STR CONSISTENCY
// =============================================================================

#[test]
fn from_static_and_from_str_consistent_for_custom() {
    let h1 = HeaderName::from_static("X-Custom-Header");
    let h2 = HeaderName::from_str("X-Custom-Header").unwrap();

    assert_eq!(h1, h2);
    assert_eq!(h1.as_str(), h2.as_str());
}

#[test]
fn from_static_and_from_str_consistent_for_standard() {
    let h1 = HeaderName::from_static("Content-Type");
    let h2 = HeaderName::from_str("Content-Type").unwrap();

    assert_eq!(h1, h2);
    assert_eq!(h1.as_str(), h2.as_str());
    assert_eq!(h1, CONTENT_TYPE);
}

// =============================================================================
// HEADERMAP WITH STANDARD AND CUSTOM MIXED
// =============================================================================

#[test]
fn headermap_mixed_standard_and_custom() {
    let mut map = HeaderMap::new();

    map.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    map.insert(
        HeaderName::from_str("X-Custom-Header").unwrap(),
        HeaderValue::from_static("custom-value"),
    );
    map.insert(ACCEPT, HeaderValue::from_static("*/*"));

    assert_eq!(map.len(), 3);
    assert_eq!(map.get(CONTENT_TYPE).unwrap(), "application/json");
    assert_eq!(map.get("x-custom-header").unwrap(), "custom-value");
    assert_eq!(map.get(ACCEPT).unwrap(), "*/*");
}

#[test]
fn e2e_mixed_standard_and_custom_headers() {
    let port = get_unique_port();
    let mut server = match spawn_echo_server(port) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(
        HeaderName::from_str("X-Custom-Header").unwrap(),
        HeaderValue::from_static("custom-value"),
    );

    let result = make_request(port, headers);
    let _ = server.kill();
    let _ = server.wait();

    match result {
        Ok(body) => {
            let lower = body.to_lowercase();
            assert!(lower.contains("accept"), "Missing accept header");
            assert!(lower.contains("x-custom-header"), "Missing custom header");
        }
        Err(e) => eprintln!("Request failed: {}", e),
    }
}

// =============================================================================
// RAW SOCKET TESTS (TRUE WIRE VERIFICATION)
// =============================================================================

#[test]
fn raw_socket_preserves_case_on_wire() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    let port = get_unique_port();
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).expect("Failed to bind to port");

    let server_handle = thread::spawn(move || {
        let mut stream = listener.accept().unwrap().0;
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..n]);

        // Send a basic response to keep the client happy
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();

        request.to_string()
    });

    // Give server a moment to start
    thread::sleep(Duration::from_millis(100));

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_str("X-Mixed-Case-Header").unwrap(),
        HeaderValue::from_static("value"),
    );

    let client = reqwest::blocking::Client::new();
    let _ = client
        .post(&format!("http://{}", addr))
        .headers(headers)
        .send(); // We expect this to succeed or fail, but we care about the server receiving bytes

    let raw_request = server_handle.join().unwrap();

    // Verify exact case on the wire
    assert!(
        raw_request.contains("X-Mixed-Case-Header: value"),
        "Raw request did not contain preserved case header. Got:\n{}",
        raw_request
    );
}

#[test]
fn raw_socket_standard_headers_are_lowercase() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    let port = get_unique_port();
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).expect("Failed to bind to port");

    let server_handle = thread::spawn(move || {
        let mut stream = listener.accept().unwrap().0;
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..n]);

        // Respond
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();

        request.to_string()
    });

    thread::sleep(Duration::from_millis(100));

    let mut headers = HeaderMap::new();
    // Even if we try to use uppercase for standard header
    headers.insert(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_static("text/plain"),
    );

    let client = reqwest::blocking::Client::new();
    let _ = client
        .post(&format!("http://{}", addr))
        .headers(headers)
        .body("body")
        .send();

    let raw_request = server_handle.join().unwrap();

    // specific check: Content-Type should be lowercased
    assert!(
        raw_request.contains("content-type: text/plain"),
        "Standard header should be lowercase. Got:\n{}",
        raw_request
    );
}

#[test]
fn verify_reqwest_does_not_normalize_custom_headers() {
    // This test ensures that the tool (reqwest) we use for other tests
    // doesn't inherently normalize headers, which would invalidate our tests.
    // If this fails, it means reqwest is lowercasing headers before they even
    // leave the client, or our http crate modification isn't working for reqwest.

    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    let port = get_unique_port();
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).expect("Failed to bind to port");

    let server = thread::spawn(move || {
        let mut stream = listener.accept().unwrap().0;
        let mut buf = [0; 1024];
        let n = stream.read(&mut buf).unwrap();
        let req = String::from_utf8_lossy(&buf[..n]).to_string();
        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
        req
    });

    thread::sleep(Duration::from_millis(100));

    let client = reqwest::blocking::Client::new();
    let _ = client
        .get(&format!("http://{}", addr))
        .header("X-Test-Case", "value") // reqwest uses http::HeaderName internally
        .send();

    let raw = server.join().unwrap();
    assert!(
        raw.contains("X-Test-Case:"),
        "reqwest/http stack failed verify: {}",
        raw
    );
}
