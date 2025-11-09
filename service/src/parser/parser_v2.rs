//! Main parser v2 with ParseService and file system adapter support
//!
//! This version uses RootReal's ParseService for all parsing operations
//! and FileSystemOperations for file access, following centralized architecture.

use linkml_core::{
    error::{LinkMLError, Result},
    types::SchemaDefinition};
use parse_core::ParseService;
use std::path::Path;
use std::sync::Arc;

use crate::file_system_adapter::FileSystemOperations;
use super::{AsyncSchemaParser, YamlParserV2, JsonParserV2};

/// Main parser that uses ParseService and file system adapter, delegating to format-specific parsers
pub struct ParserV2<P: ParseService, F: FileSystemOperations> {
    yaml_parser: YamlParserV2<P, F>,
    json_parser: JsonParserV2<P, F>,
    /// Whether to automatically resolve imports
    auto_resolve_imports: bool}

impl<P: ParseService, F: FileSystemOperations> ParserV2<P, F> {
    /// Create a new parser with ParseService and file system adapter
    pub fn new(parse_service: Arc<P>, fs: Arc<F>) -> Self {
        Self {
            yaml_parser: YamlParserV2::new(parse_service.clone(), fs.clone()),
            json_parser: JsonParserV2::new(parse_service, fs),
            auto_resolve_imports: true}
    }

    /// Set whether to automatically resolve imports
    pub fn with_auto_resolve_imports(mut self, auto_resolve: bool) -> Self {
        self.auto_resolve_imports = auto_resolve;
        self
    }

    /// Parse schema from string with explicit format
    /// Returns an error if the operation fails
    ///
    /// # Errors
    ///
    pub async fn parse_str(&self, content: &str, format: &str) -> Result<SchemaDefinition> {
        match format.to_lowercase().as_str() {
            "yaml" | "yml" => self.yaml_parser.parse_str(content).await,
            "json" => self.json_parser.parse_str(content).await,
            _ => Err(LinkMLError::invalid_format(format))}
    }

    /// Parse schema from file, detecting format from extension
    /// Returns an error if the operation fails
    ///
    /// # Errors
    ///
    pub async fn parse_file(&self, path: &Path) -> Result<SchemaDefinition> {
        let format = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| LinkMLError::invalid_format("no file extension"))?;

        match format.to_lowercase().as_str() {
            "yaml" | "yml" => self.yaml_parser.parse_file(path).await,
            "json" => self.json_parser.parse_file(path).await,
            _ => Err(LinkMLError::invalid_format(format))}
    }

    /// Parse with explicit format
    /// Returns an error if the operation fails
    ///
    /// # Errors
    ///
    pub async fn parse_with_format(
        &self,
        content: &str,
        format: Option<&str>
    ) -> Result<SchemaDefinition> {
        let fmt = format.unwrap_or("yaml");
        self.parse_str(content, fmt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_system_adapter::TokioFileSystemAdapter;
    use parse_core::{ParsedDocument, ParseError, ParseFormat};
    use tempfile::TempDir;

    // Mock ParseService for testing
    struct MockParseService;

    #[async_trait::async_trait]
    impl ParseService for MockParseService {
        type Error = ParseError;

        async fn parse(&self, content: &str) -> std::result::Result<ParsedDocument, Self::Error> {
            Ok(ParsedDocument {
                raw_content: content.to_string(),
                format: ParseFormat::Yaml,
                metadata: Default::default(),
            })
        }

        async fn parse_with_format(
            &self,
            content: &str,
            format: ParseFormat,
        ) -> std::result::Result<ParsedDocument, Self::Error> {
            Ok(ParsedDocument {
                raw_content: content.to_string(),
                format,
                metadata: Default::default(),
            })
        }
    }

    #[tokio::test]
    async fn test_parser_v2_yaml() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let fs = Arc::new(TokioFileSystemAdapter::sandboxed(temp_dir.path().to_path_buf()));
        let parse_service = Arc::new(MockParseService);
        let parser = ParserV2::new(parse_service, fs.clone());

        let schema_content = r#"
id: https://example.org/test
name: TestSchema
classes:
  Person:
    attributes:
      name:
        range: string
"#;

        // Test parse_str
        let schema = parser.parse_str(schema_content, "yaml").await?;
        assert_eq!(schema.name, "TestSchema");

        // Test parse_file
        let schema_path = temp_dir.path().join("test.yaml");
        fs.write(&schema_path, schema_content).await?;
        let schema = parser.parse_file(&schema_path).await?;
        assert_eq!(schema.name, "TestSchema");
        Ok(())
    }

    #[tokio::test]
    async fn test_parser_v2_json() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let fs = Arc::new(TokioFileSystemAdapter::sandboxed(temp_dir.path().to_path_buf()));
        let parse_service = Arc::new(MockParseService);
        let parser = ParserV2::new(parse_service, fs.clone());

        let schema_content = r#"{
  "id": "https://example.org/test",
  "name": "TestSchema",
  "classes": {
    "Person": {
      "attributes": {
        "name": {
          "range": "string"
        }
      }
    }
  }
}"#;

        // Test parse_str
        let schema = parser.parse_str(schema_content, "json").await?;
        assert_eq!(schema.name, "TestSchema");

        // Test parse_file
        let schema_path = temp_dir.path().join("test.json");
        fs.write(&schema_path, schema_content).await?;
        let schema = parser.parse_file(&schema_path).await?;
        assert_eq!(schema.name, "TestSchema");
        Ok(())
    }
}