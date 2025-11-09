//! Lightweight YAML parser without dependency injection
//!
//! This parser provides a real, lightweight parsing implementation designed
//! for CLI tools, simple applications, unit tests, and documentation examples
//! where the full ParseService infrastructure would be unnecessary overhead.
//!
//! Unlike mock implementations in testing-mocks, this performs actual YAML
//! parsing using serde_yaml and is suitable for production use in simple tools.

use linkml_core::{error::{LinkMLError, Result}, types::SchemaDefinition};
use std::{fs, path::Path};

use super::SchemaParser;

/// Lightweight YAML parser with zero service dependencies
///
/// This is a real parser (not a mock) that performs actual YAML parsing.
/// Designed for:
/// - CLI tools and simple applications
/// - Unit tests without complex setup
/// - Documentation examples
/// - Scenarios where ParseService DI infrastructure is unnecessary
///
/// For production services with full observability, use YamlParserV2 instead.
///
/// # Example
///
/// ```rust
/// use linkml_service::parser::{YamlParserSimple, SchemaParser};
/// use std::path::Path;
///
/// let parser = YamlParserSimple::new();
/// let yaml_content = r#"
/// id: https://example.org/test
/// name: test_schema
/// "#;
/// let schema = parser.parse_str(yaml_content).expect("Parse failed");
/// assert_eq!(schema.name, "test_schema");
/// ```
#[derive(Default, Debug, Clone, Copy)]
pub struct YamlParserSimple;

impl YamlParserSimple {
    /// Create a new simple YAML parser
    ///
    /// This is a zero-cost operation as the parser has no state.
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
