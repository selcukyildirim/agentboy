use std::time::Instant;
use tool_runtime::tools::csv_engine::CsvEngine;

#[test]
fn parses_10k_rows_correctly() {
    let mut csv = String::from("name,amount,category\n");
    for i in 0..10_000 {
        csv.push_str(&format!("item{i},{i},cat{}\n", i % 10));
    }

    let start = Instant::now();
    let rows = CsvEngine::new().parse(&csv).expect("parse should succeed");
    let elapsed = start.elapsed();

    assert_eq!(rows.len(), 10_001, "header + 10k rows");
    assert_eq!(rows[1][0], "item0");
    assert_eq!(rows[10_000][0], "item9999");

    // Generous bound to catch pathological (super-linear) regressions.
    assert!(
        elapsed.as_secs() < 5,
        "10k-row parse took too long: {elapsed:?}"
    );
}

#[test]
fn filters_10k_rows_correctly() {
    let mut csv = String::from("name,category\n");
    for i in 0..10_000 {
        csv.push_str(&format!("item{i},cat{}\n", i % 4));
    }

    let engine = CsvEngine::new();
    let filtered = engine
        .filter(&csv, "category", "cat0")
        .expect("filter should succeed");

    let lines = filtered.lines().count();
    assert_eq!(lines, 1 + 2500, "header + 2500 matching rows");
}

#[test]
fn json_output_preserves_quoted_fields() {
    let csv = "name,note\n\"Acme, Inc.\",\"hello, world\"\nBeta,plain\n";
    let json = CsvEngine::new()
        .to_json(csv)
        .expect("to_json should succeed");
    assert_eq!(json[0]["name"], "Acme, Inc.");
    assert_eq!(json[0]["note"], "hello, world");
    assert_eq!(json[1]["name"], "Beta");
}
