use agent_common::error::{AppError, AppResult};
use std::collections::HashMap;

pub fn parse_csv_to_maps(csv: &str) -> AppResult<Vec<HashMap<String, String>>> {
    let mut rdr = csv::Reader::from_reader(csv.as_bytes());
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| AppError::Validation(format!("CSV header error: {}", e)))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let mut records = Vec::new();
    for result in rdr.records() {
        let record =
            result.map_err(|e| AppError::Validation(format!("CSV record error: {}", e)))?;
        let map: HashMap<String, String> = headers
            .iter()
            .zip(record.iter())
            .map(|(h, v)| (h.clone(), v.to_string()))
            .collect();
        records.push(map);
    }
    Ok(records)
}

pub fn sanitize_csv_value(s: &str) -> String {
    let trimmed = s.trim();
    if let Some(first) = trimmed.chars().next() {
        if matches!(first, '=' | '+' | '-' | '@' | '\t' | '\r' | '\n') {
            return format!("'{}", trimmed);
        }
    }
    s.to_string()
}

pub fn sanitize_csv_content(csv: &str) -> String {
    csv.lines()
        .map(|line| {
            line.split(',')
                .map(|field| sanitize_csv_value(field))
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn parse_f64_locale(s: &str) -> Option<f64> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }

    let has_dot = trimmed.contains('.');
    let has_comma = trimmed.contains(',');

    match (has_dot, has_comma) {
        (true, true) => {
            let last_separator = trimmed.rfind(|c| c == '.' || c == ',');
            if let Some(pos) = last_separator {
                let integer_part: String = trimmed[..pos]
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                let decimal_part: String = trimmed[pos + 1..]
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                let combined = format!("{}.{}", integer_part, decimal_part);
                combined.parse::<f64>().ok()
            } else {
                None
            }
        }
        (true, false) => {
            let clean: String = trimmed
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            clean.parse::<f64>().ok()
        }
        (false, true) => {
            let clean: String = trimmed
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == ',')
                .collect();
            if let Some(pos) = clean.rfind(',') {
                let integer_part: String = clean[..pos]
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                let decimal_part: String = clean[pos + 1..]
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                let combined = format!("{}.{}", integer_part, decimal_part);
                combined.parse::<f64>().ok()
            } else {
                clean.parse::<f64>().ok()
            }
        }
        (false, false) => {
            let clean: String = trimmed.chars().filter(|c| c.is_ascii_digit() || *c == '-').collect();
            clean.parse::<f64>().ok()
        }
    }
}

pub fn parse_csv_column_f64(
    csv: &str,
    category_col: &str,
    amount_col: &str,
) -> AppResult<HashMap<String, f64>> {
    let records = parse_csv_to_maps(csv)?;
    let mut result = HashMap::new();
    for record in &records {
        let category = record.get(category_col).cloned().unwrap_or_default();
        let amount = record
            .get(amount_col)
            .and_then(|v| parse_f64_locale(v))
            .unwrap_or(0.0);
        *result.entry(category).or_insert(0.0) += amount;
    }
    Ok(result)
}

pub fn csv_record_to_json(record: &HashMap<String, String>) -> serde_json::Value {
    serde_json::Value::Object(
        record
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect(),
    )
}

pub fn records_to_json(records: &[HashMap<String, String>]) -> Vec<serde_json::Value> {
    records.iter().map(csv_record_to_json).collect()
}

pub fn record_get_f64(record: &HashMap<String, String>, key: &str) -> f64 {
    record
        .get(key)
        .and_then(|v| parse_f64_locale(v))
        .unwrap_or(0.0)
}

pub fn record_get_str<'a>(record: &'a HashMap<String, String>, key: &str) -> &'a str {
    record.get(key).map(|s| s.as_str()).unwrap_or("")
}

