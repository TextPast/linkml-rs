//! Abstract Syntax Tree (AST) for LinkML schemas
//!
//! This module defines the AST structures generated directly from Pest parsing.
//! These structures are designed for efficient parsing and can be converted to
//! the full type system in `types.rs`.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Span information for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Start position (byte offset)
    pub start: usize,
    /// End position (byte offset)
    pub end: usize,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

impl Span {
    /// Create a new span
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    /// Create a span that encompasses both spans
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            column: self.column.min(other.column),
        }
    }
}

/// AST node with span information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spanned<T> {
    /// The actual node value
    pub value: T,
    /// Location in source
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Create a new spanned node
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

/// Document type (schema or instance)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    /// Schema document
    Schema,
    /// Instance document
    Instance,
}

/// Type alias for LinkML document (schema or instance)
pub type LinkMLDocument = SchemaAst;

/// Root schema AST node
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SchemaAst {
    /// Document type (schema or instance)
    pub document_type: Option<DocumentType>,
    /// Schema ID (required)
    pub id: Option<Spanned<String>>,
    /// Schema name (required)
    pub name: Option<Spanned<String>>,
    /// Title
    pub title: Option<Spanned<String>>,
    /// Description
    pub description: Option<Spanned<Description>>,
    /// Version
    pub version: Option<Spanned<String>>,
    /// License
    pub license: Option<Spanned<String>>,
    /// Creation timestamp
    pub created_on: Option<Spanned<String>>,
    /// Last updated timestamp
    pub last_updated_on: Option<Spanned<String>>,
    /// Default prefix
    pub default_prefix: Option<Spanned<String>>,
    /// Default range
    pub default_range: Option<Spanned<String>>,
    /// Metamodel version
    pub metamodel_version: Option<Spanned<String>>,
    /// Source file
    pub source_file: Option<Spanned<String>>,
    /// Generation date
    pub generation_date: Option<Spanned<String>>,
    /// Status
    pub status: Option<Spanned<String>>,
    /// Prefixes
    pub prefixes: IndexMap<String, Spanned<String>>,
    /// Imports
    pub imports: Vec<Spanned<String>>,
    /// Settings (pattern definitions)
    pub settings: IndexMap<String, Spanned<String>>,
    /// Classes
    pub classes: IndexMap<String, Spanned<ClassAst>>,
    /// Slots
    pub slots: IndexMap<String, Spanned<SlotAst>>,
    /// Types
    pub types: IndexMap<String, Spanned<TypeAst>>,
    /// Enums
    pub enums: IndexMap<String, Spanned<EnumAst>>,
    /// Subsets
    pub subsets: IndexMap<String, Spanned<SubsetAst>>,
    /// Contributors
    pub contributors: Vec<Spanned<ContributorAst>>,
    /// Categories
    pub categories: Vec<Spanned<String>>,
    /// Keywords
    pub keywords: Vec<Spanned<String>>,
    /// See also references
    pub see_also: Vec<Spanned<String>>,
    /// Annotations
    pub annotations: Option<Spanned<AnnotationsAst>>,
}

/// Description (can be inline or block string)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Description {
    /// Inline string
    Inline(String),
    /// Block string (multi-line)
    Block(String),
}

/// Class definition AST
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ClassAst {
    /// Class name (implicit from map key)
    pub name: String,
    /// Description
    pub description: Option<Spanned<Description>>,
    /// Parent class (is_a)
    pub is_a: Option<Spanned<String>>,
    /// Abstract class flag
    pub abstract_: Option<Spanned<bool>>,
    /// Mixin flag
    pub mixin: Option<Spanned<bool>>,
    /// Tree root flag
    pub tree_root: Option<Spanned<bool>>,
    /// Class URI
    pub class_uri: Option<Spanned<String>>,
    /// Mixins
    pub mixins: Vec<Spanned<String>>,
    /// Slots
    pub slots: Vec<Spanned<String>>,
    /// Slot usage (overrides)
    pub slot_usage: IndexMap<String, Spanned<SlotAst>>,
    /// Attributes (inline slot definitions)
    pub attributes: IndexMap<String, Spanned<SlotAst>>,
    /// Subclass of (OWL compatibility)
    pub subclass_of: Vec<Spanned<String>>,
    /// Rules
    pub rules: Vec<Spanned<RuleAst>>,
    /// Conditional requirements
    pub if_required: IndexMap<String, Spanned<ConditionalRequirementAst>>,
    /// Unique keys
    pub unique_keys: IndexMap<String, Spanned<UniqueKeyAst>>,
    /// Recursion options
    pub recursion_options: Option<Spanned<RecursionOptionsAst>>,
    /// Aliases
    pub aliases: Vec<Spanned<String>>,
    /// See also
    pub see_also: Vec<Spanned<String>>,
    /// ID prefixes
    pub id_prefixes: Vec<Spanned<String>>,
    /// Broad mappings
    pub broad_mappings: Vec<Spanned<String>>,
    /// Exact mappings
    pub exact_mappings: Vec<Spanned<String>>,
    /// Narrow mappings
    pub narrow_mappings: Vec<Spanned<String>>,
    /// Related mappings
    pub related_mappings: Vec<Spanned<String>>,
    /// Close mappings
    pub close_mappings: Vec<Spanned<String>>,
    /// Annotations
    pub annotations: Option<Spanned<AnnotationsAst>>,
}

