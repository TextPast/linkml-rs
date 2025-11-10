//! Integration tests for LinkML YAML parser using YamlParserV2
//!
//! This module tests the production-ready PEG-based parser implementation.
//! The V2 parser uses LinkMLParser for high-performance parsing with proper
//! dependency injection and centralized error handling.

use linkml_core::error::Result;
use linkml_service::parser::SchemaParser;
use pretty_assertions::assert_eq;

/// Helper function to create test parser
fn create_test_parser() -> impl SchemaParser {
    use std::sync::Arc;
    use linkml_service::file_system_adapter::TokioFileSystemAdapter;
    use linkml_service::parser::YamlParserV2;
    
    let fs_adapter = Arc::new(TokioFileSystemAdapter::new());
    YamlParserV2::new(fs_adapter)
}

/// Test parsing a minimal schema with only required fields
#[test]
fn test_parse_minimal_schema() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema
";

    let schema = parser.parse_str(input)?;
    
    assert_eq!(schema.id, "https://example.org/test");
    assert_eq!(schema.name, "test_schema");
    
    Ok(())
}

/// Test parsing a schema with metadata fields
#[test]
fn test_parse_schema_with_metadata() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema
title: Test Schema
description: A test schema for parser validation
version: 1.0.0
license: CC-BY-4.0
";

    let schema = parser.parse_str(input)?;
    
    assert_eq!(schema.id, "https://example.org/test");
    assert_eq!(schema.name, "test_schema");
    assert_eq!(schema.title.as_ref().unwrap(), "Test Schema");
    assert_eq!(schema.version.as_ref().unwrap(), "1.0.0");
    assert_eq!(schema.license.as_ref().unwrap(), "CC-BY-4.0");
    
    Ok(())
}

/// Test parsing a schema with prefixes
#[test]
fn test_parse_schema_with_prefixes() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema
prefixes:
  linkml: https://w3id.org/linkml/
  skos: http://www.w3.org/2004/02/skos/core#
  rdf: http://www.w3.org/1999/02/22-rdf-syntax-ns#
default_prefix: linkml
";

    let schema = parser.parse_str(input)?;
    
    assert_eq!(schema.prefixes.len(), 3);
    assert_eq!(schema.prefixes.get("linkml").unwrap(), "https://w3id.org/linkml/");
    assert_eq!(schema.prefixes.get("skos").unwrap(), "http://www.w3.org/2004/02/skos/core#");
    assert_eq!(schema.default_prefix.as_ref().unwrap(), "linkml");
    
    Ok(())
}

/// Test parsing a schema with imports
#[test]
fn test_parse_schema_with_imports() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema
imports:
  - linkml:types
  - txp:meta/identifier/identifier/schema
";

    let schema = parser.parse_str(input)?;
    
    assert_eq!(schema.imports.len(), 2);
    assert_eq!(schema.imports[0], "linkml:types");
    assert_eq!(schema.imports[1], "txp:meta/identifier/identifier/schema");
    
    Ok(())
}

/// Test parsing a schema with a simple class definition
#[test]
fn test_parse_schema_with_simple_class() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema

classes:
  Person:
    description: A person
    slots:
      - name
      - email
";

    let schema = parser.parse_str(input)?;
    
    assert_eq!(schema.classes.len(), 1);
    let person_class = schema.classes.get("Person").unwrap();
    assert_eq!(person_class.description.as_ref().unwrap(), "A person");
    assert_eq!(person_class.slots.as_ref().unwrap().len(), 2);
    
    Ok(())
}

/// Test parsing a schema with class inheritance
#[test]
fn test_parse_schema_with_class_inheritance() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema

classes:
  NamedEntity:
    description: An entity with a name
    slots:
      - name
  
  Person:
    is_a: NamedEntity
    description: A person
    slots:
      - email
";

    let schema = parser.parse_str(input)?;
    
    assert_eq!(schema.classes.len(), 2);
    let person_class = schema.classes.get("Person").unwrap();
    assert_eq!(person_class.is_a.as_ref().unwrap(), "NamedEntity");
    
    Ok(())
}

/// Test parsing a schema with slot definitions
#[test]
fn test_parse_schema_with_slots() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema

slots:
  name:
    description: A name
    range: string
    required: true
  
  email:
    description: An email address
    range: string
    pattern: ^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$
";

    let schema = parser.parse_str(input)?;
    
    assert_eq!(schema.slots.len(), 2);
    assert!(schema.slots.contains_key("name"));
    assert!(schema.slots.contains_key("email"));
    
    let name_slot = schema.slots.get("name").unwrap();
    assert_eq!(name_slot.required, Some(true));
    assert_eq!(name_slot.range.as_ref().unwrap(), "string");
    
    Ok(())
}

/// Test parsing a schema with enum definitions
#[test]
fn test_parse_schema_with_enums() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema

enums:
  StatusEnum:
    description: Status values
    permissible_values:
      active:
        description: Currently active
      inactive:
        description: No longer active
      pending:
        description: Awaiting activation