pub fn group_by<'a>(records: &'a [HashMap<String, String>], key: &str) -> HashMap<String, Vec<&'a HashMap<String, String>>> {
    let mut groups: HashMap<String, Vec<&'a HashMap<String, String>>> = HashMap::new();
    for record in records {
        let value = record_get_str(record, key).to_string();
        groups.entry(value).or_default().push(record);
    }
    groups
}

pub fn sum_by(records: &[HashMap<String, String>], group_key: &str, amount_key: &str) -> HashMap<String, f64> {
    let mut result = HashMap::new();
    for record in records {
        let group = record_get_str(record, group_key).to_string();
        let amount = record_get_f64(record, amount_key);
        *result.entry(group).or_insert(0.0) += amount;
    }
    result
}

pub fn detect_duplicates(
    records: &[HashMap<String, String>],
    keys: &[&str],
) -> Vec<(usize, usize)> {
    use std::collections::HashMap as StdHashMap;

    let mut seen: StdHashMap<String, Vec<usize>> = StdHashMap::new();
    for (i, rec) in records.iter().enumerate() {
        let mut key_parts = Vec::new();
        for k in keys {
            let val = record_get_str(rec, k);
            if val.is_empty() {
                key_parts.clear();
                break;
            }
            key_parts.push(val);
        }
        if !key_parts.is_empty() {
            let signature = key_parts.join("\x00");
            seen.entry(signature).or_default().push(i);
        }
    }

    let mut duplicates = Vec::new();
    for indices in seen.values() {
        if indices.len() > 1 {
            for w in indices.windows(2) {
                duplicates.push((w[0], w[1]));
            }
        }
    }
    duplicates
}

