use crate::manifest::ToolManifest;
use crate::tool::Tool;
use agent_common::error::{AppError, AppResult};
use agent_common::types::ToolRisk;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetData {
    pub sheets: Vec<SheetData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetData {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct XlsxEngine;

impl XlsxEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_csv(&self, content: &str) -> AppResult<SpreadsheetData> {
        let mut sheets = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Ok(SpreadsheetData { sheets: vec![] });
        }

        let headers: Vec<String> = lines[0].split(',').map(|s| s.trim().to_string()).collect();
        let rows: Vec<Vec<String>> = lines[1..]
            .iter()
            .map(|l| l.split(',').map(|s| s.trim().to_string()).collect())
            .collect();

        sheets.push(SheetData {
            name: "Sheet1".to_string(),
            headers,
            rows,
        });

        Ok(SpreadsheetData { sheets })
    }

    pub fn merge(
        &self,
        data1: &SpreadsheetData,
        data2: &SpreadsheetData,
    ) -> AppResult<SpreadsheetData> {
        let mut merged_sheets = data1.sheets.clone();

        for sheet2 in &data2.sheets {
            if let Some(existing) = merged_sheets.iter_mut().find(|s| s.name == sheet2.name) {
                existing.rows.extend(sheet2.rows.clone());
            } else {
                merged_sheets.push(sheet2.clone());
            }
        }

        Ok(SpreadsheetData {
            sheets: merged_sheets,
        })
    }

    pub fn summarize(&self, data: &SpreadsheetData) -> AppResult<serde_json::Value> {
        let mut summaries = Vec::new();

        for sheet in &data.sheets {
            let numeric_cols: Vec<usize> = sheet
                .headers
                .iter()
                .enumerate()
                .filter(|(i, _h)| {
                    sheet
                        .rows
                        .iter()
                        .all(|row| row.get(*i).and_then(|v| v.parse::<f64>().ok()).is_some())
                })
                .map(|(i, _)| i)
                .collect();

            let mut col_stats = Vec::new();
            for &col_idx in &numeric_cols {
                let values: Vec<f64> = sheet
                    .rows
                    .iter()
                    .filter_map(|row| row.get(col_idx)?.parse::<f64>().ok())
                    .collect();

                if !values.is_empty() {
                    let sum: f64 = values.iter().sum();
                    let avg = sum / values.len() as f64;
                    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

                    col_stats.push(serde_json::json!({
                        "column": sheet.headers[col_idx],
                        "count": values.len(),
                        "sum": sum,
                        "avg": avg,
                        "min": min,
                        "max": max,
                    }));
                }
            }

            summaries.push(serde_json::json!({
                "sheet": sheet.name,
                "rows": sheet.rows.len(),
                "columns": sheet.headers.len(),
                "numeric_columns": col_stats,
            }));
        }

        Ok(serde_json::json!(summaries))
    }
}

impl Default for XlsxEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for XlsxEngine {
    fn manifest(&self) -> &ToolManifest {
        static MANIFEST: ToolManifest = ToolManifest {
            id: "spreadsheet.process",
            version: "1.0.0",
            name: "Spreadsheet Engine",
            description: "Process spreadsheet data (CSV/XLSX)",
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
                let data = self.parse_csv(content)?;
                Ok(serde_json::to_value(data).unwrap())
            }
            "summarize" => {
                let data = self.parse_csv(content)?;
                self.summarize(&data)
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
    fn test_spreadsheet_parse() {
        let engine = XlsxEngine::new();
        let csv = "name,salary\nAlice,50000\nBob,60000";
        let data = engine.parse_csv(csv).unwrap();
        assert_eq!(data.sheets.len(), 1);
        assert_eq!(data.sheets[0].headers, vec!["name", "salary"]);
        assert_eq!(data.sheets[0].rows.len(), 2);
    }

    #[test]
    fn test_spreadsheet_summarize() {
        let engine = XlsxEngine::new();
        let csv = "name,salary\nAlice,50000\nBob,60000";
        let data = engine.parse_csv(csv).unwrap();
        let summary = engine.summarize(&data).unwrap();
        assert!(summary.is_array());
        assert_eq!(summary.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_spreadsheet_merge() {
        let engine = XlsxEngine::new();
        let data1 = engine.parse_csv("name,age\nAlice,30").unwrap();
        let data2 = engine.parse_csv("name,age\nBob,25").unwrap();
        let merged = engine.merge(&data1, &data2).unwrap();
        assert_eq!(merged.sheets[0].rows.len(), 2);
    }
}
