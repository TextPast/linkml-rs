//! Schema parsing module for LinkML service
//!
//! This module provides schema parsing for LinkML schemas in YAML and JSON formats.
//!
//! # Architecture
//!
//! This module uses RootReal's ParseService for all document parsing operations.
//! ParseService handles format detection, validation, and initial parsing, then
//! we deserialize the raw content to LinkML's typed `SchemaDefinition` structures.
//!
//! This architecture ensures:
//! 1. Consistent parsing across RootReal services
//! 2. Centralized format detection and validation
//! 3. Type-safe deserialization to LinkML domain models

use linkml_core::{
    error::{LinkMLError, Result},
    types::SchemaDefinition,
};
use parse_core::{ParseService, ParseFormat};
use std::path::Path;
use std::sync::Arc;

pub mod factory;
pub mod import_resolver;
pub mod import_resolver_v2;
pub mod json_parser_v2;
pub mod schema_loader;
pub mod yaml_parser_v2;

pub use import_resolver::ImportResolver;
pub use import_resolver_v2::{ImportResolverV2, ImportSpec};
pub use json_parser_v2::JsonParserV2;
pub use schema_loader::SchemaLoader;
pub use yaml_parser_v2::{AsyncSchemaParser, YamlParserV2};

/// Trait for schema parsers
pub trait SchemaParser: Send + Sync {
    /// Parse schema from string content
    ///
    /// # Errors
    ///
    /// Returns a `LinkMLError` if parsing fails
    fn parse_str(&self, content: &str) -> Result<SchemaDefinition>;

    /// Parse schema from file
    ///
    /// # Errors
    ///
    /// Returns a `LinkMLError` if:
    /// - File cannot be read
    /// - Parsing fails
    fn parse_file(&self, path: &Path) -> Result<SchemaDefinition>;
}

/// Main parser for LinkML schemas
///
/// Supports both YAML and JSON formats with automatic format detection.
/// Uses ParseService for consistent document parsing across RootReal.
pub struct Parser<P: ParseService> {
    /// Parse service for document parsing
    parse_service: Arc<P>,
    /// Whether to automatically resolve imports
    auto_resolve_imports: bool,
}

impl<P: ParseService> Parser<P> {
    /// Create a new parser with ParseService
    #[must_use]
    pub fn new(parse_service: Arc<P>) -> Self {
        Self {
            parse_service,
            auto_resolve_imports: false,
        }
    }

    /// Create a parser that automatically resolves imports
    #[must_use]
    pub fn with_import_resolution(parse_service: Arc<P>) -> Self {
        Self {
            parse_service,
            auto_resolve_imports: true,
        }
    }

    /// Set whether to automatically resolve imports
    pub fn set_auto_resolve_imports(&mut self, enabled: bool) {
        self.auto_resolve_imports = enabled;
    }

    /// Parse schema from file, detecting format from extension
    ///
    /// # Errors
    ///
    /// Returns a `LinkMLError` if:
    /// - File has no extension
    /// - File format is not supported
    /// - Parsing fails
    pub async fn parse_file(&self, path: &Path) -> Result<SchemaDefinition> {
        // Read file content
        let content = std::fs::read_to_string(path)
            .map_err(|e| LinkMLError::io(format!("Failed to read file {}: {}", path.display(), e)))?;

        // Detect format from extension
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| LinkMLError::parse("No file extension found"))?;

        self.parse_str(&content, extension).await
    }

    /// Parse schema from string with specified format
    ///
    /// # Errors
    ///
    /// Returns a `LinkMLError` if:
    /// - Format is not supported
    /// - Parsing fails
    pub async fn parse_str(&self, content: &str, format: &str) -> Result<SchemaDefinition> {
        // Map format string to ParseFormat enum
        let parse_format = match format {
            "yaml" | "yml" => ParseFormat::Yaml,
            "json" => ParseFormat::Json,
            _ => return Err(LinkMLError::parse(format!("Unsupported format: {format}"))),
        };

        // Use ParseService to parse the document
        let parsed_doc = self.parse_service
            .parse_with_format(content, parse_format)
            .await
            .map_err(|e| LinkMLError::parse(format!("Parse service error: {e}")))?;

        // Extract raw content and deserialize to SchemaDefinition
        match parse_format {
            ParseFormat::Yaml => {
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
            ParseFormat::Json => {
                serde_json::from_str(&parsed_doc.raw_content).map_err(|e| {
                    LinkMLError::parse_at(
                        format!("JSON deserialization error: {e}"),
                        format!("line {}, column {}", e.line(), e.column()),
                    )
                })
            }
            _ => Err(LinkMLError::parse(format!("Unsupported format: {format}"))),
        }
    }
}