pub fn standard_deviation(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

pub fn all_empty(records: &[HashMap<String, String>]) -> bool {
    records.is_empty()
}

pub fn empty_response(agent: &str, inputs: &[&str]) -> serde_json::Value {
    serde_json::json!({
        "status": "no_data",
        "agent": agent,
        "message": format!("{}: no data rows found in input(s): {}", agent, inputs.join(", ")),
        "record_count": 0,
    })
}

pub fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_to_maps() {
        let csv = "name,amount\nAcme,100\nBeta,250\n";
        let records = parse_csv_to_maps(csv).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].get("name").unwrap(), "Acme");
        assert_eq!(records[1].get("amount").unwrap(), "250");
    }

    #[test]
    fn test_parse_csv_column_f64() {
        let csv = "category,amount\nFood,50\nTransport,30\nFood,20\n";
        let result = parse_csv_column_f64(csv, "category", "amount").unwrap();
        assert_eq!(result.get("Food").unwrap(), &70.0);
        assert_eq!(result.get("Transport").unwrap(), &30.0);
    }

    #[test]
    fn test_group_by() {
        let csv = "dept,name\nEng,Alice\nEng,Bob\nSales,Charlie\n";
        let records = parse_csv_to_maps(csv).unwrap();
        let groups = group_by(&records, "dept");
        assert_eq!(groups.get("Eng").unwrap().len(), 2);
        assert_eq!(groups.get("Sales").unwrap().len(), 1);
    }

    #[test]
    fn test_sum_by() {
        let csv = "category,amount\nFood,50\nTransport,30\nFood,20\n";
        let records = parse_csv_to_maps(csv).unwrap();
        let sums = sum_by(&records, "category", "amount");
        assert_eq!(sums.get("Food").unwrap(), &70.0);
    }

    #[test]
    fn test_detect_duplicates() {
        let csv = "date,amount,desc\n2024-01-01,100,Lunch\n2024-01-02,200,Dinner\n2024-01-01,100,Lunch\n";
        let records = parse_csv_to_maps(csv).unwrap();
        let dups = detect_duplicates(&records, &["date", "amount", "desc"]);
        assert_eq!(dups.len(), 1);
    }

    #[test]
    fn test_standard_deviation() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let std = standard_deviation(&values);
        assert!((std - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_percentile() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&values, 50.0), 3.0);
        assert_eq!(percentile(&values, 0.0), 1.0);
        assert_eq!(percentile(&values, 100.0), 5.0);
    }

    // M5: Locale-aware parsing tests
    #[test]
    fn test_parse_f64_locale_tr() {
        assert_eq!(parse_f64_locale("1.234,56"), Some(1234.56));
        assert_eq!(parse_f64_locale("1.234.567,89"), Some(1234567.89));
        assert_eq!(parse_f64_locale("0,50"), Some(0.50));
    }

    #[test]
    fn test_parse_f64_locale_en() {
        assert_eq!(parse_f64_locale("1,234.56"), Some(1234.56));
        assert_eq!(parse_f64_locale("1,234,567.89"), Some(1234567.89));
    }

    #[test]
    fn test_parse_f64_locale_plain() {
        assert_eq!(parse_f64_locale("1234.56"), Some(1234.56));
        assert_eq!(parse_f64_locale("1234"), Some(1234.0));
        assert_eq!(parse_f64_locale("  42.0  "), Some(42.0));
    }

    #[test]
    fn test_parse_f64_locale_empty() {
        assert_eq!(parse_f64_locale(""), None);
        assert_eq!(parse_f64_locale("N/A"), None);
        assert_eq!(parse_f64_locale("--"), None);
    }

    #[test]
    fn test_record_get_f64_locale() {
        let mut record = HashMap::new();
        record.insert("amount".to_string(), "1.234,56".to_string());
        assert_eq!(record_get_f64(&record, "amount"), 1234.56);
    }

    #[test]
    fn test_csv_with_locale_numbers() {
        let csv = "kategori,tutar\nYiyecek,1234.56\nUlaşım,500.00\nYiyecek,250.75\n";
        let result = parse_csv_column_f64(csv, "kategori", "tutar").unwrap();
        assert!((result.get("Yiyecek").unwrap() - 1485.31).abs() < 0.01);
        assert_eq!(result.get("Ulaşım").unwrap(), &500.0);
    }

    // M7: CSV injection tests
    #[test]
    fn test_sanitize_csv_value_injection() {
        assert_eq!(sanitize_csv_value("=SUM(A1:A10)"), "'=SUM(A1:A10)");
        assert_eq!(sanitize_csv_value("+cmd|'/C calc'!A0"), "'+cmd|'/C calc'!A0");
        assert_eq!(sanitize_csv_value("-1+2"), "'-1+2");
        assert_eq!(sanitize_csv_value("@SUM(A1)"), "'@SUM(A1)");
        assert_eq!(sanitize_csv_value("\t=cmd"), "'=cmd");
    }

    #[test]
    fn test_sanitize_csv_value_safe() {
        assert_eq!(sanitize_csv_value("hello"), "hello");
        assert_eq!(sanitize_csv_value("123"), "123");
        assert_eq!(sanitize_csv_value("2024-01-01"), "2024-01-01");
    }

    #[test]
    fn test_sanitize_csv_content() {
        let csv = "name,formula\nAcme,=SUM(A1:A10)\nBeta,normal\n";
        let sanitized = sanitize_csv_content(csv);
        assert!(sanitized.contains("'=SUM(A1:A10)"));
        assert!(sanitized.contains("normal"));
    }

    #[test]
    fn test_all_empty() {
        let records: Vec<HashMap<String, String>> = Vec::new();
        assert!(all_empty(&records));

        let non_empty = parse_csv_to_maps("a,b\n1,2\n").unwrap();
        assert!(!all_empty(&non_empty));
    }

    #[test]
    fn test_empty_response() {
        let resp = empty_response("test.agent", &["input_a", "input_b"]);
        assert_eq!(resp["status"], "no_data");
        assert_eq!(resp["agent"], "test.agent");
        assert_eq!(resp["record_count"], 0);
        assert!(resp["message"].as_str().unwrap().contains("input_a"));
    }
}
