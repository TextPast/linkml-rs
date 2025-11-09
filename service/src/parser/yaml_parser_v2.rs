//! YAML parser v2 using ParseService and file system adapter
//!
//! This version uses RootReal's `ParseService` for all parsing operations,
//! following the centralized parsing architecture. File system operations
//! are handled via the `FileSystemOperations` trait.

use linkml_core::{
    error::{LinkMLError, Result},
    types::SchemaDefinition,
};
use parse_core::{ParseService, ParseFormat};
use std::path::Path;
use std::sync::Arc;

use super::SchemaParser;
use crate::file_system_adapter::FileSystemOperations;

/// `YAML` parser implementation with ParseService and file system adapter
pub struct YamlParserV2<P: ParseService, F: FileSystemOperations> {
    parse_service: Arc<P>,
    fs: Arc<F>,
}

impl<P: ParseService, F: FileSystemOperations> YamlParserV2<P, F> {
    /// Create a new `YAML` parser with ParseService and file system adapter
    pub fn new(parse_service: Arc<P>, fs: Arc<F>) -> Self {
        Self { parse_service, fs }
    }
}

impl<P: ParseService, F: FileSystemOperations> SchemaParser for YamlParserV2<P, F> {
    fn parse_str(&self, content: &str) -> Result<SchemaDefinition> {
        // Use ParseService, then deserialize from raw_content
        let parsed_doc = tokio::runtime::Handle::current()
            .block_on(self.parse_service.parse_with_format(content, ParseFormat::Yaml))
            .map_err(|e| LinkMLError::parse(format!("Parse service error: {e}")))?;

        serde_yaml::from_str(&parsed_doc.raw_content).map_err(|e| {
            LinkMLError::parse_at(
                format!("YAML deserialization error: {e}"),
                e.location().map_or_else(
                    || "unknown location".to_string(),
                    |l| format!("line {}, column {}", l.line(), l.column()),
                ),
            )
        })
    }

    fn parse_file(&self, path: &Path) -> Result<SchemaDefinition> {
        // Note: This is a sync trait method, but we need to use async fs operations
        // In a real implementation, we'd need to refactor the trait to be async
        // For now, we'll use tokio's block_on, but this should be addressed
        let content = tokio::runtime::Handle::current().block_on(self.fs.read_to_string(path))?;

        <Self as SchemaParser>::parse_str(self, &content).map_err(|e| match e {
            LinkMLError::ParseError { message, location } => LinkMLError::ParseError {
                message: format!("{message} in file {}", path.display()),
                location,
            },
            other => other,
        })
    }
}

/// Async version of the `SchemaParser` trait
#[async_trait::async_trait]
pub trait AsyncSchemaParser: Send + Sync {
    /// Parse schema from string content
    async fn parse_str(&self, content: &str) -> Result<SchemaDefinition>;

    /// Parse schema from file
    async fn parse_file(&self, path: &Path) -> Result<SchemaDefinition>;
}

#[async_trait::async_trait]
impl<P: ParseService, F: FileSystemOperations> AsyncSchemaParser for YamlParserV2<P, F> {
    async fn parse_str(&self, content: &str) -> Result<SchemaDefinition> {
        // Use ParseService, then deserialize from raw_content
        let parsed_doc = self
            .parse_service
            .parse_with_format(content, ParseFormat::Yaml)
            .await
            .map_err(|e| LinkMLError::parse(format!("Parse service error: {e}")))?;

        serde_yaml::from_str(&parsed_doc.raw_content).map_err(|e| {
            LinkMLError::parse_at(
                format!("YAML deserialization error: {e}"),
                e.location().map_or_else(
                    || "unknown location".to_string(),
                    |l| format!("line {}, column {}", l.line(), l.column()),
                ),
            )
        })
    }

    async fn parse_file(&self, path: &Path) -> Result<SchemaDefinition> {
        let content = self.fs.read_to_string(path).await?;

        <Self as AsyncSchemaParser>::parse_str(self, &content)
            .await
            .map_err(|e| match e {
                LinkMLError::ParseError { message, location } => LinkMLError::ParseError {
                    message: format!("{message} in file {}", path.display()),
                    location,
                },
                other => other,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_system_adapter::TokioFileSystemAdapter;
    use parse_core::{ParsedDocument, ParseError};
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
            _format: ParseFormat,
        ) -> std::result::Result<ParsedDocument, Self::Error> {
            Ok(ParsedDocument {
                raw_content: content.to_string(),
                format: ParseFormat::Yaml,
                metadata: Default::default(),
            })
        }
    }

    #[tokio::test]
    async fn test_yaml_parser_v2() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let fs = Arc::new(TokioFileSystemAdapter::sandboxed(
            temp_dir.path().to_path_buf(),
        ));
        let parse_service = Arc::new(MockParseService);
        let parser = YamlParserV2::new(parse_service, fs.clone());

        // Create a test schema
        let schema_content = r"
id: https://example.org/test
name: TestSchema
description: A test schema
classes:
  Person:
    name: Person
    description: A person
    attributes:
      name:
        name: name
        range: string
        required: true
      age:
        name: age
        range: integer
";

        // Write to file
        let schema_path = Path::new("test_schema.yaml");
        fs.write(schema_path, schema_content).await?;

        // Parse using async trait - explicitly use AsyncSchemaParser trait
        let schema = <YamlParserV2<MockParseService, TokioFileSystemAdapter> as AsyncSchemaParser>::parse_file(
            &parser,
            schema_path,
        )
        .await?;
        assert_eq!(schema.name, "TestSchema");
        assert!(schema.classes.contains_key("Person"));
        Ok(())
    }
}
