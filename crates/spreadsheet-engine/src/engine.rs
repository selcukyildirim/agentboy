use agent_common::error::{AppError, AppResult};
use calamine::{open_workbook_from_rs, Reader as CalamineReader, Xlsx};
use std::io::Cursor;

pub fn read_csv(content: &[u8]) -> AppResult<crate::model::Spreadsheet> {
    let cursor = Cursor::new(content);
    let mut rdr = csv::Reader::from_reader(cursor);

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| AppError::Validation(format!("CSV header error: {}", e)))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| AppError::Validation(format!("CSV record error: {}", e)))?;
        let row: Vec<String> = record.iter().map(|field| field.to_string()).collect();
        rows.push(row);
    }

    Ok(crate::model::Spreadsheet {
        sheets: vec![crate::model::Sheet {
            name: "Sheet1".to_string(),
            headers,
            rows,
        }],
    })
}

pub fn read_csv_with_delimiter(content: &[u8], delimiter: u8) -> AppResult<crate::model::Spreadsheet> {
    let cursor = Cursor::new(content);
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_reader(cursor);

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| AppError::Validation(format!("CSV header error: {}", e)))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| AppError::Validation(format!("CSV record error: {}", e)))?;
        let row: Vec<String> = record.iter().map(|field| field.to_string()).collect();
        rows.push(row);
    }

    Ok(crate::model::Spreadsheet {
        sheets: vec![crate::model::Sheet {
            name: "Sheet1".to_string(),
            headers,
            rows,
        }],
    })
}

pub fn read_xlsx(content: &[u8]) -> AppResult<crate::model::Spreadsheet> {
    let mut workbook: Xlsx<Cursor<&[u8]>> =
        open_workbook_from_rs(Cursor::new(content))
            .map_err(|e| AppError::Validation(format!("XLSX open error: {}", e)))?;

    let sheet_names = workbook.sheet_names().to_vec();
    let mut sheets = Vec::new();

    for name in &sheet_names {
        let range = workbook
            .worksheet_range(name)
            .map_err(|e| AppError::Validation(format!("XLSX sheet error: {}", e)))?;

        let mut headers = Vec::new();
        let mut rows = Vec::new();

        for (i, row) in range.rows().enumerate() {
            let values: Vec<String> = row
                .iter()
                .map(|cell| match cell {
                    calamine::Data::Empty => String::new(),
                    calamine::Data::String(s) => s.clone(),
                    calamine::Data::Float(f) => {
                        if *f == (*f as i64) as f64 {
                            format!("{}", *f as i64)
                        } else {
                            format!("{}", f)
                        }
                    }
                    calamine::Data::Int(n) => format!("{}", n),
                    calamine::Data::Bool(b) => format!("{}", b),
                    calamine::Data::Error(e) => format!("ERROR:{:?}", e),
                    calamine::Data::DateTime(dt) => format!("{}", dt),
                    calamine::Data::DateTimeIso(s) => s.clone(),
                    calamine::Data::DurationIso(s) => s.clone(),
                })
                .collect();

            if i == 0 {
                headers = values;
            } else {
                rows.push(values);
            }
        }

        sheets.push(crate::model::Sheet {
            name: name.clone(),
            headers,
            rows,
        });
    }

    Ok(crate::model::Spreadsheet { sheets })
}

pub fn to_json(spreadsheet: &crate::model::Spreadsheet) -> serde_json::Value {
    let sheets: Vec<serde_json::Value> = spreadsheet
        .sheets
        .iter()
        .map(|sheet| {
            let records: Vec<serde_json::Value> = sheet
                .rows
                .iter()
                .map(|row| {
                    let mut obj = serde_json::Map::new();
                    for (i, header) in sheet.headers.iter().enumerate() {
                        let value = row.get(i).map(|s| s.as_str()).unwrap_or("");
                        obj.insert(
                            header.clone(),
                            serde_json::Value::String(value.to_string()),
                        );
                    }
                    serde_json::Value::Object(obj)
                })
                .collect();

            serde_json::json!({
                "name": sheet.name,
                "headers": sheet.headers,
                "rows": records,
                "row_count": sheet.rows.len()
            })
        })
        .collect();

    serde_json::json!({
        "sheets": sheets,
        "sheet_count": spreadsheet.sheets.len()
    })
}

