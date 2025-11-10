//! YAML parser v2 using parse-linkml from crates/data/parsing/parse/linkml-parser/
//!
//! # Architecture
//!
//! This parser implements RootReal's **mandatory centralized parsing architecture**.
//! ALL parsing operations MUST use parsers from `crates/data/parsing/parse/`.
//!
//! ## Why parse-linkml Instead of ParseService?
//!
//! LinkML YAML schemas use `parse-linkml` (from `crates/data/parsing/parse/linkml-parser/`)
//! instead of ParseService because:
//!
//! 1. **Specialized Parser**: LinkML requires PEG parser for schema-specific grammar
//! 2. **Part of Central Infrastructure**: `parse-linkml` IS within `crates/data/parsing/parse/`
//! 3. **ParseFormat Limitation**: `ParseFormat` enum doesn't support LinkML/YAML yet
//! 4. **High Performance**: Direct PEG parsing optimized for LinkML schemas
//! 5. **Better Error Messages**: PEG parser provides precise line/column locations
//!
//! ## Centralized Parsing Compliance
//!
//! While this doesn't use ParseService directly, it DOES follow centralized parsing:
//! - Uses dedicated parser from `crates/data/parsing/parse/linkml-parser/`
//! - NOT using generic `serde_yaml` directly (which would violate architecture)
//! - Provides consistent error handling via `LinkMLError`
//! - Integrates with file system abstraction
//! - Follows same patterns as other specialized parsers
//!
//! **Compare to JSON parser**: json_parser_v2.rs uses ParseService → serde_json.
//! **This YAML parser**: Uses parse-linkml (specialized PEG parser for LinkML).
//! Both comply with mandatory centralized parsing architecture.
//!
//! File system operations are handled via the `FileSystemOperations` trait
//! for sandboxed, testable file access.

use linkml_core::{
    error::{LinkMLError, Result},
    types::SchemaDefinition,
};
use parse_linkml::LinkMLParser;
use std::path::Path;
use std::sync::Arc;

use super::SchemaParser;
use crate::file_system_adapter::FileSystemOperations;

/// `YAML` parser implementation with LinkML Parser and file system adapter
#[derive(Clone)]
pub struct YamlParserV2<F: FileSystemOperations> {
    fs: Arc<F>,
}

impl<F: FileSystemOperations> YamlParserV2<F> {
    /// Create a new `YAML` parser with file system adapter
    pub fn new(fs: Arc<F>) -> Self {
        Self { fs }
    }
}

impl<F: FileSystemOperations> SchemaParser for YamlParserV2<F> {
    fn parse_str(&self, content: &str) -> Result<SchemaDefinition> {
        // Use LinkMLParser directly for high-performance PEG parsing
        LinkMLParser::parse_schema(content).map_err(|e| {
            // Convert parse-linkml error to linkml_core error
            match e {
                parse_linkml::LinkMLError::SyntaxError { message, line, column } => {
                    LinkMLError::parse_at(message, format!("line {line}, column {column}"))
                }
                // Handle all other error variants generically
                other => LinkMLError::parse(other.to_string())
            }
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
impl<F: FileSystemOperations> AsyncSchemaParser for YamlParserV2<F> {
    async fn parse_str(&self, content: &str) -> Result<SchemaDefinition> {
        // Use LinkMLParser directly for high-performance PEG parsing
        LinkMLParser::parse_schema(content).map_err(|e| {
            // Convert parse-linkml error to linkml_core error
            match e {
                parse_linkml::LinkMLError::SyntaxError { message, line, column } => {
                    LinkMLError::parse_at(message, format!("line {line}, column {column}"))
                }
                // Handle all other error variants generically
                other => LinkMLError::parse(other.to_string())
            }
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
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_yaml_parser_v2() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let fs = Arc::new(TokioFileSystemAdapter::sandboxed(
            temp_dir.path().to_path_buf(),
        ));
        let parser = YamlParserV2::new(fs.clone());

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
        let schema = <YamlParserV2<TokioFileSystemAdapter> as AsyncSchemaParser>::parse_file(
            &parser,
            schema_path,
        )
        .await?;
        assert_eq!(schema.name, "TestSchema");
        assert!(schema.classes.contains_key("Person"));
        Ok(())
    }
}
