//! JSON parser v2 using ParseService and file system adapter
//!
//! # Architecture
//!
//! This parser implements RootReal's **mandatory centralized parsing architecture**.
//! All JSON parsing operations MUST use `ParseService` from `crates/data/parsing/parse/`.
//!
//! ## Why Centralized Parsing?
//!
//! Using ParseService instead of direct `serde_json` provides critical benefits:
//!
//! 1. **Consistent Error Handling**: Unified error types and recovery strategies
//! 2. **Telemetry Integration**: All parsing operations are monitored and logged
//! 3. **Caching**: Frequently accessed schemas are cached for performance
//! 4. **Validation**: Centralized validation rules applied consistently
//! 5. **Format Detection**: Automatic format detection and conversion
//! 6. **Security**: Centralized input sanitization and size limits
//!
//! ## Integration Pattern
//!
//! ```rust,ignore
//! // 1. ParseService performs initial parsing and validation
//! let parsed_doc = self.parse_service
//!     .parse_with_format(content, ParseFormat::Json(...))
//!     .await?;
//!
//! // 2. Extract validated text from ParsedDocument
//! let text = match &parsed_doc.content {
//!     DocumentContent::Text(s) => s.as_str(),
//!     _ => return Err(...),
//! };
//!
//! // 3. Deserialize into LinkML SchemaDefinition
//! serde_json::from_str::<SchemaDefinition>(text)?
//! ```
//!
//! File system operations are handled via the `FileSystemOperations` trait
//! for sandboxed, testable file access.

use linkml_core::{
    error::{LinkMLError, Result},
    types::SchemaDefinition,
};
use parse_core::{ParseService, ParseFormat};
use std::path::Path;
use std::sync::Arc;

use super::{AsyncSchemaParser, SchemaParser};
use crate::file_system_adapter::FileSystemOperations;

/// `JSON` parser implementation with ParseService and file system adapter
#[derive(Clone)]
pub struct JsonParserV2<P: ParseService, F: FileSystemOperations> {
    parse_service: Arc<P>,
    fs: Arc<F>,
}

impl<P: ParseService, F: FileSystemOperations> JsonParserV2<P, F> {
    /// Create a new `JSON` parser with ParseService and file system adapter
    pub fn new(parse_service: Arc<P>, fs: Arc<F>) -> Self {
        Self { parse_service, fs }
    }
}

impl<P: ParseService, F: FileSystemOperations> SchemaParser for JsonParserV2<P, F> {
    fn parse_str(&self, content: &str) -> Result<SchemaDefinition> {
        // Use ParseService, then deserialize from content
        let parsed_doc = tokio::runtime::Handle::current()
            .block_on(self.parse_service.parse_with_format(content, ParseFormat::Json(parse_core::JsonFormat::Standard)))
            .map_err(|e| LinkMLError::parse(format!("Parse service error: {e}")))?;

        // Extract text from DocumentContent
        let text = match &parsed_doc.content {
            parse_core::DocumentContent::Text(s) => s.as_str(),
            _ => return Err(LinkMLError::parse("Expected text content from JSON parser")),
        };

        serde_json::from_str(text).map_err(|e| {
            LinkMLError::parse_at(
                format!("JSON deserialization error: {e}"),
                format!("line {}, column {}", e.line(), e.column()),
            )
        })
    }

    fn parse_file(&self, path: &Path) -> Result<SchemaDefinition> {
        // Note: This is a sync trait method, but we need to use async fs operations
        // In a real implementation, we'd need to refactor the trait to be async
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

#[async_trait::async_trait]
impl<P: ParseService, F: FileSystemOperations> AsyncSchemaParser for JsonParserV2<P, F> {
    async fn parse_str(&self, content: &str) -> Result<SchemaDefinition> {
        // Use ParseService, then deserialize from content
        let parsed_doc = self
            .parse_service
            .parse_with_format(content, ParseFormat::Json(parse_core::JsonFormat::Standard))
            .await
            .map_err(|e| LinkMLError::parse(format!("Parse service error: {e}")))?;

        // Extract text from DocumentContent
        let text = match &parsed_doc.content {
            parse_core::DocumentContent::Text(s) => s.as_str(),
            _ => return Err(LinkMLError::parse("Expected text content from JSON parser")),
        };

        serde_json::from_str(text).map_err(|e| {
            LinkMLError::parse_at(
                format!("JSON deserialization error: {e}"),
                format!("line {}, column {}", e.line(), e.column()),
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
                id: "mock".to_string(),
                format: ParseFormat::Json(parse_core::JsonFormat::Standard),
                metadata: Default::default(),
                content: parse_core::DocumentContent::Text(content.to_string()),
                validation_status: None,
                parsing_metadata: None,
            })
        }

        async fn parse_with_format(
            &self,
            content: &str,
            format: ParseFormat,
        ) -> std::result::Result<ParsedDocument, Self::Error> {
            Ok(ParsedDocument {
                id: "mock".to_string(),
                format,
                metadata: Default::default(),
                content: parse_core::DocumentContent::Text(content.to_string()),
                validation_status: None,
                parsing_metadata: None,
            })
        }
    }

    #[tokio::test]
    async fn test_json_parser_v2() -> std::result::Result<(), anyhow::Error> {
        let temp_dir = TempDir::new()?;
        let fs = Arc::new(TokioFileSystemAdapter::sandboxed(
            temp_dir.path().to_path_buf(),
        ));
        let parse_service = Arc::new(MockParseService);
        let parser = JsonParserV2::new(parse_service, fs.clone());

        // Create a test schema
        let schema_content = r#"{
  "id": "https://example.org/test",
  "name": "TestSchema",
  "description": "A test schema",
  "classes": {
    "Person": {
      "name": "Person",
      "description": "A person",
      "attributes": {
        "name": {
          "name": "name",
          "range": "string",
          "required": true
        },
        "age": {
          "name": "age",
          "range": "integer"
        }
      }
    }
  }
}"#;

        // Write to file using relative path within sandbox
        let schema_path = Path::new("test_schema.json");
        fs.write(schema_path, schema_content).await?;

        // Parse using async trait - explicitly use AsyncSchemaParser trait
        let schema = <JsonParserV2<MockParseService, TokioFileSystemAdapter> as AsyncSchemaParser>::parse_file(
            &parser,
            schema_path,
        )
        .await?;
        assert_eq!(schema.name, "TestSchema");
        assert!(schema.classes.contains_key("Person"));
        Ok(())
    }
}
