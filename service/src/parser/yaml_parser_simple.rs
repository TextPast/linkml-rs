//! Lightweight YAML parser without dependency injection
//!
//! **DEPRECATED**: This parser bypasses RootReal's centralized parsing architecture.
//! Use the proper test utilities from `crates/testing/test-utils/` instead, which
//! wrap the centralized ParseService with proper dependency injection.
//!
//! This parser was designed for CLI tools and simple applications, but RootReal's
//! architecture requires ALL parsing to go through centralized infrastructure for
//! consistent error handling, logging, and telemetry.

use linkml_core::{error::{LinkMLError, Result}, types::SchemaDefinition};
use std::{fs, path::Path};

use super::SchemaParser;

/// Lightweight YAML parser with zero service dependencies
///
/// **DEPRECATED**: Use test utilities from `crates/testing/test-utils/` instead.
///
/// This parser bypasses RootReal's centralized parsing architecture by calling
/// serde_yaml directly. RootReal requires ALL parsing to use the centralized
/// ParseService for consistent error handling, logging, and telemetry.
///
/// # Migration Path
///
/// Instead of:
/// ```rust,ignore
/// let parser = YamlParserSimple::new();
/// let schema = parser.parse_str(content)?;
/// ```
///
/// Use proper test utilities:
/// ```rust,ignore
/// use testing_test_utils::linkml::create_test_linkml_parser;
///
/// let parser = create_test_linkml_parser();
/// let schema = parser.parse_yaml_str(content)?;
/// ```
///
/// For production code, use YamlParserV2 with full dependency injection.
#[deprecated(
    since = "0.1.0",
    note = "Use test utilities from testing-test-utils crate or YamlParserV2 with proper DI"
)]
#[derive(Default, Debug, Clone, Copy)]
pub struct YamlParserSimple;

impl YamlParserSimple {
    /// Create a new simple YAML parser
    ///
    /// **DEPRECATED**: Use `testing_test_utils::linkml::create_test_linkml_parser()` instead.
    ///
    /// This bypasses RootReal's centralized ParseService architecture.
    #[deprecated(
        since = "0.1.0",
        note = "Use testing_test_utils::linkml::create_test_linkml_parser() for tests, or YamlParserV2 with DI for production"
    )]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl SchemaParser for YamlParserSimple {
    fn parse_str(&self, content: &str) -> Result<SchemaDefinition> {
        serde_yaml::from_str(content).map_err(|e| {
            LinkMLError::parse_at(
                format!("YAML parsing error: {e}"),
                e.location().map_or_else(
                    || "unknown location".to_string(),
                    |l| format!("line {}, column {}", l.line(), l.column()),
                ),
            )
        })
    }

    fn parse_file(&self, path: &Path) -> Result<SchemaDefinition> {
        let content = fs::read_to_string(path).map_err(LinkMLError::IoError)?;
        self.parse_str(&content).map_err(|e| match e {
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

    #[test]
    fn test_yaml_parser_simple_new() {
        let parser = YamlParserSimple::new();
        assert!(std::mem::size_of_val(&parser) == 0, "YamlParserSimple should be zero-sized");
    }

    #[test]
    fn test_parse_str_basic() {
        let parser = YamlParserSimple::new();
        let yaml = r"
id: https://example.org/test
name: test_schema
";
        let schema = parser.parse_str(yaml).expect("Parse failed");
        assert_eq!(schema.id, "https://example.org/test");
        assert_eq!(schema.name, "test_schema");
    }

    #[test]
    fn test_parse_str_invalid_yaml() {
        let parser = YamlParserSimple::new();
        let invalid_yaml = "{ invalid: yaml: structure";
        let result = parser.parse_str(invalid_yaml);
        assert!(result.is_err());
        
        if let Err(LinkMLError::ParseError { message, .. }) = result {
            assert!(message.contains("YAML parsing error"));
        } else {
            panic!("Expected ParseError");
        }
    }

    #[test]
    fn test_parse_file_not_found() {
        let parser = YamlParserSimple::new();
        let result = parser.parse_file(Path::new("/nonexistent/file.yaml"));
        assert!(result.is_err());
        
        if let Err(LinkMLError::IoError(_)) = result {
            // Expected
        } else {
            panic!("Expected IoError");
        }
    }

    #[test]
    fn test_default_trait() {
        let parser = YamlParserSimple::default();
        let yaml = r"
id: https://example.org/default
name: default_schema
";
        let schema = parser.parse_str(yaml).expect("Parse failed");
        assert_eq!(schema.name, "default_schema");
    }

    #[test]
    fn test_clone_trait() {
        let parser1 = YamlParserSimple::new();
        let parser2 = parser1.clone();
        
        let yaml = r"
id: https://example.org/clone
name: clone_schema
";
        let schema = parser2.parse_str(yaml).expect("Parse failed");
        assert_eq!(schema.name, "clone_schema");
    }
}
