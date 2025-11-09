//! Lightweight JSON parser without dependency injection
//!
//! This parser provides a real, lightweight parsing implementation designed
//! for CLI tools, simple applications, unit tests, and documentation examples
//! where the full ParseService infrastructure would be unnecessary overhead.
//!
//! Unlike mock implementations in testing-mocks, this performs actual JSON
//! parsing using serde_json and is suitable for production use in simple tools.

use linkml_core::{error::{LinkMLError, Result}, types::SchemaDefinition};
use std::{fs, path::Path};

use super::SchemaParser;

/// Lightweight JSON parser with zero service dependencies
///
/// This is a real parser (not a mock) that performs actual JSON parsing.
/// Designed for:
/// - CLI tools and simple applications
/// - Unit tests without complex setup
/// - Documentation examples
/// - Scenarios where ParseService DI infrastructure is unnecessary
///
/// For production services with full observability, use JsonParserV2 instead.
///
/// # Example
///
/// ```rust
/// use linkml_service::parser::{JsonParserSimple, SchemaParser};
/// use std::path::Path;
///
/// let parser = JsonParserSimple::new();
/// let json_content = r#"
/// {
///   "id": "https://example.org/test",
///   "name": "test_schema"
/// }
/// "#;
/// let schema = parser.parse_str(json_content).expect("Parse failed");
/// assert_eq!(schema.name, "test_schema");
/// ```
#[derive(Default, Debug, Clone, Copy)]
pub struct JsonParserSimple;

impl JsonParserSimple {
    /// Create a new simple JSON parser
    ///
    /// This is a zero-cost operation as the parser has no state.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl SchemaParser for JsonParserSimple {
    fn parse_str(&self, content: &str) -> Result<SchemaDefinition> {
        serde_json::from_str(content).map_err(|e| {
            LinkMLError::parse_at(
                format!("JSON parsing error: {e}"),
                format!("line {}, column {}", e.line(), e.column()),
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
    fn test_json_parser_simple_new() {
        let parser = JsonParserSimple::new();
        assert!(std::mem::size_of_val(&parser) == 0, "JsonParserSimple should be zero-sized");
    }

    #[test]
    fn test_parse_str_basic() {
        let parser = JsonParserSimple::new();
        let json = r#"
{
  "id": "https://example.org/test",
  "name": "test_schema"
}
"#;
        let schema = parser.parse_str(json).expect("Parse failed");
        assert_eq!(schema.id, "https://example.org/test");
        assert_eq!(schema.name, "test_schema");
    }

    #[test]
    fn test_parse_str_invalid_json() {
        let parser = JsonParserSimple::new();
        let invalid_json = "{ invalid: json structure";
        let result = parser.parse_str(invalid_json);
        assert!(result.is_err());
        
        if let Err(LinkMLError::ParseError { message, .. }) = result {
            assert!(message.contains("JSON parsing error"));
        } else {
            panic!("Expected ParseError");
        }
    }

    #[test]
    fn test_parse_file_not_found() {
        let parser = JsonParserSimple::new();
        let result = parser.parse_file(Path::new("/nonexistent/file.json"));
        assert!(result.is_err());
        
        if let Err(LinkMLError::IoError(_)) = result {
            // Expected
        } else {
            panic!("Expected IoError");
        }
    }

    #[test]
    fn test_default_trait() {
        let parser = JsonParserSimple::default();
        let json = r#"
{
  "id": "https://example.org/default",
  "name": "default_schema"
}
"#;
        let schema = parser.parse_str(json).expect("Parse failed");
        assert_eq!(schema.name, "default_schema");
    }

    #[test]
    fn test_clone_trait() {
        let parser1 = JsonParserSimple::new();
        let parser2 = parser1.clone();
        
        let json = r#"
{
  "id": "https://example.org/clone",
  "name": "clone_schema"
}
"#;
        let schema = parser2.parse_str(json).expect("Parse failed");
        assert_eq!(schema.name, "clone_schema");
    }
}
