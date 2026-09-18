use crate::manifest::ToolManifest;
use crate::tool::Tool;
use agent_common::error::{AppError, AppResult};
use agent_common::types::ToolRisk;
use async_trait::async_trait;

pub struct JsonTransform;

impl JsonTransform {
    pub fn new() -> Self {
        Self
    }

    pub fn flatten(&self, input: &serde_json::Value, prefix: &str) -> AppResult<serde_json::Value> {
        let mut result = serde_json::Map::new();
        self.flatten_inner(input, prefix, &mut result);
        Ok(serde_json::Value::Object(result))
    }

    fn flatten_inner(
        &self,
        value: &serde_json::Value,
        prefix: &str,
        result: &mut serde_json::Map<String, serde_json::Value>,
    ) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, val) in map {
                    let new_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    self.flatten_inner(val, &new_key, result);
                }
            }
            serde_json::Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let new_key = format!("{}[{}]", prefix, i);
                    self.flatten_inner(val, &new_key, result);
                }
            }
            _ => {
                result.insert(prefix.to_string(), value.clone());
            }
        }
    }

    pub fn filter_keys(
        &self,
        input: &serde_json::Value,
        keys: &[String],
    ) -> AppResult<serde_json::Value> {
        match input {
            serde_json::Value::Object(map) => {
                let filtered: serde_json::Map<String, serde_json::Value> = map
                    .iter()
                    .filter(|(k, _)| keys.contains(k))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                Ok(serde_json::Value::Object(filtered))
            }
            _ => Ok(input.clone()),
        }
    }

    pub fn transform(
        &self,
        input: &serde_json::Value,
        mapping: &serde_json::Value,
    ) -> AppResult<serde_json::Value> {
        match mapping {
            serde_json::Value::Object(map) => {
                let mut result = serde_json::Map::new();
                for (key, path) in map {
                    if let Some(val_path) = path.as_str() {
                        let value = self.resolve_path(input, val_path);
                        result.insert(key.clone(), value);
                    } else {
                        result.insert(key.clone(), path.clone());
                    }
                }
                Ok(serde_json::Value::Object(result))
            }
            _ => Ok(input.clone()),
        }
    }

    fn resolve_path(&self, input: &serde_json::Value, path: &str) -> serde_json::Value {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = input;

        for part in parts {
            match current {
                serde_json::Value::Object(map) => {
                    current = map.get(part).unwrap_or(&serde_json::Value::Null);
                }
                _ => return serde_json::Value::Null,
            }
        }

        current.clone()
    }
}

impl Default for JsonTransform {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for JsonTransform {
    fn manifest(&self) -> &ToolManifest {
        static MANIFEST: ToolManifest = ToolManifest {
            id: "json.transform",
            version: "1.0.0",
            name: "JSON Transform",
            description: "Transform and manipulate JSON data",
            risk: ToolRisk::Read,
            input_schema: None,
            output_schema: None,
        };
        &MANIFEST
    }

    async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value> {
        let data = &input["data"];
        let operation = input["operation"].as_str().unwrap_or("flatten");

        match operation {
            "flatten" => self.flatten(data, ""),
            "filter_keys" => {
                let keys: Vec<String> = input["keys"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                self.filter_keys(data, &keys)
            }
            "transform" => self.transform(data, &input["mapping"]),
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
    fn test_flatten() {
        let transform = JsonTransform::new();
        let input = serde_json::json!({"a": {"b": 1, "c": 2}});
        let flat = transform.flatten(&input, "").unwrap();
        assert_eq!(flat["a.b"], 1);
        assert_eq!(flat["a.c"], 2);
    }

    #[test]
    fn test_filter_keys() {
        let transform = JsonTransform::new();
        let input = serde_json::json!({"a": 1, "b": 2, "c": 3});
        let filtered = transform
            .filter_keys(&input, &["a".to_string(), "c".to_string()])
            .unwrap();
        assert!(filtered["a"].is_number());
        assert!(filtered["c"].is_number());
        assert!(filtered.get("b").is_none());
    }

    #[test]
    fn test_transform() {
        let transform = JsonTransform::new();
        let input = serde_json::json!({"user": {"name": "Alice", "age": 30}});
        let mapping = serde_json::json!({"person_name": "user.name", "person_age": "user.age"});
        let result = transform.transform(&input, &mapping).unwrap();
        assert_eq!(result["person_name"], "Alice");
        assert_eq!(result["person_age"], 30);
    }
}
