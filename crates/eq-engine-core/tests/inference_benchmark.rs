//! # Inference Benchmark (S1-016)
//!
//! Measures end-to-end inference latency through the LlamaCppAdapter.
//! Results are printed to stderr so they don't interfere with test pass/fail.
//!
//! ## Running
//!
//! ```bash
//! cargo test -p eq-engine-core --test inference_benchmark -- --ignored --nocapture
//! ```
//!
//! ## Variables
//!
//! - `EQ_BENCH_ITERATIONS` — number of samples (default: 5)
//! - `EQ_BACKEND_URL` — override backend URL (default: auto-detect)
//! - `EQ_WARMUP` — perform warm-up run (set to "1")

use eq_engine_core::slm_adapter::{LlamaCppAdapter, ModelProfile, SLMAdapter};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

// ============================================================================
// Helpers
// ============================================================================

fn discover_backend() -> Option<(String, u16)> {
    let timeout = Duration::from_secs(2);

    let can_connect = |host: &str, port: u16| -> bool {
        let addr: SocketAddr = match format!("{}:{}", host, port).parse() {
            Ok(a) => a,
            Err(_) => return false,
        };
        TcpStream::connect_timeout(&addr, timeout).is_ok()
    };

    if let Ok(url) = std::env::var("EQ_BACKEND_URL") {
        let url = url.trim().to_lowercase();
        let url = url.strip_prefix("http://").unwrap_or(&url);
        if let Some((host, port_str)) = url.split_once(':') {
            if let Ok(port) = port_str.parse::<u16>() {
                if can_connect(host, port) {
                    return Some((host.to_string(), port));
                }
            }
        }
        return None;
    }

    // Try backend directly first (port 9120) — this is where the SLM lives
    if can_connect("127.0.0.1", 9120) {
        return Some(("127.0.0.1".to_string(), 9120));
    }

    // Fall back to router (port 8080)
    if can_connect("127.0.0.1", 8080) {
        return Some(("127.0.0.1".to_string(), 8080));
    }

    None
}

fn create_live_adapter() -> Option<LlamaCppAdapter> {
    let (host, port) = discover_backend()?;
    let profile = ModelProfile {
        host,
        port,
        timeout_ms: 300_000,
        ..Default::default()
    };
    Some(LlamaCppAdapter::new(profile))
}

/// Test prompts covering different emotional categories
const TEST_PROMPTS: &[&str] = &[
    "I'm really frustrated with this bug in the build system. It keeps failing and I can't figure out why.",
    "I have a big presentation tomorrow and I'm really nervous about it. What if I mess up?",
    "Can you explain how the EQ Gateway pipeline works step by step?",
    "I've been working on this for weeks and nothing is going right. I feel like giving up.",
    "I'm really worried about my friend — they haven't been themselves lately and I'm concerned.",
];

// ============================================================================
// Benchmark
// ============================================================================

#[test]
#[ignore = "requires running llama.cpp backend"]
fn inference_latency_benchmark() {
    let adapter = match create_live_adapter() {
        Some(a) => a,
        None => {
            eprintln!("SKIP: no backend reachable");
            return;
        }
    };

    let iterations: usize = std::env::var("EQ_BENCH_ITERATIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    let do_warmup = std::env::var("EQ_WARMUP").is_ok();
    let prompts: Vec<&str> = TEST_PROMPTS
        .iter()
        .cycle()
        .take(iterations)
        .copied()
        .collect();

    // Warm-up
    if do_warmup {
        eprintln!("Warm-up run...");
        let _ = adapter.classify("Hello, this is a warm-up message.", "bench-warmup");
    }

    // Run benchmark
    let mut times: Vec<u64> = Vec::with_capacity(iterations);
    let mut successes = 0u32;
    let mut failures = 0u32;

    eprintln!(
        "\n=== Inference Benchmark ({} iterations) ===",
        iterations
    );

    for (i, prompt) in prompts.iter().enumerate() {
        let start = Instant::now();
        let result = adapter.classify(prompt, &format!("bench-session-{}", i));
        let elapsed = start.elapsed();

        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        let status = if result.is_ok() { "OK" } else { "PARSE_ERR" };

        if result.is_ok() {
            successes += 1;
        } else {
            failures += 1;
        }

        eprintln!(
            "  [{:3}/{}] {:>10.0}ms  {}  {:>.50}...",
            i + 1,
            iterations,
            elapsed_ms,
            status,
            prompt,
        );

        times.push(elapsed_ms as u64);
    }

    // Statistics
    times.sort_unstable();
    let total: u64 = times.iter().sum();
    let mean = total as f64 / times.len() as f64;
    let min = times.first().copied().unwrap_or(0);
    let max = times.last().copied().unwrap_or(0);
    let median = if times.len() % 2 == 0 {
        (times[times.len() / 2 - 1] + times[times.len() / 2]) as f64 / 2.0
    } else {
        times[times.len() / 2] as f64
    };

    eprintln!("\n=== Results ===");
    eprintln!("  Iterations: {}", iterations);
    eprintln!("  Successes:  {} (parse OK)", successes);
    eprintln!("  Failures:   {} (parse error)", failures);
    eprintln!("  Mean:       {:.0} ms", mean);
    eprintln!("  Median:     {:.0} ms", median);
    eprintln!("  Min:        {} ms", min);
    eprintln!("  Max:        {} ms", max);
    eprintln!("  Total:      {} ms", total);
    eprintln!();

    // Sanity: total time should be reasonable (within 5 minutes)
    assert!(
        total < 300_000,
        "Benchmark total time exceeded 5 minutes ({} ms)",
        total
    );
}
