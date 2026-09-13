use async_trait::async_trait;
use agent_common::error::{AppError, AppResult};
use agent_common::types::ToolRisk;
use crate::tool::Tool;
use crate::manifest::ToolManifest;

pub struct CsvEngine;

impl CsvEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, content: &str) -> AppResult<Vec<Vec<String>>> {
        let mut rows = Vec::new();
        for line in content.lines() {
            let row: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            rows.push(row);
        }
        Ok(rows)
    }

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
                    .zip(row.iter())
                    .map(|(h, v)| (h.clone(), serde_json::Value::String(v.clone())))
                    .collect();
                serde_json::Value::Object(obj)
            })
            .collect();

        Ok(serde_json::json!(data))
    }

    pub fn filter(&self, content: &str, column: &str, value: &str) -> AppResult<String> {
        let rows = self.parse(content)?;
        if rows.is_empty() {
            return Ok(String::new());
        }

        let headers = &rows[0];
        let col_idx = headers
            .iter()
            .position(|h| h == column)
            .ok_or_else(|| AppError::Validation(format!("Column '{}' not found", column)))?;

        let mut result = vec![headers.join(",")];
        for row in &rows[1..] {
            if row.get(col_idx).map(|v| v == value).unwrap_or(false) {
                result.push(row.join(","));
            }
        }

        Ok(result.join("\n"))
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
            version: "1.0.0",
            name: "CSV Parser",
            description: "Parse and transform CSV data",
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
                Ok(serde_json::json!({"rows": rows, "count": rows.len()}))
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
                Ok(serde_json::json!({"content": filtered}))
            }
            _ => Err(AppError::Validation(format!(
                "Unknown operation: {}",
                operation
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parse() {
        let engine = CsvEngine::new();
        let csv = "name,age\nAlice,30\nBob,25";
        let rows = engine.parse(csv).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], vec!["name", "age"]);
    }

    #[test]
    fn test_csv_to_json() {
        let engine = CsvEngine::new();
        let csv = "name,age\nAlice,30\nBob,25";
        let json = engine.to_json(csv).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_csv_filter() {
        let engine = CsvEngine::new();
        let csv = "name,age\nAlice,30\nBob,25";
        let filtered = engine.filter(csv, "age", "30").unwrap();
        assert!(filtered.contains("Alice"));
        assert!(!filtered.contains("Bob"));
    }
}