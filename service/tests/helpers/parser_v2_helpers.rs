//! Test helper functions for V2 parser usage
//!
//! This module provides convenient helper functions for creating V2 parsers
//! in test code, reducing boilerplate and ensuring consistent usage patterns.
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::helpers::parser_v2_helpers::create_test_yaml_parser;
//!
//! #[tokio::test]
//! async fn test_parse_schema() {
//!     let parser = create_test_yaml_parser();
//!     let schema = parser.parse_file(&path).await?;
//!     // ... test assertions
//! }
//! ```

use std::sync::Arc;
use crate::file_system_adapter::TokioFileSystemAdapter;
use crate::parser::{YamlParserV2, JsonParserV2};

/// Create a YamlParserV2 instance for testing
///
/// This function creates a `YamlParserV2` with a `TokioFileSystemAdapter`
/// suitable for use in test code. It eliminates boilerplate setup code
/// and ensures consistent parser configuration across tests.
///
/// # Returns
///
/// A fully configured `YamlParserV2` ready for parsing operations.
///
/// # Example
///
/// ```rust,ignore
/// use crate::helpers::parser_v2_helpers::create_test_yaml_parser;
///
/// #[tokio::test]
/// async fn test_yaml_parsing() {
///     let parser = create_test_yaml_parser();
///     
///     let yaml = r#"
///     id: https://example.org/test
///     name: test_schema
///     "#;
///     
///     let schema = parser.parse_str(yaml).expect("Parse failed");
///     assert_eq!(schema.name, Some("test_schema".to_string()));
/// }
/// ```
#[must_use]
pub fn create_test_yaml_parser() -> YamlParserV2<TokioFileSystemAdapter> {
    let fs_adapter = Arc::new(TokioFileSystemAdapter::new());
    YamlParserV2::new(fs_adapter)
}

/// Create a JsonParserV2 instance for testing
///
/// This function creates a `JsonParserV2` with a `TokioFileSystemAdapter`
/// and a ParseService suitable for use in test code.
///
/// # Returns
///
/// A fully configured `JsonParserV2` ready for parsing operations.
///
/// # Example
///
/// ```rust,ignore
/// use crate::helpers::parser_v2_helpers::create_test_json_parser;
///
/// #[tokio::test]
/// async fn test_json_parsing() {
///     let parser = create_test_json_parser();
///     
///     let json = r#"
///     {
///         "id": "https://example.org/test",
///         "name": "test_schema"
///     }
///     "#;
///     
///     let schema = parser.parse_str(json).expect("Parse failed");
///     assert_eq!(schema.name, Some("test_schema".to_string()));
/// }
/// ```
#[must_use]
pub fn create_test_json_parser() -> JsonParserV2<parse_core::ParseService, TokioFileSystemAdapter> {
    let fs_adapter = Arc::new(TokioFileSystemAdapter::new());
    let parse_service = Arc::new(parse_core::ParseService::new());
    JsonParserV2::new(parse_service, fs_adapter)
}

/// Create both YAML and JSON parsers sharing the same file system adapter
///
/// This function is useful when you need both parsers in the same test
/// and want them to share a single file system adapter for efficiency.
///
/// # Returns
///
/// A tuple of `(YamlParserV2, JsonParserV2)` sharing the same adapter.
///
/// # Example
///
/// ```rust,ignore
/// use crate::helpers::parser_v2_helpers::create_test_parsers;
///
/// #[tokio::test]
/// async fn test_multi_format_parsing() {
///     let (yaml_parser, json_parser) = create_test_parsers();
///     
///     // Parse YAML
///     let yaml_schema = yaml_parser.parse_str(yaml_content)?;
///     
///     // Parse JSON
///     let json_schema = json_parser.parse_str(json_content)?;
///     
///     // Compare results
///     assert_eq!(yaml_schema.name, json_schema.name);
/// }
/// ```
#[must_use]
pub fn create_test_parsers() -> (
    YamlParserV2<TokioFileSystemAdapter>,
    JsonParserV2<parse_core::ParseService, TokioFileSystemAdapter>
) {
    let fs_adapter = Arc::new(TokioFileSystemAdapter::new());
    let parse_service = Arc::new(parse_core::ParseService::new());
    
    let yaml_parser = YamlParserV2::new(fs_adapter.clone());
    let json_parser = JsonParserV2::new(parse_service, fs_adapter);
    
    (yaml_parser, json_parser)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_yaml_parser() {
        let parser = create_test_yaml_parser();
        
        // Test with valid YAML
        let yaml = r#"
id: https://example.org/test
name: test_schema
"#;
        
        let result = parser.parse_str(yaml);
        assert!(result.is_ok(), "Parser should successfully parse valid YAML");
    }

    #[tokio::test]
    async fn test_create_json_parser() {
        let parser = create_test_json_parser();
        
        // Test with valid JSON
        let json = r#"
{
    "id": "https://example.org/test",
    "name": "test_schema"
}
"#;
        
        let result = parser.parse_str(json);
        assert!(result.is_ok(), "Parser should successfully parse valid JSON");
    }

    #[tokio::test]
    async fn test_create_both_parsers() {
        let (yaml_parser, json_parser) = create_test_parsers();
        
        // Both parsers should work
        let yaml = r#"
id: https://example.org/test
name: yaml_test
"#;
        
        let json = r#"
{
    "id": "https://example.org/test",
    "name": "json_test"
}
"#;
        
        let yaml_result = yaml_parser.parse_str(yaml);
        let json_result = json_parser.parse_str(json);
        
        assert!(yaml_result.is_ok(), "YAML parser should work");
        assert!(json_result.is_ok(), "JSON parser should work");
    }
}
