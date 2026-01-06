//! Verification example for the case-preserving http crate fork.
//!
//! This example demonstrates that custom headers preserve their original casing
//! when sent via reqwest (which uses the patched http crate).
//!
//! ## Running
//!
//! 1. Start the Python server: `python3 server.py`
//! 2. Run this client: `cargo run`
//!
//! You should see "VERIFICATION SUCCESS: X-Custom-Header found" in the server output.

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    // Custom header with mixed case - THIS SHOULD BE PRESERVED
    let custom_header = HeaderName::from_str("X-Custom-Header")?;
    println!("Custom header as_str(): {}", custom_header.as_str());
    assert_eq!(
        custom_header.as_str(),
        "X-Custom-Header",
        "Case should be preserved!"
    );

    // Standard header - will be normalized to lowercase (expected behavior)
    let standard_header = HeaderName::from_str("Content-Type")?;
    println!("Standard header as_str(): {}", standard_header.as_str());
    // Standard headers are lowercase for h2 compatibility
    assert_eq!(standard_header.as_str(), "content-type");

    let mut headers = HeaderMap::new();
    headers.insert(custom_header, HeaderValue::from_static("CustomValue"));
    headers.insert(
        HeaderName::from_str("X-Another-Custom")?,
        HeaderValue::from_static("AnotherValue"),
    );

    println!("\nSending POST request to http://localhost:8000 with headers:");
    for (name, value) in headers.iter() {
        println!("  {}: {:?}", name.as_str(), value);
    }

    let response = client
        .post("http://localhost:8000")
        .headers(headers)
        .body("test body")
        .send()?;

    println!("\nResponse status: {}", response.status());

    if response.status().is_success() {
        println!("✓ Request succeeded! Check server output for header verification.");
    } else {
        println!("✗ Request failed with status: {}", response.status());
    }

    Ok(())
}