pub fn summarize(spreadsheet: &crate::model::Spreadsheet) -> serde_json::Value {
    let sheets_summary: Vec<serde_json::Value> = spreadsheet
        .sheets
        .iter()
        .map(|sheet| {
            let numeric_columns: Vec<(usize, String)> = sheet
                .headers
                .iter()
                .enumerate()
                .filter_map(|(i, h)| {
                    if sheet.rows.iter().any(|row| {
                        row.get(i)
                            .and_then(|v| v.parse::<f64>().ok())
                            .is_some()
                    }) {
                        Some((i, h.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            let mut column_stats = serde_json::Map::new();
            for (col_idx, col_name) in &numeric_columns {
                let values: Vec<f64> = sheet
                    .rows
                    .iter()
                    .filter_map(|row| {
                        row.get(*col_idx)
                            .and_then(|v| v.parse::<f64>().ok())
                    })
                    .collect();

                if !values.is_empty() {
                    let sum: f64 = values.iter().sum();
                    let count = values.len() as f64;
                    let mean = sum / count;
                    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

                    column_stats.insert(
                        col_name.clone(),
                        serde_json::json!({
                            "sum": sum,
                            "mean": mean,
                            "min": min,
                            "max": max,
                            "count": count as u64,
                        }),
                    );
                }
            }

            serde_json::json!({
                "name": sheet.name,
                "row_count": sheet.rows.len(),
                "column_count": sheet.headers.len(),
                "columns": sheet.headers,
                "numeric_stats": column_stats,
            })
        })
        .collect();

    serde_json::json!({
        "sheets": sheets_summary,
        "total_sheets": spreadsheet.sheets.len(),
    })
}

pub fn merge(spreadsheets: &[crate::model::Spreadsheet]) -> AppResult<crate::model::Spreadsheet> {
    if spreadsheets.is_empty() {
        return Err(AppError::Validation(
            "No spreadsheets to merge".to_string(),
        ));
    }

    let mut all_sheets = Vec::new();
    for ss in spreadsheets {
        all_sheets.extend(ss.sheets.clone());
    }

    Ok(crate::model::Spreadsheet { sheets: all_sheets })
}

pub fn filter_rows(
    spreadsheet: &crate::model::Spreadsheet,
    column: &str,
    predicate: &dyn Fn(&str) -> bool,
) -> AppResult<crate::model::Spreadsheet> {
    let mut filtered_sheets = Vec::new();

    for sheet in &spreadsheet.sheets {
        let col_idx = sheet
            .headers
            .iter()
            .position(|h| h == column)
            .ok_or_else(|| {
                AppError::Validation(format!("Column '{}' not found", column))
            })?;

        let filtered_rows: Vec<Vec<String>> = sheet
            .rows
            .iter()
            .filter(|row| {
                row.get(col_idx)
                    .map(|v| predicate(v))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        filtered_sheets.push(crate::model::Sheet {
            name: sheet.name.clone(),
            headers: sheet.headers.clone(),
            rows: filtered_rows,
        });
    }

    Ok(crate::model::Spreadsheet {
        sheets: filtered_sheets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_csv() {
        let csv_content = b"name,amount,category\nAcme,100,Office\nBeta,250,Travel\n";
        let ss = read_csv(csv_content).unwrap();
        assert_eq!(ss.sheets.len(), 1);
        assert_eq!(ss.sheets[0].headers, vec!["name", "amount", "category"]);
        assert_eq!(ss.sheets[0].rows.len(), 2);
        assert_eq!(ss.sheets[0].rows[0][0], "Acme");
        assert_eq!(ss.sheets[0].rows[0][1], "100");
    }

    #[test]
    fn test_read_csv_delimiter() {
        let csv_content = b"name;amount;category\nAcme;100;Office\n";
        let ss = read_csv_with_delimiter(csv_content, b';').unwrap();
        assert_eq!(ss.sheets[0].headers, vec!["name", "amount", "category"]);
        assert_eq!(ss.sheets[0].rows.len(), 1);
    }

    #[test]
    fn test_read_csv_quoted_fields() {
        let csv_content = b"name,description\nAcme,\"Office supplies, furniture\"\n";
        let ss = read_csv(csv_content).unwrap();
        assert_eq!(ss.sheets[0].rows[0][1], "Office supplies, furniture");
    }

    #[test]
    fn test_to_json() {
        let csv_content = b"name,amount\nAcme,100\n";
        let ss = read_csv(csv_content).unwrap();
        let json = to_json(&ss);
        assert!(json["sheets"].is_array());
        assert_eq!(json["sheets"][0]["name"], "Sheet1");
    }

    #[test]
    fn test_summarize() {
        let csv_content = b"item,amount,quantity\nA,100,10\nB,200,5\n";
        let ss = read_csv(csv_content).unwrap();
        let summary = summarize(&ss);
        assert!(summary["sheets"][0]["numeric_stats"].is_object());
        let stats = &summary["sheets"][0]["numeric_stats"];
        assert!(stats.get("amount").is_some());
        assert!(stats.get("quantity").is_some());
    }

    #[test]
    fn test_filter_rows() {
        let csv_content = b"name,category\nAcme,Office\nBeta,Travel\nGamma,Office\n";
        let ss = read_csv(csv_content).unwrap();
        let filtered = filter_rows(&ss, "category", &|v| v == "Office").unwrap();
        assert_eq!(filtered.sheets[0].rows.len(), 2);
        assert_eq!(filtered.sheets[0].rows[0][0], "Acme");
        assert_eq!(filtered.sheets[0].rows[1][0], "Gamma");
    }

    #[test]
    fn test_filter_rows_column_not_found() {
        let csv_content = b"name,amount\nAcme,100\n";
        let ss = read_csv(csv_content).unwrap();
        let result = filter_rows(&ss, "nonexistent", &|_| true);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge() {
        let csv1 = b"name,amount\nAcme,100\n";
        let csv2 = b"item,quantity\nWidget,5\n";
        let ss1 = read_csv(csv1).unwrap();
        let ss2 = read_csv(csv2).unwrap();
        let merged = merge(&[ss1, ss2]).unwrap();
        assert_eq!(merged.sheets.len(), 2);
    }

    #[test]
    fn test_merge_empty() {
        let result = merge(&[]);
        assert!(result.is_err());
    }
}