";

    let schema = parser.parse_str(input)?;
    
    assert_eq!(schema.enums.len(), 1);
    let status_enum = schema.enums.get("StatusEnum").unwrap();
    assert_eq!(status_enum.description.as_ref().unwrap(), "Status values");
    assert!(status_enum.permissible_values.is_some());
    
    Ok(())
}

/// Test parsing a schema with type definitions
#[test]
fn test_parse_schema_with_types() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema

types:
  EmailAddress:
    typeof: string
    description: An email address
    pattern: ^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$
";

    let schema = parser.parse_str(input)?;
    
    assert_eq!(schema.types.len(), 1);
    let email_type = schema.types.get("EmailAddress").unwrap();
    assert_eq!(email_type.typeof_field.as_ref().unwrap(), "string");
    
    Ok(())
}

/// Test parsing the actual LinkML meta schema
#[test]
fn test_parse_linkml_meta_schema() -> Result<()> {
    let parser = create_test_parser();
    let input = include_str!("../../schemas/meta.yaml");
    
    let schema = parser.parse_str(input)?;
    
    assert_eq!(schema.id, "https://w3id.org/linkml/meta");
    assert_eq!(schema.name, "meta");
    assert!(schema.classes.len() > 0);
    
    Ok(())
}

/// Test error handling for invalid YAML
#[test]
fn test_parse_invalid_yaml() {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: [invalid: yaml: structure
";

    let result = parser.parse_str(input);
    assert!(result.is_err());
}

/// Test parsing schema with inline and block descriptions
#[test]
fn test_parse_descriptions() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema

classes:
  Person:
    description: A simple inline description
  
  Organization:
    description: |
      A multi-line block description
      that spans multiple lines
      and preserves formatting
";

    let schema = parser.parse_str(input)?;
    
    assert_eq!(schema.classes.len(), 2);
    let person = schema.classes.get("Person").unwrap();
    assert_eq!(person.description.as_ref().unwrap(), "A simple inline description");
    
    let org = schema.classes.get("Organization").unwrap();
    assert!(org.description.as_ref().unwrap().contains("multi-line"));
    
    Ok(())
}

/// Test parsing schema with annotations
#[test]
fn test_parse_annotations() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema

classes:
  Person:
    description: A person
    annotations:
      owl: Thing
      rdfs:label: Person
";

    let schema = parser.parse_str(input)?;
    
    assert_eq!(schema.classes.len(), 1);
    let person = schema.classes.get("Person").unwrap();
    assert!(person.annotations.is_some());
    
    Ok(())
}

/// Test parsing performance with a large schema
#[test]
#[ignore] // Run with --ignored flag for performance testing
fn test_parse_performance_large_schema() -> Result<()> {
    use std::time::Instant;
    
    let parser = create_test_parser();
    
    // Generate a large schema with 100 classes
    let mut schema_parts = vec![
        "id: https://example.org/test".to_string(),
        "name: large_schema".to_string(),
        "classes:".to_string(),
    ];
    
    for i in 0..100 {
        schema_parts.push(format!("  Class{}:", i));
        schema_parts.push(format!("    description: Class number {}", i));
        schema_parts.push("    slots:".to_string());
        schema_parts.push(format!("      - field{}", i));
    }
    
    let input = schema_parts.join("\n");
    
    let start = Instant::now();
    let schema = parser.parse_str(&input)?;
    let duration = start.elapsed();
    
    println!("Parsed {} classes in {:?}", schema.classes.len(), duration);
    
    // Target: <10ms for 100-class schema (~400 lines)
    assert!(duration.as_millis() < 10, "Parser took too long: {:?}", duration);
    
    Ok(())
}

/// Test parsing empty optional fields
#[test]
fn test_parse_optional_fields_absent() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: minimal_schema
";

    let schema = parser.parse_str(input)?;
    
    assert!(schema.title.is_none());
    assert!(schema.description.is_none());
    assert!(schema.version.is_none());
    assert!(schema.license.is_none());
    assert!(schema.default_prefix.is_none());
    
    Ok(())
}

/// Test parsing schema with slot usage
#[test]
fn test_parse_slot_usage() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema

classes:
  Person:
    slots:
      - name
    slot_usage:
      name:
        required: true
        description: Person's full name
";

    let schema = parser.parse_str(input)?;
    
    let person = schema.classes.get("Person").unwrap();
    assert!(person.slot_usage.is_some());
    
    Ok(())
}

/// Test parsing schema with mixins
#[test]
fn test_parse_mixins() -> Result<()> {
    let parser = create_test_parser();
    let input = r"
id: https://example.org/test
name: test_schema

classes:
  Timestamped:
    description: Mixin for timestamped entities
    slots:
      - created_at
      - updated_at
  
  Person:
    mixins:
      - Timestamped
    slots:
      - name
";

    let schema = parser.parse_str(input)?;
    
    let person = schema.classes.get("Person").unwrap();
    assert_eq!(person.mixins.as_ref().unwrap().len(), 1);
    assert_eq!(person.mixins.as_ref().unwrap()[0], "Timestamped");
    
    Ok(())
}
