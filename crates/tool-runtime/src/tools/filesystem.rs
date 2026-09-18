use crate::manifest::ToolManifest;
use crate::tool::Tool;
use agent_common::error::{AppError, AppResult};
use agent_common::types::ToolRisk;
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FilesystemReadTool {
    allowed_dirs: Vec<PathBuf>,
}

impl FilesystemReadTool {
    #[must_use]
    pub const fn new(allowed_dirs: Vec<PathBuf>) -> Self {
        Self { allowed_dirs }
    }

    fn validate_path(&self, path: &str) -> AppResult<PathBuf> {
        let canonical =
            Path::new(path)
                .canonicalize()
                .map_err(|e| AppError::ToolPermissionDenied {
                    tool: "filesystem.read".to_string(),
                    reason: e.to_string(),
                })?;

        for dir in &self.allowed_dirs {
            if let Ok(dir_canonical) = dir.canonicalize() {
                if canonical.starts_with(&dir_canonical) {
                    return Ok(canonical);
                }
            }
        }

        Err(AppError::ToolPermissionDenied {
            tool: "filesystem.read".to_string(),
            reason: format!("Path '{path}' not in allowed directories"),
        })
    }
}

#[async_trait]
impl Tool for FilesystemReadTool {
    fn manifest(&self) -> &ToolManifest {
        static MANIFEST: ToolManifest = ToolManifest {
            id: "filesystem.read",
            version: "1.0.0",
            name: "Filesystem Read",
            description: "Read files from allowed directories",
            risk: ToolRisk::Read,
            input_schema: None,
            output_schema: None,
        };
        &MANIFEST
    }

    async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'path' field".to_string()))?;

        let canonical = self.validate_path(path)?;

        let content =
            fs::read_to_string(&canonical).map_err(|e| AppError::ToolExecutionFailed {
                tool: "filesystem.read".to_string(),
                reason: e.to_string(),
            })?;

        Ok(serde_json::json!({
            "path": canonical.to_string_lossy(),
            "content": content,
            "size": content.len(),
        }))
    }
}

pub struct FilesystemWriteTool {
    allowed_dirs: Vec<PathBuf>,
}

impl FilesystemWriteTool {
    #[must_use]
    pub const fn new(allowed_dirs: Vec<PathBuf>) -> Self {
        Self { allowed_dirs }
    }

    fn validate_path(&self, path: &str) -> AppResult<PathBuf> {
        let path_obj = Path::new(path);

        if let Some(parent) = path_obj.parent() {
            if let Ok(parent_canonical) = parent.canonicalize() {
                for dir in &self.allowed_dirs {
                    if let Ok(dir_canonical) = dir.canonicalize() {
                        if parent_canonical.starts_with(&dir_canonical) {
                            return path_obj.canonicalize().or_else(|_| {
                                Ok(parent_canonical.join(path_obj.file_name().unwrap_or_default()))
                            });
                        }
                    }
                }
            }
        }

        Err(AppError::ToolPermissionDenied {
            tool: "filesystem.write".to_string(),
            reason: format!("Path '{path}' not in allowed directories"),
        })
    }
}

#[async_trait]
impl Tool for FilesystemWriteTool {
    fn manifest(&self) -> &ToolManifest {
        static MANIFEST: ToolManifest = ToolManifest {
            id: "filesystem.write",
            version: "1.0.0",
            name: "Filesystem Write",
            description: "Write files to allowed directories",
            risk: ToolRisk::Write,
            input_schema: None,
            output_schema: None,
        };
        &MANIFEST
    }

    async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'path' field".to_string()))?;
        let content = input["content"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'content' field".to_string()))?;

        let canonical = self.validate_path(path)?;

        if let Some(parent) = canonical.parent() {
            fs::create_dir_all(parent).map_err(|e| AppError::ToolExecutionFailed {
                tool: "filesystem.write".to_string(),
                reason: e.to_string(),
            })?;
        }

        fs::write(&canonical, content).map_err(|e| AppError::ToolExecutionFailed {
            tool: "filesystem.write".to_string(),
            reason: e.to_string(),
        })?;

        Ok(serde_json::json!({
            "path": canonical.to_string_lossy(),
            "bytes_written": content.len(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_path_traversal_blocked() {
        let dir = env::temp_dir();
        let tool = FilesystemReadTool::new(vec![dir]);

        let result = tool.validate_path("/etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_path_allowed() {
        let dir = env::temp_dir();
        let tool = FilesystemReadTool::new(vec![dir.clone()]);

        let test_file = dir.join("test_allowed.txt");
        fs::write(&test_file, "test").ok();

        let result = tool.validate_path(test_file.to_str().unwrap());
        assert!(result.is_ok());

        fs::remove_file(&test_file).ok();
    }
}