// Note: No Default impl - Parser requires ParseService dependency

#[cfg(test)]
mod tests {
    use super::*;
    use parse_core::{ParsedDocument, DocumentContent, ParseError};
    use async_trait::async_trait;

    // Simple mock ParseService for testing
    struct MockParseService;

    #[async_trait]
    impl ParseService for MockParseService {
        type Error = ParseError;

        async fn parse(&self, _content: &str) -> Result<ParsedDocument, Self::Error> {
            unimplemented!("Not needed for these tests")
        }

        async fn parse_with_format(
            &self,
            content: &str,
            _format: ParseFormat,
        ) -> Result<ParsedDocument, Self::Error> {
            // Return a ParsedDocument with the content as raw_content
            Ok(ParsedDocument {
                raw_content: content.to_string(),
                content: DocumentContent::Raw(content.to_string()),
                format: _format,
                metadata: Default::default(),
            })
        }

        async fn detect_format(&self, _content: &str) -> Result<(ParseFormat, f64), Self::Error> {
            unimplemented!("Not needed for these tests")
        }

        async fn parse_oai_pmh_harvest_session(
            &self,
            _content: &str,
        ) -> Result<parse_core::OaiPmhHarvestSession, Self::Error> {
            unimplemented!("Not needed for these tests")
        }

        async fn parse_oai_pmh_records(
            &self,
            _oai_pmh_response: &str,
        ) -> Result<Vec<parse_core::OaiPmhRecord>, Self::Error> {
            unimplemented!("Not needed for these tests")
        }

        async fn parse_with_profile(
            &self,
            _content: &str,
            _format: ParseFormat,
            _profile: &parse_core::ExtractionProfile,
        ) -> Result<parse_core::ExtractedData, Self::Error> {
            unimplemented!("Not needed for these tests")
        }

        async fn parse_csv_with_options(
            &self,
            _content: &str,
            _options: parse_core::CsvOptions,
        ) -> Result<ParsedDocument, Self::Error> {
            unimplemented!("Not needed for these tests")
        }

        async fn validate_document(
            &self,
            _document: &ParsedDocument,
            _schema_name: Option<&str>,
        ) -> Result<parse_core::ValidationResult, Self::Error> {
            unimplemented!("Not needed for these tests")
        }

        async fn health_check(&self) -> Result<bool, Self::Error> {
            Ok(true)
        }

        async fn reload_configuration(&self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_parser_creation() {
        let parse_service = Arc::new(MockParseService);
        let parser = Parser::new(parse_service);
        assert!(!parser.auto_resolve_imports);
    }

    #[tokio::test]
    async fn test_parser_with_import_resolution() {
        let parse_service = Arc::new(MockParseService);
        let parser = Parser::with_import_resolution(parse_service);
        assert!(parser.auto_resolve_imports);
    }

    #[tokio::test]
    async fn test_parse_str_yaml() -> Result<()> {
        let yaml = r"
id: https://example.org/test
name: test_schema
";
        let parse_service = Arc::new(MockParseService);
        let parser = Parser::new(parse_service);
        let schema = parser.parse_str(yaml, "yaml").await?;

        assert_eq!(schema.id, "https://example.org/test");
        assert_eq!(schema.name, "test_schema");
        Ok(())
    }

    #[tokio::test]
    async fn test_unsupported_format() {
        let parse_service = Arc::new(MockParseService);
        let parser = Parser::new(parse_service);
        let result = parser.parse_str("content", "xml").await;
        assert!(result.is_err());
        
        if let Err(LinkMLError::ParseError { message, .. }) = result {
            assert!(message.contains("Unsupported format"));
        } else {
            panic!("Expected ParseError for unsupported format");
        }
    }
}
