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
            .and_then(|v| v.parse::<f64>().ok())
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
        .and_then(|v| v.parse::<f64>().ok())
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
    let mut duplicates = Vec::new();
    for (i, r1) in records.iter().enumerate() {
        for (j, r2) in records.iter().enumerate().skip(i + 1) {
            let all_match = keys.iter().all(|k| record_get_str(r1, k) == record_get_str(r2, k));
            if all_match && keys.iter().all(|k| !record_get_str(r1, k).is_empty()) {
                duplicates.push((i, j));
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
}