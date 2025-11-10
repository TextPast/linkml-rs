//! Schema parsing module for LinkML service
//!
//! This module provides schema parsing for LinkML schemas in YAML and JSON formats.
//!
//! # Architecture
//!
//! **CRITICAL**: All LinkML parsing MUST use RootReal's centralized parsing infrastructure
//! from `crates/data/parsing/parse/`:
//!
//! - **JSON parsing**: Uses `ParseService` → `serde_json` pipeline
//!   - ParseService handles format detection, validation, caching
//!   - Then deserializes to `SchemaDefinition` via `serde_json`
//!
//! - **YAML parsing**: Uses `parse-linkml` specialized PEG parser
//!   - Located at `crates/data/parsing/parse/linkml-parser/`
//!   - Part of centralized infrastructure (not direct `serde_yaml` usage)
//!   - Required because `ParseFormat` doesn't support LinkML/YAML yet
//!   - Provides LinkML-specific grammar validation and semantic checks
//!
//! ## Why Different Approaches for JSON vs YAML?
//!
//! - **JSON**: ParseService supports JSON via `ParseFormat::Json`, so we use it
//! - **YAML**: ParseService doesn't support YAML/LinkML yet, so we use specialized
//!   parser from `crates/data/parsing/parse/linkml-parser/`
//! - **Both comply** with mandatory centralized parsing (both use parsers from
//!   `crates/data/parsing/parse/`, NOT generic `serde_*` directly)
//!
//! ## Parser Implementations
//!
//! - **Simple Parsers** (`json_parser_simple`, `yaml_parser_simple`): Legacy direct serde
//!   parsing - these violate centralized architecture and should be migrated
//! - **V2 Parsers** (`json_parser_v2`, `yaml_parser_v2`): Production-ready implementations
//!   that comply with RootReal's mandatory centralized parsing architecture

use linkml_core::{
    error::{LinkMLError, Result},
    types::SchemaDefinition,
};
use std::path::Path;

pub mod factory;
pub mod import_resolver;
pub mod import_resolver_v2;
pub mod json_parser_simple;
pub mod json_parser_v2;
pub mod schema_loader;
pub mod yaml_parser_simple;
pub mod yaml_parser_v2;

pub use import_resolver::ImportResolver;
pub use import_resolver_v2::{ImportResolverV2, ImportSpec};
pub use json_parser_simple::JsonParserSimple;
pub use json_parser_v2::JsonParserV2;
pub use schema_loader::SchemaLoader;
pub use yaml_parser_simple::YamlParserSimple;
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
/// **DEPRECATED**: This parser uses direct `serde_yaml`/`serde_json` which violates
/// RootReal's centralized parsing architecture. Use `YamlParserV2` or `JsonParserV2` instead.
///
/// Supports both YAML and JSON formats with automatic format detection.
/// Parses LinkML schemas directly using serde_yaml/serde_json.
#[derive(Clone)]
#[deprecated(
    since = "0.2.0",
    note = "Use YamlParserV2 or JsonParserV2 instead. This parser violates RootReal's centralized parsing architecture."
)]
pub struct Parser {
    /// Whether to automatically resolve imports
    auto_resolve_imports: bool,
}

impl Parser {
    /// Create a new parser
    ///
    /// **DEPRECATED**: Use `YamlParserV2::new()` or `JsonParserV2::new()` instead.
    ///
    /// # Migration Example
    ///
    /// ```rust,ignore
    /// // Old (deprecated):
    /// use linkml_service::parser::{Parser, SchemaParser};
    /// let parser = Parser::new();
    /// let schema = parser.parse_str(yaml_content, "yaml")?;
    ///
    /// // New (V2 YAML):
    /// use std::sync::Arc;
    /// use file_system_adapter::TokioFileSystemAdapter;
    /// use linkml_service::parser::{YamlParserV2, AsyncSchemaParser};
    /// 
    /// let fs_adapter = Arc::new(TokioFileSystemAdapter::new());
    /// let parser = YamlParserV2::new(fs_adapter);
    /// let schema = parser.parse_str(yaml_content).await?;
    ///
    /// // New (V2 JSON):
    /// use parse_core::ParseService;
    /// use linkml_service::parser::JsonParserV2;
    /// 
    /// let fs_adapter = Arc::new(TokioFileSystemAdapter::new());
    /// let parse_service = Arc::new(ParseService::new());
    /// let parser = JsonParserV2::new(parse_service, fs_adapter);
    /// let schema = parser.parse_str(json_content).await?;
    /// ```
    #[must_use]
    #[deprecated(
        since = "0.2.0",
        note = "Use YamlParserV2::new() or JsonParserV2::new() instead"
    )]
    pub fn new() -> Self {
        Self {
            auto_resolve_imports: false,
        }
    }

    /// Create a parser that automatically resolves imports
    ///
    /// **DEPRECATED**: Use `YamlParserV2::new()` or `JsonParserV2::new()` instead.
    /// Import resolution is built into V2 parsers.
    #[must_use]
    #[deprecated(
        since = "0.2.0",
        note = "Use YamlParserV2::new() or JsonParserV2::new() instead. Import resolution is built-in."
    )]
    pub fn with_import_resolution() -> Self {
        Self {
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
    /// - File cannot be read
    /// - File has no extension
    /// - File format is not supported
    /// - Parsing fails
    pub fn parse_file(&self, path: &Path) -> Result<SchemaDefinition> {
        // Read file content
        let content = std::fs::read_to_string(path)?;

        // Detect format from extension
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| LinkMLError::parse("No file extension found"))?;

        self.parse_str(&content, extension)
    }

    /// Parse schema from string with specified format
    ///
    /// # Errors
    ///
    /// Returns a `LinkMLError` if:
    /// - Format is not supported
    /// - Parsing fails
    pub fn parse_str(&self, content: &str, format: &str) -> Result<SchemaDefinition> {
        // Parse based on format
        match format {
            "yaml" | "yml" => {
                serde_yaml::from_str(content).map_err(|e| {
                    LinkMLError::parse_at(
                        format!("YAML deserialization error: {e}"),
                        e.location().map_or_else(
                            || "unknown location".to_string(),
                            |l| format!("line {}, column {}", l.line(), l.column()),
                        ),
                    )
                })
            }
            "json" => {
                serde_json::from_str(content).map_err(|e| {
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

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_parser_creation() {
        let parser = Parser::new();
        assert!(!parser.auto_resolve_imports);
    }

    #[test]
    #[allow(deprecated)]
    fn test_parser_with_import_resolution() {
        let parser = Parser::with_import_resolution();
        assert!(parser.auto_resolve_imports);
    }

    #[test]
    #[allow(deprecated)]
    fn test_parse_str_yaml() -> Result<()> {
        let yaml = r"
id: https://example.org/test
name: test_schema
";
        let parser = Parser::new();
        let schema = parser.parse_str(yaml, "yaml")?;

        assert_eq!(schema.id, "https://example.org/test");
        assert_eq!(schema.name, "test_schema");
        Ok(())
    }

    #[test]
    #[allow(deprecated)]
    fn test_unsupported_format() {
        let parser = Parser::new();
        let result = parser.parse_str("content", "xml");
        assert!(result.is_err());
        
        if let Err(LinkMLError::ParseError { message, .. }) = result {
            assert!(message.contains("Unsupported format"));
        } else {
            panic!("Expected ParseError for unsupported format");
        }
    }
}