/// Slot definition AST
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SlotAst {
    /// Slot name (implicit from map key)
    pub name: String,
    /// Description
    pub description: Option<Spanned<Description>>,
    /// Range (type)
    pub range: Option<Spanned<String>>,
    /// Range type (class vs instance)
    pub range_type: Option<Spanned<RangeType>>,
    /// Range properties (for instance validation)
    pub range_properties: Vec<Spanned<String>>,
    /// Required flag
    pub required: Option<Spanned<bool>>,
    /// Multivalued flag
    pub multivalued: Option<Spanned<bool>>,
    /// Identifier flag
    pub identifier: Option<Spanned<bool>>,
    /// Pattern validation
    pub pattern: Option<Spanned<String>>,
    /// Structured pattern (for identifier interpolation)
    pub structured_pattern: Option<Spanned<StructuredPatternAst>>,
    /// Minimum value
    pub minimum_value: Option<Spanned<ValueAst>>,
    /// Maximum value
    pub maximum_value: Option<Spanned<ValueAst>>,
    /// Minimum cardinality
    pub minimum_cardinality: Option<Spanned<i64>>,
    /// Maximum cardinality
    pub maximum_cardinality: Option<Spanned<i64>>,
    /// Parent slot (is_a)
    pub is_a: Option<Spanned<String>>,
    /// Mixins
    pub mixins: Vec<Spanned<String>>,
    /// Slot URI
    pub slot_uri: Option<Spanned<String>>,
    /// Domain (class this slot belongs to)
    pub domain: Option<Spanned<String>>,
    /// Inverse slot
    pub inverse: Option<Spanned<String>>,
    /// Symmetric property
    pub symmetric: Option<Spanned<bool>>,
    /// Asymmetric property
    pub asymmetric: Option<Spanned<bool>>,
    /// Reflexive property
    pub reflexive: Option<Spanned<bool>>,
    /// Irreflexive property
    pub irreflexive: Option<Spanned<bool>>,
    /// Locally reflexive property
    pub locally_reflexive: Option<Spanned<bool>>,
    /// Transitive property
    pub transitive: Option<Spanned<bool>>,
    /// Default value if absent
    pub ifabsent: Option<Spanned<String>>,
    /// Equals string constraint
    pub equals_string: Option<Spanned<String>>,
    /// Equals number constraint
    pub equals_number: Option<Spanned<f64>>,
    /// Aliases
    pub aliases: Vec<Spanned<String>>,
    /// See also
    pub see_also: Vec<Spanned<String>>,
    /// Scoped imports (for slot range resolution)
    pub imports: Vec<Spanned<String>>,
    /// Annotations
    pub annotations: Option<Spanned<AnnotationsAst>>,
}

/// Range type (class or instance)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RangeType {
    /// Range refers to a class definition
    Class,
    /// Range refers to an instance
    Instance,
}

/// Structured pattern for identifier validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredPatternAst {
    /// Pattern syntax (e.g., "{prefix}:{local_id}")
    pub syntax: String,
    /// Whether pattern uses interpolation
    pub interpolated: bool,
}

/// Type definition AST
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TypeAst {
    /// Type name
    pub name: String,
    /// Description
    pub description: Option<Spanned<Description>>,
    /// Parent type (typeof)
    pub typeof_: Option<Spanned<String>>,
    /// Base primitive type
    pub base: Option<Spanned<String>>,
    /// Type URI
    pub uri: Option<Spanned<String>>,
    /// Pattern validation
    pub pattern: Option<Spanned<String>>,
    /// Minimum value
    pub minimum_value: Option<Spanned<ValueAst>>,
    /// Maximum value
    pub maximum_value: Option<Spanned<ValueAst>>,
    /// Annotations
    pub annotations: Option<Spanned<AnnotationsAst>>,
}

