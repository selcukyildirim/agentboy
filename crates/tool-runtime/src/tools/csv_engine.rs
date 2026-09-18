use agent_common::error::{AppError, AppResult};
use agent_common::types::ToolRisk;
use async_trait::async_trait;

use crate::manifest::ToolManifest;
use crate::tool::Tool;

/// Robust CSV engine backed by the `csv` crate (RFC 4180: quoted fields,
/// embedded commas/newlines, escaped quotes). Single source of truth for CSV
/// parsing across the workspace.
pub struct CsvEngine;

impl CsvEngine {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Parse CSV into rows. Row 0 is the header.
    pub fn parse(&self, content: &str) -> AppResult<Vec<Vec<String>>> {
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(content.as_bytes());

        let mut rows: Vec<Vec<String>> = Vec::new();

        let headers = rdr
            .headers()
            .map_err(|e| AppError::Validation(format!("CSV header error: {e}")))?;
        rows.push(
            headers
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        );

        for record in rdr.records() {
            let record =
                record.map_err(|e| AppError::Validation(format!("CSV record error: {e}")))?;
            rows.push(
                record
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
            );
        }

        Ok(rows)
    }

    /// Parse CSV into an array of JSON objects keyed by header.
    pub fn to_json(&self, content: &str) -> AppResult<serde_json::Value> {
        let rows = self.parse(content)?;
        if rows.is_empty() {
            return Ok(serde_json::json!([]));
        }
        let headers = &rows[0];
        let data: Vec<serde_json::Value> = rows[1..]
            .iter()
            .map(|row| {
                let obj: serde_json::Map<String, serde_json::Value> = headers
                    .iter()
                    .enumerate()
                    .map(|(i, h)| {
                        let v = row.get(i).cloned().unwrap_or_default();
                        (h.clone(), serde_json::Value::String(v))
                    })
                    .collect();
                serde_json::Value::Object(obj)
            })
            .collect();
        Ok(serde_json::json!(data))
    }

    /// Filter rows where `column == value`, preserving the header.
    pub fn filter(&self, content: &str, column: &str, value: &str) -> AppResult<String> {
        let rows = self.parse(content)?;
        if rows.is_empty() {
            return Ok(String::new());
        }
        let headers = &rows[0];
        let col_idx = headers
            .iter()
            .position(|h| h == column)
            .ok_or_else(|| AppError::Validation(format!("Column '{column}' not found")))?;

        let mut out = csv::WriterBuilder::new().from_writer(vec![]);
        out.write_record(headers)
            .map_err(|e| AppError::Validation(format!("CSV write error: {e}")))?;
        for row in &rows[1..] {
            if row.get(col_idx).is_some_and(|v| v == value) {
                out.write_record(row)
                    .map_err(|e| AppError::Validation(format!("CSV write error: {e}")))?;
            }
        }
        let bytes = out
            .into_inner()
            .map_err(|e| AppError::Validation(format!("CSV flush error: {e}")))?;
        String::from_utf8(bytes).map_err(|e| AppError::Validation(format!("CSV utf8 error: {e}")))
    }
}

impl Default for CsvEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CsvEngine {
    fn manifest(&self) -> &ToolManifest {
        static MANIFEST: ToolManifest = ToolManifest {
            id: "csv.parse",
            version: "2.0.0",
            name: "CSV Parser",
            description: "Parse and transform CSV data (RFC 4180)",
            risk: ToolRisk::Read,
            input_schema: None,
            output_schema: None,
        };
        &MANIFEST
    }

    async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value> {
        let content = input["content"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'content' field".to_string()))?;
        let operation = input["operation"].as_str().unwrap_or("parse");

        match operation {
            "parse" => {
                let rows = self.parse(content)?;
                Ok(serde_json::json!({ "rows": rows, "count": rows.len() }))
            }
            "to_json" => self.to_json(content),
            "filter" => {
                let column = input["column"]
                    .as_str()
                    .ok_or_else(|| AppError::Validation("Missing 'column' field".to_string()))?;
                let value = input["value"]
                    .as_str()
                    .ok_or_else(|| AppError::Validation("Missing 'value' field".to_string()))?;
                let filtered = self.filter(content, column, value)?;
                Ok(serde_json::json!({ "content": filtered }))
            }
            _ => Err(AppError::Validation(format!(
                "Unknown operation: {operation}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parse() {
        let rows = CsvEngine::new()
            .parse("name,age\nAlice,30\nBob,25")
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], vec!["name", "age"]);
        assert_eq!(rows[1], vec!["Alice", "30"]);
    }

    #[test]
    fn test_quoted_fields_with_commas() {
        let rows = CsvEngine::new()
            .parse("name,note\n\"Acme, Inc.\",\"hello, world\"")
            .unwrap();
        assert_eq!(rows[1], vec!["Acme, Inc.", "hello, world"]);
    }

    #[test]
    fn test_csv_to_json() {
        let json = CsvEngine::new()
            .to_json("name,age\nAlice,30\nBob,25")
            .unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
        assert_eq!(json[0]["name"], "Alice");
    }

    #[test]
    fn test_csv_filter() {
        let filtered = CsvEngine::new()
            .filter("name,age\nAlice,30\nBob,25", "age", "30")
            .unwrap();
        assert!(filtered.contains("Alice"));
        assert!(!filtered.contains("Bob"));
    }
}