/// Enum definition AST
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EnumAst {
    /// Enum name
    pub name: String,
    /// Description
    pub description: Option<Spanned<Description>>,
    /// Permissible values
    pub permissible_values: IndexMap<String, Spanned<PermissibleValueAst>>,
    /// Code set URI
    pub code_set: Option<Spanned<String>>,
    /// Code set tag
    pub code_set_tag: Option<Spanned<String>>,
    /// Code set version
    pub code_set_version: Option<Spanned<String>>,
    /// Annotations
    pub annotations: Option<Spanned<AnnotationsAst>>,
}

/// Permissible value AST
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PermissibleValueAst {
    /// Value identifier
    pub name: String,
    /// Description
    pub description: Option<Spanned<Description>>,
    /// Semantic meaning URI
    pub meaning: Option<Spanned<String>>,
    /// Aliases
    pub aliases: Vec<Spanned<String>>,
    /// See also
    pub see_also: Vec<Spanned<String>>,
}

/// Subset definition AST
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SubsetAst {
    /// Subset name
    pub name: String,
    /// Description
    pub description: Option<Spanned<Description>>,
    /// Annotations
    pub annotations: Option<Spanned<AnnotationsAst>>,
}

/// Rule AST (preconditions → postconditions)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleAst {
    /// Preconditions
    pub preconditions: IndexMap<String, Spanned<String>>,
    /// Postconditions
    pub postconditions: IndexMap<String, Spanned<String>>,
}

/// Conditional requirement AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalRequirementAst {
    /// Condition slot name
    pub condition: String,
    /// Required slots if condition is met
    pub then_required: Vec<Spanned<String>>,
}

/// Unique key constraint AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UniqueKeyAst {
    /// Unique key name
    pub name: String,
    /// Slots that form the unique key
    pub unique_key_slots: Vec<Spanned<String>>,
}

/// Recursion options AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecursionOptionsAst {
    /// Use Box for recursive types
    pub use_box: bool,
    /// Maximum recursion depth
    pub max_depth: Option<i64>,
}

/// Contributor AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContributorAst {
    /// Contributor name
    pub name: String,
    /// Email address
    pub email: Option<String>,
    /// ORCID identifier
    pub orcid: Option<String>,
}

/// Annotations AST (arbitrary key-value metadata)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AnnotationsAst {
    /// Annotation entries
    pub entries: IndexMap<String, Spanned<AnnotationValueAst>>,
}

/// Annotation value AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationValueAst {
    /// Boolean value
    Bool(bool),
    /// Number value
    Number(f64),
    /// String value
    String(String),
    /// Block string value
    Block(String),
    /// List of values
    List(Vec<Spanned<String>>),
}

/// Generic value AST (for constraints)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueAst {
    /// String value
    String(String),
    /// Number value
    Number(f64),
    /// Integer value
    Integer(i64),
}

impl SchemaAst {
    /// Create a new empty schema AST
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate that required fields are present
    pub fn validate_required_fields(&self) -> Result<(), String> {
        if self.id.is_none() {
            return Err("Schema 'id' field is required".to_string());
        }
        if self.name.is_none() {
            return Err("Schema 'name' field is required".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(0, 10, 1, 1);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 10);
        assert_eq!(span.line, 1);
        assert_eq!(span.column, 1);
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(0, 10, 1, 1);
        let span2 = Span::new(15, 25, 2, 5);
        let merged = span1.merge(&span2);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 25);
        assert_eq!(merged.line, 1);
        assert_eq!(merged.column, 1);
    }

    #[test]
    fn test_spanned_node() {
        let span = Span::new(0, 5, 1, 1);
        let spanned = Spanned::new("test".to_string(), span);
        assert_eq!(spanned.value, "test");
        assert_eq!(spanned.span.start, 0);
    }

    #[test]
    fn test_schema_ast_validation() {
        let mut schema = SchemaAst::new();
        assert!(schema.validate_required_fields().is_err());

        schema.id = Some(Spanned::new(
            "https://example.org/test".to_string(),
            Span::new(0, 5, 1, 1),
        ));
        assert!(schema.validate_required_fields().is_err());

        schema.name = Some(Spanned::new(
            "test_schema".to_string(),
            Span::new(0, 5, 1, 1),
        ));
        assert!(schema.validate_required_fields().is_ok());
    }

    #[test]
    fn test_description_variants() {
        let inline = Description::Inline("Test description".to_string());
        let block = Description::Block("Multi-line\ndescription".to_string());
        assert!(matches!(inline, Description::Inline(_)));
        assert!(matches!(block, Description::Block(_)));
    }

    #[test]
    fn test_range_type() {
        let class_range = RangeType::Class;
        let instance_range = RangeType::Instance;
        assert_eq!(class_range, RangeType::Class);
        assert_eq!(instance_range, RangeType::Instance);
    }
}
